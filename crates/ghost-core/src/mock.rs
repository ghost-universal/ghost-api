//! Mock Worker Implementation for Testing
//!
//! This module provides a mock worker implementation for testing purposes.
//! It can be configured to simulate various behaviors and responses.

use async_trait::async_trait;
use ghost_schema::{
    Capability, CapabilityManifest, GhostError, PayloadBlob, PayloadContentType,
    Platform, RawContext, WorkerHealth, WorkerStatus, WorkerType,
};
use crate::GhostWorker;

/// Mock worker for testing
pub struct MockWorker {
    /// Worker ID
    id: String,
    /// Capabilities
    capabilities: Vec<Capability>,
    /// Platforms
    platforms: Vec<Platform>,
    /// Response to return
    response: MockResponse,
    /// Whether to fail
    should_fail: bool,
    /// Simulated latency in ms
    latency_ms: u64,
    /// Worker type
    worker_type: WorkerType,
    /// Priority
    priority: u32,
}

/// Response type for mock worker
#[derive(Debug, Clone)]
pub enum MockResponse {
    /// Return success with payload
    Success(PayloadBlob),
    /// Return failure with error
    Failure(GhostError),
    /// Return rate limited
    RateLimited { retry_after: u64 },
    /// Timeout
    Timeout,
}

impl MockWorker {
    /// Creates a new mock worker
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            capabilities: vec![Capability::XRead, Capability::ThreadsRead],
            platforms: vec![Platform::X, Platform::Threads],
            response: MockResponse::Success(PayloadBlob::new(
                br#"{"id":"mock","text":"Mock response"}"#.to_vec(),
                PayloadContentType::Json,
            )),
            should_fail: false,
            latency_ms: 0,
            worker_type: WorkerType::Mock,
            priority: 50,
        }
    }

    /// Creates a mock worker that always succeeds
    pub fn always_success() -> Self {
        Self::new("mock-success")
    }

    /// Creates a mock worker that always fails
    pub fn always_failure() -> Self {
        let mut worker = Self::new("mock-failure");
        worker.should_fail = true;
        worker.response = MockResponse::Failure(GhostError::ScraperError {
            worker: "mock-failure".to_string(),
            message: "Mock failure".to_string(),
        });
        worker
    }

    /// Creates a mock worker that returns rate limited
    pub fn rate_limited(retry_after: u64) -> Self {
        let mut worker = Self::new("mock-rate-limited");
        worker.response = MockResponse::RateLimited { retry_after };
        worker
    }

    /// Creates a mock worker that times out
    pub fn timeout() -> Self {
        let mut worker = Self::new("mock-timeout");
        worker.response = MockResponse::Timeout;
        worker
    }

    /// Sets the capabilities
    pub fn with_capabilities(mut self, capabilities: Vec<Capability>) -> Self {
        self.capabilities = capabilities;
        self
    }

    /// Sets the platforms
    pub fn with_platforms(mut self, platforms: Vec<Platform>) -> Self {
        self.platforms = platforms;
        self
    }

    /// Sets the response
    pub fn with_response(mut self, response: MockResponse) -> Self {
        self.response = response;
        self
    }

    /// Sets the simulated latency
    pub fn with_latency(mut self, latency_ms: u64) -> Self {
        self.latency_ms = latency_ms;
        self
    }

    /// Sets the worker type
    pub fn with_worker_type(mut self, worker_type: WorkerType) -> Self {
        self.worker_type = worker_type;
        self
    }

    /// Sets the priority
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// Sets a success JSON response
    pub fn with_success_json(mut self, json: impl Into<String>) -> Self {
        self.response = MockResponse::Success(PayloadBlob::new(
            json.into().into_bytes(),
            PayloadContentType::Json,
        ));
        self
    }
}

#[async_trait]
impl GhostWorker for MockWorker {
    fn id(&self) -> &str {
        &self.id
    }

    fn capabilities(&self) -> Vec<Capability> {
        self.capabilities.clone()
    }

    fn platforms(&self) -> Vec<Platform> {
        self.platforms.clone()
    }

    async fn execute(&self, _ctx: &RawContext) -> Result<PayloadBlob, GhostError> {
        // Simulate latency
        if self.latency_ms > 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(self.latency_ms)).await;
        }

        match &self.response {
            MockResponse::Success(payload) => Ok(payload.clone()),
            MockResponse::Failure(e) => Err(e.clone()),
            MockResponse::RateLimited { retry_after } => Err(GhostError::RateLimited {
                retry_after: Some(*retry_after),
                platform: Platform::X,
            }),
            MockResponse::Timeout => Err(GhostError::Timeout("Mock timeout".into())),
        }
    }

    fn manifest(&self) -> CapabilityManifest {
        CapabilityManifest {
            worker_id: self.id.clone(),
            version: "0.1.0".to_string(),
            capabilities: self.capabilities.clone(),
            worker_type: self.worker_type,
            max_concurrent: 5,
            health_threshold: 0.7,
            priority: self.priority,
            tags: vec!["mock".to_string()],
        }
    }

    async fn health_check(&self) -> Result<WorkerHealth, GhostError> {
        Ok(WorkerHealth::new())
    }

    fn status(&self) -> WorkerStatus {
        WorkerStatus::Idle
    }

    fn load(&self) -> f64 {
        0.0
    }

    fn worker_type(&self) -> WorkerType {
        self.worker_type
    }

    fn priority(&self) -> u32 {
        self.priority
    }
}

