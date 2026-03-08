"""
GhostWorker Implementation for Threads Scraper

Production-ready wrapper that:
1. Implements the GhostWorker interface
2. Injects cookies directly into Playwright context
3. Supports proxy injection
4. Provides health checking
5. Returns raw data for adapter parsing

The external threads-scraper submodule is NOT modified.
"""

import json
import sys
import os
import time
import asyncio
import logging
from datetime import datetime, timezone
from typing import Dict, List, Optional, Any, Tuple
from dataclasses import dataclass, field, asdict
from enum import Enum
from traceback import format_exception

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s [%(levelname)s] %(name)s: %(message)s',
    stream=sys.stderr
)
logger = logging.getLogger('py-threads')

# Add external submodule to path
EXTERNAL_PATH = os.path.join(
    os.path.dirname(__file__), 
    '..', '..', '..', 'external', 'threads-scraper'
)
sys.path.insert(0, os.path.abspath(EXTERNAL_PATH))


# =============================================================================
# Data Classes
# =============================================================================

class WorkerStatus(str, Enum):
    """Worker health status"""
    HEALTHY = "healthy"
    DEGRADED = "degraded"
    UNHEALTHY = "unhealthy"
    UNKNOWN = "unknown"


@dataclass
class GhostContext:
    """
    Unified context for all polyglot workers.
    
    Received from Rust via PyO3 as JSON, deserialized into this structure.
    """
    tenant_id: Optional[str] = None
    cookies: Optional[str] = None      # JSON string or cookie header format
    proxy: Optional[str] = None        # Proxy URL (socks5://, http://, https://)
    user_agent: Optional[str] = None
    timeout_ms: Optional[int] = None   # Request timeout in milliseconds
    metadata: Dict[str, Any] = field(default_factory=dict)
    
    def __post_init__(self):
        if self.metadata is None:
            self.metadata = {}
        if self.timeout_ms is None:
            self.timeout_ms = 30000
    
    @classmethod
    def from_json(cls, json_str: str) -> 'GhostContext':
        """Parse from JSON string"""
        data = json.loads(json_str)
        return cls(**data)
    
    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)


@dataclass
class PayloadBlob:
    """
    Raw data returned by scrapers.
    
    Serialized to JSON and returned to Rust via PyO3.
    """
    data: bytes                                    # Raw content (JSON, HTML, etc.)
    content_type: str                              # MIME type
    source_url: str                                # Original URL
    status_code: int                               # HTTP status code
    headers: Dict[str, str] = field(default_factory=dict)
    metadata: Dict[str, Any] = field(default_factory=dict)
    error: Optional[str] = None                    # Error message if failed
    
    def to_json(self) -> str:
        """Serialize to JSON for FFI transfer"""
        return json.dumps({
            'data': self.data.decode('utf-8') if isinstance(self.data, bytes) else self.data,
            'content_type': self.content_type,
            'source_url': self.source_url,
            'status_code': self.status_code,
            'headers': self.headers,
            'metadata': self.metadata,
            'error': self.error,
        })
    
    @classmethod
    def error_blob(cls, error: str, source_url: str = "") -> 'PayloadBlob':
        """Create an error PayloadBlob"""
        return cls(
            data=json.dumps({'error': error}).encode('utf-8'),
            content_type='application/json',
            source_url=source_url,
            status_code=500,
            error=error,
            metadata={'error': True}
        )


@dataclass
class WorkerHealth:
    """Health status for a worker"""
    status: str                           # 'healthy', 'degraded', 'unhealthy', 'unknown'
    score: float                          # 0.0 - 1.0
    latency_ms: int                       # Average latency
    success_rate: float                   # Recent success rate
    last_check: str                       # ISO timestamp
    message: Optional[str] = None
    details: Dict[str, Any] = field(default_factory=dict)
    
    def to_json(self) -> str:
        return json.dumps(asdict(self))


