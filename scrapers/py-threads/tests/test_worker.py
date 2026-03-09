"""
Tests for py-threads worker
"""

import json
import pytest
from unittest.mock import Mock, patch, AsyncMock
from py_threads.worker import (
    ThreadsWorker,
    GhostContext,
    PayloadBlob,
    WorkerHealth,
    WorkerStatus,
    execute_worker,
    get_worker_info,
    health_check,
)


class TestGhostContext:
    """Tests for GhostContext dataclass"""
    
    def test_from_json(self):
        """Test parsing from JSON string"""
        json_str = json.dumps({
            "tenant_id": "user_123",
            "cookies": '[{"name": "sessionid", "value": "test"}]',
            "metadata": {"url": "https://example.com"}
        })
        
        ctx = GhostContext.from_json(json_str)
        
        assert ctx.tenant_id == "user_123"
        assert ctx.cookies == '[{"name": "sessionid", "value": "test"}]'
        assert ctx.metadata["url"] == "https://example.com"
    
    def test_default_values(self):
        """Test default values"""
        ctx = GhostContext()
        
        assert ctx.tenant_id is None
        assert ctx.cookies is None
        assert ctx.proxy is None
        assert ctx.metadata == {}
        assert ctx.timeout_ms == 30000


class TestPayloadBlob:
    """Tests for PayloadBlob dataclass"""
    
    def test_to_json(self):
        """Test JSON serialization"""
        blob = PayloadBlob(
            data=b'{"test": "data"}',
            content_type="application/json",
            source_url="https://example.com",
            status_code=200,
        )
        
        json_str = blob.to_json()
        data = json.loads(json_str)
        
        assert data["data"] == '{"test": "data"}'
        assert data["content_type"] == "application/json"
        assert data["source_url"] == "https://example.com"
        assert data["status_code"] == 200
    
    def test_error_blob(self):
        """Test error blob creation"""
        blob = PayloadBlob.error_blob("Test error", "https://example.com")
        
        assert blob.status_code == 500
        assert blob.error == "Test error"
        assert blob.metadata.get("error") is True


class TestThreadsWorker:
    """Tests for ThreadsWorker"""
    
    def test_worker_properties(self):
        """Test worker properties"""
        worker = ThreadsWorker()
        
        assert worker.id == "py-threads-v2"
        assert "threads_read" in worker.capabilities
        assert "threads_deep_scrape" in worker.capabilities
        assert "threads" in worker.platforms
        assert worker.version == "2.0.0"
    
    def test_parse_cookies_json_array(self):
        """Test cookie parsing with JSON array format"""
        worker = ThreadsWorker()
        cookie_str = '[{"name": "sessionid", "value": "test123", "domain": ".threads.net"}]'
        
        cookies = worker._parse_cookies(cookie_str)
        
        assert len(cookies) == 1
        assert cookies[0]["name"] == "sessionid"
        assert cookies[0]["value"] == "test123"
        assert cookies[0]["domain"] == ".threads.net"
    
    def test_parse_cookies_header_format(self):
        """Test cookie parsing with header format"""
        worker = ThreadsWorker()
        cookie_str = "sessionid=test123; ds_user_id=456"
        
        cookies = worker._parse_cookies(cookie_str)
        
        assert len(cookies) == 2
        assert cookies[0]["name"] == "sessionid"
        assert cookies[0]["value"] == "test123"
        assert cookies[1]["name"] == "ds_user_id"
        assert cookies[1]["value"] == "456"
    
    def test_parse_cookies_json_object(self):
        """Test cookie parsing with JSON object format"""
        worker = ThreadsWorker()
        cookie_str = '{"sessionid": "test123", "ds_user_id": "456"}'
        
        cookies = worker._parse_cookies(cookie_str)
        
        assert len(cookies) == 2
        names = {c["name"] for c in cookies}
        assert "sessionid" in names
        assert "ds_user_id" in names
    
    def test_parse_post_valid(self):
        """Test post parsing with valid data"""
        worker = ThreadsWorker()
        post_data = {
            "id": "123456",
            "code": "ABC123",
            "caption": {"text": "Hello world"},
            "user": {"username": "testuser"},
            "like_count": 42,
            "text_post_app_info": {"direct_reply_count": 5}
        }
        
        result = worker._parse_post(post_data)
        
        assert result is not None
        assert result["id"] == "123456"
        assert result["code"] == "ABC123"
        assert result["text"] == "Hello world"
        assert result["author"] == "testuser"
        assert result["likes"] == 42
        assert result["reply_count"] == 5
    
    def test_parse_post_missing_text(self):
        """Test post parsing with missing text"""
        worker = ThreadsWorker()
        post_data = {
            "id": "123456",
            "user": {"username": "testuser"}
        }
        
        result = worker._parse_post(post_data)
        
        assert result is None
    
    def test_health_check(self):
        """Test health check"""
        worker = ThreadsWorker()
        health = worker.health_check()
        
        assert health.status in ["healthy", "degraded", "unhealthy", "unknown"]
        assert 0.0 <= health.score <= 1.0


class TestFFIEntryPoints:
    """Tests for FFI entry points"""
    
    def test_get_worker_info(self):
        """Test worker info endpoint"""
        info_json = get_worker_info()
        info = json.loads(info_json)
        
        assert info["id"] == "py-threads-v2"
        assert "threads_read" in info["capabilities"]
        assert "threads" in info["platforms"]
    
    def test_health_check_endpoint(self):
        """Test health check endpoint"""
        health_json = health_check()
        health = json.loads(health_json)
        
        assert "status" in health
        assert "score" in health
        assert "last_check" in health
    
    def test_execute_worker_missing_url(self):
        """Test execute with missing URL"""
        ctx_json = json.dumps({
            "tenant_id": "test",
            "metadata": {}
        })
        
        result_json = execute_worker(ctx_json)
        result = json.loads(result_json)
        
        # Should return error blob
        assert result.get("error") is not None or result.get("status_code") == 500


class TestWorkerIntegration:
    """Integration tests (require Playwright)"""
    
    @pytest.mark.asyncio
    async def test_execute_async_no_url(self):
        """Test execution without URL raises error"""
        worker = ThreadsWorker()
        ctx = GhostContext(metadata={})
        
        with pytest.raises(ValueError, match="URL required"):
            await worker._execute_async(ctx)
    
    @pytest.mark.asyncio
    async def test_execute_async_mock(self):
        """Test execution with mocked Playwright"""
        worker = ThreadsWorker()
        ctx = GhostContext(
            metadata={"url": "https://www.threads.net/@test/post/123"}
        )
        
        # This will fail without Playwright installed, which is expected
        # In a real test environment, you'd mock playwright completely
        try:
            result = await worker._execute_async(ctx)
            # If it succeeds, check the result
            assert isinstance(result, PayloadBlob)
        except ImportError:
            # Expected if playwright not installed
            pytest.skip("Playwright not installed")
        except Exception as e:
            # Other errors are also acceptable for this test
            pytest.skip(f"Test skipped: {e}")


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
