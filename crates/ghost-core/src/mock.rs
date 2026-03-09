//! Mock Worker Implementation for Testing
//!
//! This module provides a mock worker implementation for testing purposes.
//! It can be configured to simulate various behaviors and responses.

use async_trait::async_trait;
use ghost_schema::{
    Capability, CapabilityManifest, GhostError, PayloadBlob, PayloadContentType,
    Platform, RawContext, WorkerHealth, WorkerStatus, WorkerType,
};
use ghost_core::GhostWorker;

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
        // TODO: Implement mock worker construction
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
        }
    }

    /// Creates a mock worker that always succeeds
    pub fn always_success() -> Self {
        // TODO: Implement always-success worker
        Self::new("mock-success")
    }

    /// Creates a mock worker that always fails
    pub fn always_failure() -> Self {
        // TODO: Implement always-failure worker
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
        // TODO: Implement rate-limited worker
        let mut worker = Self::new("mock-rate-limited");
        worker.response = MockResponse::RateLimited { retry_after };
        worker
    }

    /// Sets the capabilities
    pub fn with_capabilities(mut self, capabilities: Vec<Capability>) -> Self {
        // TODO: Implement capabilities setter
        self.capabilities = capabilities;
        self
    }

    /// Sets the platforms
    pub fn with_platforms(mut self, platforms: Vec<Platform>) -> Self {
        // TODO: Implement platforms setter
        self.platforms = platforms;
        self
    }

    /// Sets the response
    pub fn with_response(mut self, response: MockResponse) -> Self {
        // TODO: Implement response setter
        self.response = response;
        self
    }

    /// Sets the simulated latency
    pub fn with_latency(mut self, latency_ms: u64) -> Self {
        // TODO: Implement latency setter
        self.latency_ms = latency_ms;
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
        // TODO: Implement mock execution
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
        // TODO: Implement manifest generation
        CapabilityManifest::new(&self.id, self.capabilities.clone())
    }

    async fn health_check(&self) -> Result<WorkerHealth, GhostError> {
        // TODO: Implement health check
        Ok(WorkerHealth::new())
    }

    fn status(&self) -> WorkerStatus {
        WorkerStatus::Idle
    }

    fn load(&self) -> f64 {
        0.0
    }
}

/// Builder for creating mock workers with specific behaviors
pub struct MockWorkerBuilder {
    id: String,
    capabilities: Vec<Capability>,
    platforms: Vec<Platform>,
    response: MockResponse,
    latency_ms: u64,
}

impl MockWorkerBuilder {
    /// Creates a new builder
    pub fn new(id: impl Into<String>) -> Self {
        // TODO: Implement builder construction
        Self {
            id: id.into(),
            capabilities: vec![Capability::XRead],
            platforms: vec![Platform::X],
            response: MockResponse::Success(PayloadBlob::new(
                br#"{"id":"mock"}"#.to_vec(),
                PayloadContentType::Json,
            )),
            latency_ms: 0,
        }
    }

    /// Adds a capability
    pub fn capability(mut self, cap: Capability) -> Self {
        // TODO: Implement capability addition
        self.capabilities.push(cap);
        self
    }

    /// Adds a platform
    pub fn platform(mut self, platform: Platform) -> Self {
        // TODO: Implement platform addition
        self.platforms.push(platform);
        self
    }

    /// Sets success response with JSON
    pub fn success_json(mut self, json: impl Into<String>) -> Self {
        // TODO: Implement success JSON setter
        self.response = MockResponse::Success(PayloadBlob::new(
            json.into().into_bytes(),
            PayloadContentType::Json,
        ));
        self
    }

    /// Sets failure response
    pub fn failure(mut self, message: impl Into<String>) -> Self {
        // TODO: Implement failure setter
        self.response = MockResponse::Failure(GhostError::ScraperError {
            worker: self.id.clone(),
            message: message.into(),
        });
        self
    }

    /// Sets rate limited response
    pub fn rate_limited(mut self, retry_after: u64) -> Self {
        // TODO: Implement rate limited setter
        self.response = MockResponse::RateLimited { retry_after };
        self
    }

    /// Sets simulated latency
    pub fn latency(mut self, ms: u64) -> Self {
        // TODO: Implement latency setter
        self.latency_ms = ms;
        self
    }

    /// Builds the mock worker
    pub fn build(self) -> MockWorker {
        // TODO: Implement mock worker building
        MockWorker {
            id: self.id,
            capabilities: self.capabilities,
            platforms: self.platforms,
            response: self.response,
            should_fail: false,
            latency_ms: self.latency_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_worker_success() {
        // TODO: Implement mock worker success test
        let worker = MockWorker::always_success();
        let ctx = RawContext::get("https://example.com");
        let result = worker.execute(&ctx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mock_worker_failure() {
        // TODO: Implement mock worker failure test
        let worker = MockWorker::always_failure();
        let ctx = RawContext::get("https://example.com");
        let result = worker.execute(&ctx).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_worker_rate_limited() {
        // TODO: Implement mock worker rate limited test
        let worker = MockWorker::rate_limited(60);
        let ctx = RawContext::get("https://example.com");
        let result = worker.execute(&ctx).await;
        assert!(matches!(result, Err(GhostError::RateLimited { .. })));
    }

    #[test]
    fn test_mock_worker_builder() {
        // TODO: Implement mock worker builder test
        let worker = MockWorkerBuilder::new("test-worker")
            .capability(Capability::XSearch)
            .platform(Platform::Threads)
            .success_json(r#"{"test": true}"#)
            .latency(100)
            .build();

        assert_eq!(worker.id(), "test-worker");
        assert!(worker.capabilities().contains(&Capability::XSearch));
        assert!(worker.platforms().contains(&Platform::Threads));
    }
}