@dataclass
class ScraperStats:
    """Statistics for a scraping session"""
    pages_visited: int = 0
    posts_found: int = 0
    errors: int = 0
    start_time: float = 0.0
    end_time: float = 0.0
    
    @property
    def duration_ms(self) -> int:
        return int((self.end_time - self.start_time) * 1000)


# =============================================================================
# Threads Worker Implementation
# =============================================================================

class ThreadsWorker:
    """
    GhostWorker implementation for threads-scraper.
    
    This wrapper:
    1. Creates its own Playwright instance with injected cookies/proxy
    2. Reuses the scraping logic from threads_scraper_v2.py
    3. Returns raw JSON for the Rust adapter to parse
    
    The external scraper (threads_scraper_v2.py) is NOT modified.
    """
    
    # Required authentication cookies for Threads
    AUTH_COOKIES = {'sessionid', 'ds_user_id', 'ig_did', 'mid'}
    
    # Load more button selectors (from threads_scraper_v2.py)
    LOAD_MORE_SELECTORS = [
        "text=/View more repli/i",
        "text=/Show hidden repli/i", 
        "text=/View all repli/i",
        "text=/more repli/i",
        "text=/Load more/i",
        "text=/See more/i",
        "text=/Show replies/i",
        "text=/View replies/i",
        # German locale
        "text=/Weitere Antworten/i",
        "text=/Mehr anzeigen/i",
        "text=/Antworten anzeigen/i",
    ]
    
    def __init__(self):
        self._stats = ScraperStats()
        self._last_health_check: Optional[WorkerHealth] = None
    
    @property
    def id(self) -> str:
        """Unique worker identifier"""
        return "py-threads-v2"
    
    @property
    def capabilities(self) -> List[str]:
        """List of capabilities this worker provides"""
        return ["threads_read", "threads_deep_scrape"]
    
    @property
    def platforms(self) -> List[str]:
        """List of platforms this worker supports"""
        return ["threads"]
    
    @property
    def version(self) -> str:
        """Worker version"""
        return "2.0.0"
    
    # =========================================================================
    # Main Execution
    # =========================================================================
    
    def execute(self, ctx: GhostContext) -> PayloadBlob:
        """
        Execute scraping request.
        
        This is the main entry point called from Rust via PyO3.
        
        Args:
            ctx: GhostContext with:
                - metadata['url']: Threads URL to scrape (required)
                - metadata['max_pages']: Max pages to visit (default: 100)
                - metadata['scroll_stale_rounds']: Max scroll rounds without new content (default: 12)
                - cookies: Optional cookie string or JSON
                - proxy: Optional proxy URL
                - user_agent: Optional custom user agent
                - timeout_ms: Request timeout (default: 30000)
        
        Returns:
            PayloadBlob with JSON array of scraped posts
        """
        try:
            return asyncio.run(self._execute_async(ctx))
        except Exception as e:
            logger.exception(f"Execution failed: {e}")
            return PayloadBlob.error_blob(str(e), ctx.metadata.get('url', ''))
    
    async def _execute_async(self, ctx: GhostContext) -> PayloadBlob:
        """Async implementation of execute"""
        from playwright.async_api import async_playwright, TimeoutError as PlaywrightTimeout
        
        # Extract parameters
        url = ctx.metadata.get('url')
        if not url:
            raise ValueError("URL required in context.metadata['url']")
        
        max_pages = ctx.metadata.get('max_pages', 100)
        scroll_stale_rounds = ctx.metadata.get('scroll_stale_rounds', 12)
        timeout_ms = ctx.timeout_ms or 30000
        
        logger.info(f"Starting scrape: {url}")
        logger.info(f"Max pages: {max_pages}, Timeout: {timeout_ms}ms")
        
        # Initialize stats
        self._stats = ScraperStats(start_time=time.time())
        
        extracted_posts: Dict[str, Dict] = {}
        urls_to_visit: List[str] = [url]
        visited_urls: set = set()
        
        try:
            async with async_playwright() as pw:
                # Launch browser with options
                browser = await pw.chromium.launch(
                    headless=True,
                    args=[
                        '--disable-blink-features=AutomationControlled',
                        '--disable-infobars',
                        '--no-sandbox',
                        '--disable-dev-shm-usage',
                    ]
                )
                
                # Create context with cookies and proxy
                context_options = {
                    'locale': 'en-US',
                    'user_agent': ctx.user_agent or self._get_default_user_agent(),
                }
                
                context = await browser.new_context(**context_options)
                
                # ★ COOKIE INJECTION ★
                if ctx.cookies:
                    cookies = self._parse_cookies(ctx.cookies)
                    await context.add_cookies(cookies)
                    auth_cookies = [c['name'] for c in cookies if c['name'] in self.AUTH_COOKIES]
                    logger.info(f"Injected {len(cookies)} cookies (auth: {', '.join(auth_cookies) or 'none'})")
                else:
                    logger.info("No cookies provided - running without authentication")
                
                # ★ PROXY INJECTION ★
                if ctx.proxy:
                    await context.set_proxy(self._parse_proxy(ctx.proxy))
                    logger.info(f"Proxy configured: {ctx.proxy[:30]}...")
                
                page = await context.new_page()
                
                # Set default timeout
                page.set_default_timeout(timeout_ms)
                
                # Response handler for GraphQL/Ajax
                async def handle_response(response):
                    await self._handle_response(response, extracted_posts, urls_to_visit, visited_urls)
                
                page.on("response", handle_response)
                
                # BFS main loop
                while urls_to_visit and len(visited_urls) < max_pages:
                    current_url = urls_to_visit.pop(0)
                    
                    if current_url in visited_urls:
                        continue
                    
                    visited_urls.add(current_url)
                    self._stats.pages_visited += 1
                    
                    logger.debug(f"Visiting [{self._stats.pages_visited}/{max_pages}]: {current_url[:60]}...")
                    
                    try:
                        # Navigate to page
                        await page.goto(current_url, wait_until='networkidle', timeout=timeout_ms)
                        
                        # Parse embedded JSON from HTML
                        await self._parse_html_content(page, extracted_posts, urls_to_visit, visited_urls)
                        
                        # Scroll and expand lazy-loaded content
                        await self._scroll_and_expand(page, extracted_posts, scroll_stale_rounds)
                        
                    except PlaywrightTimeout:
                        logger.warning(f"Timeout on page: {current_url}")
                        self._stats.errors += 1
                    except Exception as e:
                        logger.warning(f"Error on page {current_url}: {e}")
                        self._stats.errors += 1
                
                await browser.close()
        
        except ImportError as e:
            raise RuntimeError(f"Playwright not installed. Run: pip install playwright && playwright install chromium. Error: {e}")
        except Exception as e:
            logger.exception(f"Scraping failed: {e}")
            raise
        
        # Finalize stats
        self._stats.end_time = time.time()
        self._stats.posts_found = len(extracted_posts)
        
        logger.info(f"Scraping complete: {self._stats.pages_visited} pages, {self._stats.posts_found} posts, {self._stats.errors} errors")
        
        # Return raw JSON for adapter to parse
        return PayloadBlob(
            data=json.dumps(list(extracted_posts.values()), ensure_ascii=False).encode('utf-8'),
            content_type='application/json',
            source_url=url,
            status_code=200,
            metadata={
                'scraper': self.id,
                'scraper_version': self.version,
                'max_pages': max_pages,
                'pages_visited': self._stats.pages_visited,
                'posts_found': self._stats.posts_found,
                'errors': self._stats.errors,
                'duration_ms': self._stats.duration_ms,
                'tenant_id': ctx.tenant_id,
                'authenticated': bool(ctx.cookies),
            }
        )
    
    # =========================================================================
    # Response Handlers
    # =========================================================================
    
    async def _handle_response(
        self, 
        response, 
        extracted_posts: Dict[str, Dict],
        urls_to_visit: List[str],
        visited_urls: set
    ):
        """Handle HTTP responses to extract posts from GraphQL/Ajax"""
        url = response.url
        
        try:
            # GraphQL and API responses
            if any(k in url for k in ["graphql", "api/v1"]):
                try:
                    json_data = await response.json()
                    self._extract_posts_from_response(json_data, extracted_posts, urls_to_visit, visited_urls)
                except Exception:
                    pass
            
            # Ajax responses with "for (;;);" prefix
            if "/ajax/" in url:
                try:
                    text = await response.text()
                    if text.startswith("for (;;);"):
                        text = text[len("for (;;);"):]
                    data = json.loads(text)
                    self._extract_posts_from_response(data, extracted_posts, urls_to_visit, visited_urls)
                except Exception:
                    pass
        except Exception:
            pass
    
    def _extract_posts_from_response(
        self,
        data: Dict,
        extracted_posts: Dict[str, Dict],
        urls_to_visit: List[str],
        visited_urls: set
    ):
        """Extract posts from GraphQL/Ajax response using nested_lookup"""
        try:
            from nested_lookup import nested_lookup
        except ImportError:
            logger.warning("nested_lookup not installed - skipping response extraction")
            return
        
        all_posts = nested_lookup("post", data)
        
        for post_obj in all_posts:
            parsed = self._parse_post(post_obj)
            if parsed and parsed.get("id"):
                extracted_posts[parsed["id"]] = parsed
                
                # Queue sub-threads for BFS
                if parsed.get("reply_count", 0) > 0 and parsed.get("code"):
                    new_url = f"https://www.threads.net/@{parsed['author']}/post/{parsed['code']}"
                    if new_url not in visited_urls and new_url not in urls_to_visit:
                        urls_to_visit.append(new_url)
    
    async def _parse_html_content(
        self,
        page,
        extracted_posts: Dict[str, Dict],
        urls_to_visit: List[str],
        visited_urls: set
    ):
        """Parse embedded JSON from HTML page"""
        from parsel import Selector
        
        html = await page.content()
        selector = Selector(text=html)
        
        for script in selector.xpath("//script/text()").getall():
            try:
                # Find JSON objects in script tags
                start_index = script.find("{")
                end_index = script.rfind("}") + 1
                
                if start_index == -1 or end_index == 0:
                    continue
                
                data = json.loads(script[start_index:end_index])
                self._extract_posts_from_response(data, extracted_posts, urls_to_visit, visited_urls)
                
            except (json.JSONDecodeError, AttributeError):
                continue
    
    # =========================================================================
    # Post Parsing
    # =========================================================================
    
    def _parse_post(self, post_data: Dict) -> Optional[Dict]:
        """
        Parse a post from GraphQL response.
        
        Based on threads_scraper_v2.py parse_post function.
        Returns minimal dict for adapter to map to GhostPost.
        """
        if not isinstance(post_data, dict):
            return None
        
        try:
            caption = post_data.get("caption") or {}
            text = caption.get("text")
            
            user = post_data.get("user") or {}
            author = user.get("username")
            
            if not text or not author:
                return None
            
            return {
                "id": post_data.get("id"),
                "code": post_data.get("code"),
                "text": text,
                "author": author,
                "likes": post_data.get("like_count", 0),
                "reply_count": post_data.get("text_post_app_info", {}).get("direct_reply_count", 0),
            }
        except Exception:
            return None
    
    # =========================================================================
    # Scrolling
    # =========================================================================
    
    async def _scroll_and_expand(
        self,
        page,
        extracted_posts: Dict[str, Dict],
        max_stale_rounds: int = 12
    ):
        """
        Scroll and click load-more buttons until no new content appears.
        
        Based on threads_scraper_v2.py scroll_and_expand function.
        """
        previous_count = len(extracted_posts)
        previous_scroll_height = await self._get_scroll_height(page)
        stale_rounds = 0
        
        while stale_rounds < max_stale_rounds:
            # Scroll down
            await page.mouse.wheel(0, 4000)
            await page.wait_for_timeout(1500)
            
            # Click any "load more" / "show replies" buttons
            for sel in self.LOAD_MORE_SELECTORS:
                try:
                    buttons = await page.locator(sel).all()
                    for button in buttons:
                        try:
                            if await button.is_visible(timeout=500):
                                await button.click(timeout=2000)
                                await page.wait_for_timeout(2000)
                                stale_rounds = 0  # Reset on successful click
                        except Exception:
                            pass
                except Exception:
                    pass
            
            # Check for progress
            current_count = len(extracted_posts)
            current_scroll_height = await self._get_scroll_height(page)
            
            if current_count > previous_count or current_scroll_height > previous_scroll_height:
                previous_count = current_count
                previous_scroll_height = current_scroll_height
                stale_rounds = 0
            else:
                stale_rounds += 1
    
    async def _get_scroll_height(self, page) -> int:
        """Get current scroll height of page"""
        return await page.evaluate("document.body.scrollHeight")
    
    # =========================================================================
    # Cookie & Proxy Parsing
    # =========================================================================
    
    def _parse_cookies(self, cookie_str: str) -> List[Dict]:
        """
        Parse cookies from various formats into Playwright format.
        
        Supported formats:
        1. JSON array (from vault): '[{"name":"sessionid","value":"xxx",...}]'
        2. Cookie header: 'sessionid=xxx; ds_user_id=yyy'
        3. JSON object: '{"sessionid": "xxx", "ds_user_id": "yyy"}'
        """
        # Try JSON first (standard from vault)
        try:
            cookies = json.loads(cookie_str)
            
            # JSON array - already in Playwright format
            if isinstance(cookies, list):
                for cookie in cookies:
                    if "domain" not in cookie:
                        cookie["domain"] = ".threads.net"
                    if "path" not in cookie:
                        cookie["path"] = "/"
                return cookies
            
            # JSON object - convert to list
            elif isinstance(cookies, dict):
                return [
                    {
                        "name": k,
                        "value": str(v),
                        "domain": ".threads.net",
                        "path": "/",
                    }
                    for k, v in cookies.items()
                ]
        except json.JSONDecodeError:
            pass
        
        # Parse cookie header format
        cookies = []
        for part in cookie_str.split(';'):
            part = part.strip()
            if '=' in part:
                name, value = part.split('=', 1)
                cookies.append({
                    "name": name.strip(),
                    "value": value.strip(),
                    "domain": ".threads.net",
                    "path": "/",
                })
        return cookies
    
    def _parse_proxy(self, proxy_str: str) -> Dict:
        """
        Parse proxy URL into Playwright format.
        
        Supported formats:
        - socks5://user:pass@host:port
        - http://host:port
        - https://host:port
        """
        return {"server": proxy_str}
    
    def _get_default_user_agent(self) -> str:
        """Get default user agent string"""
        return (
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) "
            "AppleWebKit/537.36 (KHTML, like Gecko) "
            "Chrome/120.0.0.0 Safari/537.36"
        )
    
    # =========================================================================
    # Health Check
    # =========================================================================
    
    def health_check(self, deep: bool = False) -> WorkerHealth:
        """
        Check worker health status.
        
        Args:
            deep: If True, perform a test scrape (slower but more accurate)
        
        Returns:
            WorkerHealth with current status
        """
        now = datetime.now(timezone.utc).isoformat()
        
        # Basic checks
        issues = []
        
        # Check if playwright is available
        try:
            from playwright.async_api import async_playwright
            playwright_ok = True
        except ImportError:
            playwright_ok = False
            issues.append("playwright not installed")
        
        # Check if nested_lookup is available
        try:
            from nested_lookup import nested_lookup
            nested_lookup_ok = True
        except ImportError:
            nested_lookup_ok = False
            issues.append("nested-lookup not installed")
        
        # Check if parsel is available
        try:
            from parsel import Selector
            parsel_ok = True
        except ImportError:
            parsel_ok = False
            issues.append("parsel not installed")
        
        # Determine status
        if playwright_ok and nested_lookup_ok and parsel_ok:
            if deep:
                # TODO: Implement deep health check with test scrape
                status = WorkerStatus.HEALTHY
                score = 1.0
            else:
                status = WorkerStatus.HEALTHY
                score = 1.0
        elif playwright_ok:
            status = WorkerStatus.DEGRADED
            score = 0.5
        else:
            status = WorkerStatus.UNHEALTHY
            score = 0.0
        
        health = WorkerHealth(
            status=status.value,
            score=score,
            latency_ms=self._stats.duration_ms if self._stats.end_time > 0 else 0,
            success_rate=1.0 - (self._stats.errors / max(1, self._stats.pages_visited)),
            last_check=now,
            message="; ".join(issues) if issues else None,
            details={
                'playwright': playwright_ok,
                'nested_lookup': nested_lookup_ok,
                'parsel': parsel_ok,
                'pages_visited': self._stats.pages_visited,
                'posts_found': self._stats.posts_found,
            }
        )
        
        self._last_health_check = health
        return health


