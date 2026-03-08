//! Mock and VCR testing utilities
//!
//! This module provides types for testing Ghost API without real platform access.
//! Includes mock workers, VCR recording/playback, and chaos testing utilities.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{Capability, GhostError, GhostPost, GhostUser, Platform, PayloadBlob, PayloadContentType};

// ============================================================================
// Mock Worker Types
// ============================================================================

/// Configuration for mock worker behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockConfig {
    /// Whether to simulate delays
    pub simulate_delays: bool,
    /// Delay range in milliseconds [min, max]
    pub delay_range_ms: (u64, u64),
    /// Whether to simulate failures
    pub simulate_failures: bool,
    /// Failure rate (0.0 - 1.0)
    pub failure_rate: f64,
    /// Whether to simulate rate limits
    pub simulate_rate_limits: bool,
    /// Rate limit after N requests (0 = never)
    pub rate_limit_after: u32,
    /// Responses to return in sequence
    pub responses: Vec<MockResponse>,
    /// Default response when responses exhausted
    pub default_response: MockResponseType,
}

impl MockConfig {
    /// Creates a new mock config with defaults
    pub fn new() -> Self {
        // TODO: Implement mock config construction
        Self {
            simulate_delays: false,
            delay_range_ms: (100, 500),
            simulate_failures: false,
            failure_rate: 0.0,
            simulate_rate_limits: false,
            rate_limit_after: 0,
            responses: Vec::new(),
            default_response: MockResponseType::Success,
        }
    }

    /// Creates a config that always succeeds
    pub fn always_success() -> Self {
        // TODO: Implement success config
        Self {
            default_response: MockResponseType::Success,
            ..Self::new()
        }
    }

    /// Creates a config that always fails
    pub fn always_failure() -> Self {
        // TODO: Implement failure config
        Self {
            default_response: MockResponseType::Failure {
                error: "Mock failure".to_string(),
            },
            ..Self::new()
        }
    }

    /// Creates a config that simulates rate limiting
    pub fn rate_limited(after: u32) -> Self {
        // TODO: Implement rate limited config
        Self {
            simulate_rate_limits: true,
            rate_limit_after: after,
            ..Self::new()
        }
    }

    /// Sets the delay range
    pub fn with_delay(mut self, min_ms: u64, max_ms: u64) -> Self {
        // TODO: Implement delay setter
        self.delay_range_ms = (min_ms, max_ms);
        self.simulate_delays = true;
        self
    }

    /// Sets the failure rate
    pub fn with_failure_rate(mut self, rate: f64) -> Self {
        // TODO: Implement failure rate setter
        self.failure_rate = rate.clamp(0.0, 1.0);
        self.simulate_failures = rate > 0.0;
        self
    }

    /// Adds a response to the sequence
    pub fn with_response(mut self, response: MockResponse) -> Self {
        // TODO: Implement response addition
        self.responses.push(response);
        self
    }

    /// Validates the configuration
    pub fn validate(&self) -> Result<(), GhostError> {
        // TODO: Implement config validation
        if self.delay_range_ms.0 > self.delay_range_ms.1 {
            return Err(GhostError::ValidationError(
                "delay_range_ms min must be <= max".into(),
            ));
        }
        if self.failure_rate > 1.0 {
            return Err(GhostError::ValidationError(
                "failure_rate must be <= 1.0".into(),
            ));
        }
        Ok(())
    }
}

impl Default for MockConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock response type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MockResponseType {
    /// Successful response
    Success,
    /// Failure response
    Failure {
        /// Error message
        error: String,
    },
    /// Rate limited response
    RateLimited {
        /// Seconds until retry
        retry_after: u64,
    },
    /// Timeout response
    Timeout,
    /// Network error
    NetworkError {
        /// Error message
        error: String,
    },
    /// WAF challenge
    WafChallenge {
        /// Challenge type
        challenge_type: String,
    },
}