/// Builder for creating mock workers with specific behaviors
pub struct MockWorkerBuilder {
    id: String,
    capabilities: Vec<Capability>,
    platforms: Vec<Platform>,
    response: MockResponse,
    latency_ms: u64,
    worker_type: WorkerType,
    priority: u32,
}

impl MockWorkerBuilder {
    /// Creates a new builder
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            capabilities: vec![Capability::XRead],
            platforms: vec![Platform::X],
            response: MockResponse::Success(PayloadBlob::new(
                br#"{"id":"mock"}"#.to_vec(),
                PayloadContentType::Json,
            )),
            latency_ms: 0,
            worker_type: WorkerType::Mock,
            priority: 50,
        }
    }

    /// Adds a capability
    pub fn capability(mut self, cap: Capability) -> Self {
        self.capabilities.push(cap);
        self
    }

    /// Sets all capabilities
    pub fn capabilities(mut self, capabilities: Vec<Capability>) -> Self {
        self.capabilities = capabilities;
        self
    }

    /// Adds a platform
    pub fn platform(mut self, platform: Platform) -> Self {
        self.platforms.push(platform);
        self
    }

    /// Sets all platforms
    pub fn platforms(mut self, platforms: Vec<Platform>) -> Self {
        self.platforms = platforms;
        self
    }

    /// Sets success response with JSON
    pub fn success_json(mut self, json: impl Into<String>) -> Self {
        self.response = MockResponse::Success(PayloadBlob::new(
            json.into().into_bytes(),
            PayloadContentType::Json,
        ));
        self
    }

    /// Sets success response with payload
    pub fn success_payload(mut self, payload: PayloadBlob) -> Self {
        self.response = MockResponse::Success(payload);
        self
    }

    /// Sets failure response
    pub fn failure(mut self, message: impl Into<String>) -> Self {
        self.response = MockResponse::Failure(GhostError::ScraperError {
            worker: self.id.clone(),
            message: message.into(),
        });
        self
    }

    /// Sets failure with specific error
    pub fn failure_error(mut self, error: GhostError) -> Self {
        self.response = MockResponse::Failure(error);
        self
    }

    /// Sets rate limited response
    pub fn rate_limited(mut self, retry_after: u64) -> Self {
        self.response = MockResponse::RateLimited { retry_after };
        self
    }

    /// Sets timeout response
    pub fn timeout(mut self) -> Self {
        self.response = MockResponse::Timeout;
        self
    }

    /// Sets simulated latency
    pub fn latency(mut self, ms: u64) -> Self {
        self.latency_ms = ms;
        self
    }

    /// Sets the worker type
    pub fn worker_type(mut self, worker_type: WorkerType) -> Self {
        self.worker_type = worker_type;
        self
    }

    /// Sets the priority
    pub fn priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// Builds the mock worker
    pub fn build(self) -> MockWorker {
        MockWorker {
            id: self.id,
            capabilities: self.capabilities,
            platforms: self.platforms,
            response: self.response,
            should_fail: false,
            latency_ms: self.latency_ms,
            worker_type: self.worker_type,
            priority: self.priority,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_worker_success() {
        let worker = MockWorker::always_success();
        let ctx = RawContext::get("https://example.com");
        let result = worker.execute(&ctx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mock_worker_failure() {
        let worker = MockWorker::always_failure();
        let ctx = RawContext::get("https://example.com");
        let result = worker.execute(&ctx).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_worker_rate_limited() {
        let worker = MockWorker::rate_limited(60);
        let ctx = RawContext::get("https://example.com");
        let result = worker.execute(&ctx).await;
        assert!(matches!(result, Err(GhostError::RateLimited { .. })));
    }

    #[tokio::test]
    async fn test_mock_worker_timeout() {
        let worker = MockWorker::timeout();
        let ctx = RawContext::get("https://example.com");
        let result = worker.execute(&ctx).await;
        assert!(matches!(result, Err(GhostError::Timeout(_))));
    }

    #[test]
    fn test_mock_worker_builder() {
        let worker = MockWorkerBuilder::new("test-worker")
            .capability(Capability::XSearch)
            .platform(Platform::Threads)
            .success_json(r#"{"test": true}"#)
            .latency(100)
            .priority(75)
            .build();

        assert_eq!(worker.id(), "test-worker");
        assert!(worker.capabilities().contains(&Capability::XSearch));
        assert!(worker.capabilities().contains(&Capability::XRead)); // Default
        assert!(worker.platforms().contains(&Platform::Threads));
        assert_eq!(worker.priority(), 75);
    }

    #[test]
    fn test_mock_worker_manifest() {
        let worker = MockWorker::always_success();
        let manifest = worker.manifest();

        assert_eq!(manifest.worker_id, "mock-success");
        assert!(manifest.capabilities.contains(&Capability::XRead));
        assert_eq!(manifest.worker_type, WorkerType::Mock);
    }

    #[tokio::test]
    async fn test_mock_worker_health_check() {
        let worker = MockWorker::always_success();
        let health = worker.health_check().await;
        assert!(health.is_ok());
        assert_eq!(health.unwrap().score, 1.0);
    }

    #[tokio::test]
    async fn test_mock_worker_with_latency() {
        let worker = MockWorkerBuilder::new("slow-worker")
            .latency(50)
            .build();

        let ctx = RawContext::get("https://example.com");
        let start = std::time::Instant::now();
        let _ = worker.execute(&ctx).await;
        let elapsed = start.elapsed();

        assert!(elapsed.as_millis() >= 50);
    }
}