# =============================================================================
# FFI Entry Points (called from Rust via PyO3)
# =============================================================================

def execute_worker(context_json: str) -> str:
    """
    Main entry point called from Rust via PyO3.
    
    Args:
        context_json: JSON-serialized GhostContext
        
    Returns:
        JSON-serialized PayloadBlob
    """
    try:
        ctx = GhostContext.from_json(context_json)
        worker = ThreadsWorker()
        result = worker.execute(ctx)
        return result.to_json()
    except Exception as e:
        logger.exception(f"execute_worker failed: {e}")
        error_blob = PayloadBlob.error_blob(str(e))
        return error_blob.to_json()


def get_worker_info() -> str:
    """
    Return worker metadata as JSON.
    
    Used for capability discovery.
    """
    worker = ThreadsWorker()
    return json.dumps({
        'id': worker.id,
        'version': worker.version,
        'capabilities': worker.capabilities,
        'platforms': worker.platforms,
    })


def health_check(deep: bool = False) -> str:
    """
    Return health status as JSON.
    
    Args:
        deep: If True, perform deep health check
    """
    worker = ThreadsWorker()
    health = worker.health_check(deep=deep)
    return health.to_json()


# =============================================================================
# CLI Entry Point (for testing)
# =============================================================================

def _main():
    """CLI entry point for testing"""
    import argparse
    
    parser = argparse.ArgumentParser(description='Ghost API Threads Worker')
    parser.add_argument('url', nargs='?', help='Threads URL to scrape')
    parser.add_argument('--max-pages', type=int, default=10, help='Max pages to visit')
    parser.add_argument('--cookies', type=str, help='Cookie string or JSON file')
    parser.add_argument('--proxy', type=str, help='Proxy URL')
    parser.add_argument('--health-check', action='store_true', help='Run health check')
    parser.add_argument('--info', action='store_true', help='Print worker info')
    
    args = parser.parse_args()
    
    if args.info:
        print(get_worker_info())
        return
    
    if args.health_check:
        print(health_check())
        return
    
    if not args.url:
        parser.print_help()
        return
    
    # Load cookies if file provided
    cookies = None
    if args.cookies:
        if os.path.exists(args.cookies):
            with open(args.cookies, 'r') as f:
                cookies = f.read()
        else:
            cookies = args.cookies
    
    # Create context
    ctx = GhostContext(
        metadata={
            'url': args.url,
            'max_pages': args.max_pages,
        },
        cookies=cookies,
        proxy=args.proxy,
    )
    
    # Execute
    worker = ThreadsWorker()
    result = worker.execute(ctx)
    
    # Output
    print(json.dumps(json.loads(result.data), indent=2, ensure_ascii=False))


if __name__ == "__main__":
    _main()