/// Mock response definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockResponse {
    /// Response type
    pub response_type: MockResponseType,
    /// Platform for this response
    pub platform: Option<Platform>,
    /// Capability for this response
    pub capability: Option<Capability>,
    /// Custom data to return
    pub data: Option<MockData>,
    /// Delay in milliseconds
    pub delay_ms: Option<u64>,
}

impl MockResponse {
    /// Creates a success response
    pub fn success() -> Self {
        // TODO: Implement success response construction
        Self {
            response_type: MockResponseType::Success,
            platform: None,
            capability: None,
            data: None,
            delay_ms: None,
        }
    }

    /// Creates a failure response
    pub fn failure(error: impl Into<String>) -> Self {
        // TODO: Implement failure response construction
        Self {
            response_type: MockResponseType::Failure { error: error.into() },
            platform: None,
            capability: None,
            data: None,
            delay_ms: None,
        }
    }

    /// Creates a rate limited response
    pub fn rate_limited(retry_after: u64) -> Self {
        // TODO: Implement rate limited response construction
        Self {
            response_type: MockResponseType::RateLimited { retry_after },
            platform: None,
            capability: None,
            data: None,
            delay_ms: None,
        }
    }

    /// Sets the platform
    pub fn for_platform(mut self, platform: Platform) -> Self {
        // TODO: Implement platform setter
        self.platform = Some(platform);
        self
    }

    /// Sets the capability
    pub fn for_capability(mut self, capability: Capability) -> Self {
        // TODO: Implement capability setter
        self.capability = Some(capability);
        self
    }

    /// Sets the delay
    pub fn with_delay(mut self, delay_ms: u64) -> Self {
        // TODO: Implement delay setter
        self.delay_ms = Some(delay_ms);
        self
    }

    /// Sets the data
    pub fn with_data(mut self, data: MockData) -> Self {
        // TODO: Implement data setter
        self.data = Some(data);
        self
    }
}

/// Mock data to return
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MockData {
    /// Single post
    Post(GhostPost),
    /// Multiple posts
    Posts(Vec<GhostPost>),
    /// User profile
    User(GhostUser),
    /// Raw JSON data
    Json(serde_json::Value),
    /// Raw HTML data
    Html(String),
    /// Raw text data
    Text(String),
}

impl MockData {
    /// Creates post mock data
    pub fn post(post: GhostPost) -> Self {
        // TODO: Implement post data construction
        Self::Post(post)
    }

    /// Creates posts mock data
    pub fn posts(posts: Vec<GhostPost>) -> Self {
        // TODO: Implement posts data construction
        Self::Posts(posts)
    }

    /// Creates user mock data
    pub fn user(user: GhostUser) -> Self {
        // TODO: Implement user data construction
        Self::User(user)
    }

    /// Creates JSON mock data
    pub fn json(value: serde_json::Value) -> Self {
        // TODO: Implement json data construction
        Self::Json(value)
    }

    /// Converts to a PayloadBlob
    pub fn to_payload(&self) -> Result<PayloadBlob, GhostError> {
        // TODO: Implement payload conversion
        let (data, content_type) = match self {
            MockData::Post(post) => {
                let json = serde_json::to_vec(post)?;
                (json, PayloadContentType::Json)
            }
            MockData::Posts(posts) => {
                let json = serde_json::to_vec(posts)?;
                (json, PayloadContentType::Json)
            }
            MockData::User(user) => {
                let json = serde_json::to_vec(user)?;
                (json, PayloadContentType::Json)
            }
            MockData::Json(value) => {
                let json = serde_json::to_vec(value)?;
                (json, PayloadContentType::Json)
            }
            MockData::Html(html) => {
                (html.as_bytes().to_vec(), PayloadContentType::Html)
            }
            MockData::Text(text) => {
                (text.as_bytes().to_vec(), PayloadContentType::Text)
            }
        };
        Ok(PayloadBlob::new(data, content_type))
    }
}

