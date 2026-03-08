# py-threads

Ghost API Threads Worker - Production-ready wrapper for threads-scraper.

## Overview

This package provides a GhostWorker implementation for scraping Threads.net conversations using Playwright. It integrates with the ghost-api Rust core via PyO3 FFI.

### Key Features

- **Direct Cookie Injection**: Cookies are injected directly into Playwright context - no file writing needed
- **BFS Traversal**: Deep scrape conversation trees with nested replies
- **Proxy Support**: Full proxy support (SOCKS5, HTTP, HTTPS)
- **Health Monitoring**: Built-in health check for worker status
- **Production Ready**: Comprehensive error handling, logging, and statistics

## Installation

```bash
# Install dependencies
pip install -r requirements.txt

# Install Playwright browser
playwright install chromium
```

## Usage

### As a GhostWorker (via Rust/PyO3)

```python
from py_threads import ThreadsWorker, GhostContext

# Create context with authentication
ctx = GhostContext(
    tenant_id="user_123",
    cookies='[{"name": "sessionid", "value": "...", "domain": ".threads.net"}]',
    metadata={
        "url": "https://www.threads.net/@username/post/abc123",
        "max_pages": 50,
    }
)

# Execute scraping
worker = ThreadsWorker()
result = worker.execute(ctx)

# Access raw JSON data
posts = json.loads(result.data)
```

### Command Line (Testing)

```bash
# Basic scrape
python -m py_threads "https://www.threads.net/@username/post/abc123"

# With cookies
python -m py_threads "https://www.threads.net/@username/post/abc123" --cookies cookies.json

# Health check
python -m py_threads --health-check
```

## FFI Entry Points

These functions are called from Rust via PyO3:

| Function | Description |
|----------|-------------|
| `execute_worker(context_json)` | Execute scraping request |
| `get_worker_info()` | Return worker metadata |
| `health_check(deep=False)` | Return health status |

## Configuration

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `max_pages` | int | 100 | Maximum pages to visit during BFS |
| `timeout_ms` | int | 30000 | Request timeout in milliseconds |
| `scroll_stale_rounds` | int | 12 | Max scroll rounds without new content |
| `scroll_delay_ms` | int | 1500 | Delay between scroll operations |

## Cookie Format

The worker accepts cookies in multiple formats:

### JSON Array (Playwright format) - Recommended
```json
[
  {"name": "sessionid", "value": "abc123...", "domain": ".threads.net", "path": "/"},
  {"name": "ds_user_id", "value": "123456789", "domain": ".threads.net", "path": "/"}
]
```

### Cookie Header Format
```
sessionid=abc123...; ds_user_id=123456789
```

### JSON Object Format
```json
{"sessionid": "abc123...", "ds_user_id": "123456789"}
```

## Output Format

Returns an array of posts:

```json
[
  {
    "id": "17944322110154832",
    "code": "C_9xYzABC123",
    "text": "This is a reply to the main post...",
    "author": "username",
    "likes": 42,
    "reply_count": 5
  }
]
```

## External Dependency

This wrapper uses [threads-scraper](https://github.com/vdite/threads-scraper) as a git submodule. The external code is **never modified** - all adaptations happen in this wrapper layer.

## License

MIT License - See LICENSE file for details.