// ============================================================================
// VCR Types
// ============================================================================

/// VCR recording configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VcrConfig {
    /// Whether VCR is enabled
    pub enabled: bool,
    /// VCR mode
    pub mode: VcrMode,
    /// Directory to store cassettes
    pub cassette_dir: String,
    /// Whether to record headers
    pub record_headers: bool,
    /// Headers to exclude from recording
    pub exclude_headers: Vec<String>,
    /// Whether to match requests strictly
    pub strict_matching: bool,
    /// Request matching rules
    pub matching: Vec<RequestMatching>,
}

impl VcrConfig {
    /// Creates a new VCR config
    pub fn new() -> Self {
        // TODO: Implement VCR config construction
        Self {
            enabled: false,
            mode: VcrMode::Playback,
            cassette_dir: "cassettes".to_string(),
            record_headers: true,
            exclude_headers: vec!["authorization".to_string(), "cookie".to_string()],
            strict_matching: false,
            matching: vec![RequestMatching::Url, RequestMatching::Method],
        }
    }

    /// Creates a record mode config
    pub fn record() -> Self {
        // TODO: Implement record config
        Self {
            mode: VcrMode::Record,
            enabled: true,
            ..Self::new()
        }
    }

    /// Creates a playback mode config
    pub fn playback() -> Self {
        // TODO: Implement playback config
        Self {
            mode: VcrMode::Playback,
            enabled: true,
            ..Self::new()
        }
    }

    /// Sets the cassette directory
    pub fn with_cassette_dir(mut self, dir: impl Into<String>) -> Self {
        // TODO: Implement cassette dir setter
        self.cassette_dir = dir.into();
        self
    }

    /// Sets the mode
    pub fn with_mode(mut self, mode: VcrMode) -> Self {
        // TODO: Implement mode setter
        self.mode = mode;
        self
    }
}

impl Default for VcrConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// VCR mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VcrMode {
    /// Record new interactions
    Record,
    /// Playback recorded interactions
    Playback,
    /// Playback if exists, otherwise record
    Auto,
    /// Disable VCR (pass through)
    Disabled,
}

/// Request matching rules for VCR
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestMatching {
    /// Match by URL
    Url,
    /// Match by HTTP method
    Method,
    /// Match by request body
    Body,
    /// Match by headers
    Headers,
    /// Match by query parameters
    Query,
}

/// A VCR cassette (recorded interactions)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cassette {
    /// Cassette name
    pub name: String,
    /// Recorded interactions
    pub interactions: Vec<Interaction>,
    /// Recording timestamp
    pub recorded_at: i64,
}

impl Cassette {
    /// Creates a new cassette
    pub fn new(name: impl Into<String>) -> Self {
        // TODO: Implement cassette construction
        Self {
            name: name.into(),
            interactions: Vec::new(),
            recorded_at: 0, // TODO: Use actual timestamp
        }
    }

    /// Adds an interaction
    pub fn add_interaction(&mut self, interaction: Interaction) {
        // TODO: Implement interaction addition
        self.interactions.push(interaction);
    }

    /// Finds a matching interaction
    pub fn find_match(&self, request: &RecordedRequest) -> Option<&Interaction> {
        // TODO: Implement interaction matching
        self.interactions.iter().find(|i| i.matches(request))
    }

    /// Loads a cassette from file
    pub fn load(path: &str) -> Result<Self, GhostError> {
        // TODO: Implement cassette loading
        let _content = std::fs::read_to_string(path)
            .map_err(|e| GhostError::ParseError(format!("Failed to load cassette: {}", e)))?;
        Ok(Self::new(""))
    }

    /// Saves the cassette to file
    pub fn save(&self, path: &str) -> Result<(), GhostError> {
        // TODO: Implement cassette saving
        let _json = serde_json::to_string_pretty(self)
            .map_err(|e| GhostError::ParseError(e.to_string()))?;
        Ok(())
    }
}

/// A recorded interaction (request + response)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interaction {
    /// The request
    pub request: RecordedRequest,
    /// The response
    pub response: RecordedResponse,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

impl Interaction {
    /// Creates a new interaction
    pub fn new(request: RecordedRequest, response: RecordedResponse, duration_ms: u64) -> Self {
        // TODO: Implement interaction construction
        Self {
            request,
            response,
            duration_ms,
        }
    }

    /// Checks if this interaction matches a request
    pub fn matches(&self, _request: &RecordedRequest) -> bool {
        // TODO: Implement request matching
        false
    }
}

/// A recorded request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedRequest {
    /// HTTP method
    pub method: String,
    /// Request URL
    pub url: String,
    /// Request headers
    pub headers: HashMap<String, String>,
    /// Request body
    pub body: Option<String>,
    /// Query parameters
    pub query: HashMap<String, String>,
}

impl RecordedRequest {
    /// Creates a new recorded request
    pub fn new(method: impl Into<String>, url: impl Into<String>) -> Self {
        // TODO: Implement recorded request construction
        Self {
            method: method.into(),
            url: url.into(),
            headers: HashMap::new(),
            body: None,
            query: HashMap::new(),
        }
    }

    /// Adds a header
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        // TODO: Implement header addition
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Sets the body
    pub fn with_body(mut self, body: impl Into<String>) -> Self {
        // TODO: Implement body setter
        self.body = Some(body.into());
        self
    }
}

/// A recorded response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedResponse {
    /// HTTP status code
    pub status: u16,
    /// Response headers
    pub headers: HashMap<String, String>,
    /// Response body
    pub body: Option<String>,
}

impl RecordedResponse {
    /// Creates a new recorded response
    pub fn new(status: u16) -> Self {
        // TODO: Implement recorded response construction
        Self {
            status,
            headers: HashMap::new(),
            body: None,
        }
    }

    /// Creates from a PayloadBlob
    pub fn from_payload(payload: &PayloadBlob) -> Self {
        // TODO: Implement payload conversion
        Self {
            status: payload.status_code,
            headers: payload.headers.clone(),
            body: payload.as_text().ok().map(|s| s.to_string()),
        }
    }

    /// Converts to a PayloadBlob
    pub fn to_payload(&self) -> PayloadBlob {
        // TODO: Implement payload conversion
        let data = self.body.as_ref()
            .map(|s| s.as_bytes().to_vec())
            .unwrap_or_default();
        PayloadBlob::new(data, PayloadContentType::Json)
    }
}

// ============================================================================
// Chaos Testing Types
// ============================================================================

/// Chaos testing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaosConfig {
    /// Whether chaos testing is enabled
    pub enabled: bool,
    /// Probability of injecting errors (0.0 - 1.0)
    pub error_injection_rate: f64,
    /// Probability of injecting latency (0.0 - 1.0)
    pub latency_injection_rate: f64,
    /// Latency range to inject [min_ms, max_ms]
    pub latency_range_ms: (u64, u64),
    /// Types of errors to inject
    pub error_types: Vec<ChaosErrorType>,
    /// Probability of worker failures (0.0 - 1.0)
    pub worker_failure_rate: f64,
}

impl ChaosConfig {
    /// Creates a new chaos config
    pub fn new() -> Self {
        // TODO: Implement chaos config construction
        Self {
            enabled: false,
            error_injection_rate: 0.0,
            latency_injection_rate: 0.0,
            latency_range_ms: (100, 5000),
            error_types: Vec::new(),
            worker_failure_rate: 0.0,
        }
    }

    /// Creates a chaos config with default errors
    pub fn with_errors(rate: f64) -> Self {
        // TODO: Implement error chaos config
        Self {
            enabled: rate > 0.0,
            error_injection_rate: rate,
            error_types: vec![
                ChaosErrorType::Timeout,
                ChaosErrorType::NetworkError,
                ChaosErrorType::RateLimited,
            ],
            ..Self::new()
        }
    }

    /// Creates a chaos config with latency injection
    pub fn with_latency(rate: f64, min_ms: u64, max_ms: u64) -> Self {
        // TODO: Implement latency chaos config
        Self {
            enabled: rate > 0.0,
            latency_injection_rate: rate,
            latency_range_ms: (min_ms, max_ms),
            ..Self::new()
        }
    }

    /// Validates the configuration
    pub fn validate(&self) -> Result<(), GhostError> {
        // TODO: Implement config validation
        if self.error_injection_rate > 1.0 || self.latency_injection_rate > 1.0 {
            return Err(GhostError::ValidationError(
                "Injection rates must be <= 1.0".into(),
            ));
        }
        Ok(())
    }
}

impl Default for ChaosConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Types of errors to inject during chaos testing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChaosErrorType {
    /// Timeout errors
    Timeout,
    /// Network errors
    NetworkError,
    /// Rate limit errors
    RateLimited,
    /// WAF challenge
    WafChallenge,
    /// Authentication errors
    AuthError,
    /// Worker crash
    WorkerCrash,
}

// ============================================================================
// Data Generator Types
// ============================================================================

/// Configuration for generating mock data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataGeneratorConfig {
    /// Seed for reproducible generation
    pub seed: Option<u64>,
    /// Platform for generated data
    pub platform: Platform,
    /// Number of posts to generate
    pub post_count: usize,
    /// Number of users to generate
    pub user_count: usize,
    /// Include media in posts
    pub include_media: bool,
    /// Media probability (0.0 - 1.0)
    pub media_probability: f64,
}

impl DataGeneratorConfig {
    /// Creates a new generator config
    pub fn new(platform: Platform) -> Self {
        // TODO: Implement generator config construction
        Self {
            seed: None,
            platform,
            post_count: 10,
            user_count: 5,
            include_media: true,
            media_probability: 0.3,
        }
    }

    /// Sets the seed
    pub fn with_seed(mut self, seed: u64) -> Self {
        // TODO: Implement seed setter
        self.seed = Some(seed);
        self
    }

    /// Sets the post count
    pub fn with_posts(mut self, count: usize) -> Self {
        // TODO: Implement post count setter
        self.post_count = count;
        self
    }

    /// Sets the user count
    pub fn with_users(mut self, count: usize) -> Self {
        // TODO: Implement user count setter
        self.user_count = count;
        self
    }
}

/// Mock data generator
#[derive(Debug, Clone)]
pub struct MockDataGenerator {
    config: DataGeneratorConfig,
}

impl MockDataGenerator {
    /// Creates a new generator
    pub fn new(config: DataGeneratorConfig) -> Self {
        // TODO: Implement generator construction
        Self { config }
    }

    /// Generates a single post
    pub fn generate_post(&self) -> GhostPost {
        // TODO: Implement post generation
        GhostPost::new("mock_id", self.config.platform, "Mock post content")
    }

    /// Generates multiple posts
    pub fn generate_posts(&self) -> Vec<GhostPost> {
        // TODO: Implement posts generation
        (0..self.config.post_count)
            .map(|_| self.generate_post())
            .collect()
    }

    /// Generates a single user
    pub fn generate_user(&self) -> GhostUser {
        // TODO: Implement user generation
        GhostUser::new("mock_user_id", self.config.platform, "mock_user")
    }

    /// Generates multiple users
    pub fn generate_users(&self) -> Vec<GhostUser> {
        // TODO: Implement users generation
        (0..self.config.user_count)
            .map(|_| self.generate_user())
            .collect()
    }

    /// Generates a search result
    pub fn generate_search_results(&self, query: &str) -> Vec<GhostPost> {
        // TODO: Implement search results generation
        let _ = query;
        self.generate_posts()
    }
}
