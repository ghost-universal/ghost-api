//! Core types for the Ghost AST

use serde::{Deserialize, Serialize};

use crate::{Platform, Capability};

/// The unified struct representing a post across all platforms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhostPost {
    /// Unique identifier for the post
    pub id: String,
    /// The platform this post originates from
    pub platform: Platform,
    /// The text content of the post
    pub text: String,
    /// The author of the post
    pub author: GhostUser,
    /// Media attachments (images, videos, etc.)
    pub media: Vec<GhostMedia>,
    /// Timestamp of when the post was created
    pub created_at: i64,
    /// Number of likes/reactions
    pub like_count: Option<u64>,
    /// Number of retweets/reposts
    pub repost_count: Option<u64>,
    /// Number of replies
    pub reply_count: Option<u64>,
    /// Post this is replying to (if any)
    pub in_reply_to: Option<String>,
    /// Quoted post (if any)
    pub quoted_post: Option<Box<GhostPost>>,
    /// Raw platform-specific data for debugging
    pub raw_metadata: Option<serde_json::Value>,
}

impl GhostPost {
    /// Creates a new GhostPost with minimal required fields
    pub fn new(id: impl Into<String>, platform: Platform, text: impl Into<String>) -> Self {
        // TODO: Implement GhostPost construction with default values
        Self {
            id: id.into(),
            platform,
            text: text.into(),
            author: GhostUser::default(),
            media: Vec::new(),
            created_at: 0,
            like_count: None,
            repost_count: None,
            reply_count: None,
            in_reply_to: None,
            quoted_post: None,
            raw_metadata: None,
        }
    }

    /// Validates the post data
    pub fn validate(&self) -> Result<(), crate::GhostError> {
        // TODO: Implement validation logic for GhostPost
        Ok(())
    }

    /// Converts the post to a platform-specific format
    pub fn to_platform_format(&self, target: Platform) -> Result<serde_json::Value, crate::GhostError> {
        // TODO: Implement platform-specific serialization
        Err(crate::GhostError::NotImplemented("to_platform_format".into()))
    }
}

/// The unified struct representing a user across all platforms.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GhostUser {
    /// Unique identifier for the user
    pub id: String,
    /// The platform this user belongs to
    pub platform: Platform,
    /// Username/handle
    pub username: String,
    /// Display name
    pub display_name: Option<String>,
    /// Profile description/bio
    pub bio: Option<String>,
    /// Profile picture URL
    pub avatar_url: Option<String>,
    /// Banner image URL
    pub banner_url: Option<String>,
    /// Number of followers
    pub followers_count: Option<u64>,
    /// Number of accounts following
    pub following_count: Option<u64>,
    /// Number of posts
    pub posts_count: Option<u64>,
    /// Whether the account is verified
    pub is_verified: Option<bool>,
    /// Whether the account is private
    pub is_private: Option<bool>,
    /// Account creation timestamp
    pub created_at: Option<i64>,
    /// Raw platform-specific data
    pub raw_metadata: Option<serde_json::Value>,
}

impl GhostUser {
    /// Creates a new GhostUser with minimal required fields
    pub fn new(id: impl Into<String>, platform: Platform, username: impl Into<String>) -> Self {
        // TODO: Implement GhostUser construction with default values
        Self {
            id: id.into(),
            platform,
            username: username.into(),
            ..Default::default()
        }
    }

    /// Validates the user data
    pub fn validate(&self) -> Result<(), crate::GhostError> {
        // TODO: Implement validation logic for GhostUser
        Ok(())
    }
}

/// Media attachment in a post
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhostMedia {
    /// Unique identifier for the media
    pub id: String,
    /// Type of media
    pub media_type: MediaType,
    /// URL to the media file
    pub url: String,
    /// Preview/thumbnail URL
    pub preview_url: Option<String>,
    /// Width in pixels
    pub width: Option<u32>,
    /// Height in pixels
    pub height: Option<u32>,
    /// Duration in seconds (for video/audio)
    pub duration_secs: Option<f64>,
    /// Alt text for accessibility
    pub alt_text: Option<String>,
}

impl GhostMedia {
    /// Creates a new GhostMedia
    pub fn new(media_type: MediaType, url: impl Into<String>) -> Self {
        // TODO: Implement GhostMedia construction
        Self {
            id: String::new(),
            media_type,
            url: url.into(),
            preview_url: None,
            width: None,
            height: None,
            duration_secs: None,
            alt_text: None,
        }
    }

    /// Validates the media data
    pub fn validate(&self) -> Result<(), crate::GhostError> {
        // TODO: Implement validation logic for GhostMedia
        Ok(())
    }
}

/// Type of media content
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaType {
    Image,
    Video,
    Gif,
    Audio,
    Unknown,
}

/// Raw, unprocessed data returned by scrapers (HTML, JSON, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayloadBlob {
    /// The raw content as bytes
    pub data: Vec<u8>,
    /// Content type (html, json, etc.)
    pub content_type: PayloadContentType,
    /// Source URL where this data was fetched from
    pub source_url: Option<String>,
    /// HTTP status code
    pub status_code: u16,
    /// Response headers
    pub headers: std::collections::HashMap<String, String>,
    /// Timestamp when this blob was created
    pub fetched_at: i64,
}

impl PayloadBlob {
    /// Creates a new PayloadBlob
    pub fn new(data: Vec<u8>, content_type: PayloadContentType) -> Self {
        // TODO: Implement PayloadBlob construction
        Self {
            data,
            content_type,
            source_url: None,
            status_code: 200,
            headers: std::collections::HashMap::new(),
            fetched_at: 0,
        }
    }

    /// Parses the blob as UTF-8 string
    pub fn as_text(&self) -> Result<&str, crate::GhostError> {
        // TODO: Implement UTF-8 parsing with proper error handling
        std::str::from_utf8(&self.data)
            .map_err(|e| crate::GhostError::ParseError(e.to_string()))
    }

    /// Parses the blob as JSON
    pub fn as_json<T: for<'de> Deserialize<'de>>(&self) -> Result<T, crate::GhostError> {
        // TODO: Implement JSON parsing with proper error handling
        serde_json::from_slice(&self.data)
            .map_err(|e| crate::GhostError::ParseError(e.to_string()))
    }
}

/// Type of payload content
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PayloadContentType {
    Html,
    Json,
    Xml,
    Binary,
    Text,
    Unknown,
}

/// Raw context passed to scrapers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawContext {
    /// The URL or endpoint to fetch
    pub target: String,
    /// HTTP method
    pub method: HttpMethod,
    /// Request headers
    pub headers: std::collections::HashMap<String, String>,
    /// Request body
    pub body: Option<Vec<u8>>,
    /// Proxy configuration
    pub proxy: Option<ProxyConfig>,
    /// Session/cookie data
    pub session: Option<SessionData>,
    /// Platform-specific parameters
    pub platform_params: serde_json::Value,
}

impl RawContext {
    /// Creates a new RawContext for a GET request
    pub fn get(target: impl Into<String>) -> Self {
        // TODO: Implement RawContext construction for GET requests
        Self {
            target: target.into(),
            method: HttpMethod::Get,
            headers: std::collections::HashMap::new(),
            body: None,
            proxy: None,
            session: None,
            platform_params: serde_json::Value::Null,
        }
    }

    /// Creates a new RawContext for a POST request
    pub fn post(target: impl Into<String>, body: Option<Vec<u8>>) -> Self {
        // TODO: Implement RawContext construction for POST requests
        Self {
            target: target.into(),
            method: HttpMethod::Post,
            headers: std::collections::HashMap::new(),
            body,
            proxy: None,
            session: None,
            platform_params: serde_json::Value::Null,
        }
    }

    /// Adds a header to the context
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        // TODO: Implement header addition with validation
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Sets the proxy configuration
    pub fn with_proxy(mut self, proxy: ProxyConfig) -> Self {
        // TODO: Implement proxy configuration
        self.proxy = Some(proxy);
        self
    }

    /// Sets the session data
    pub fn with_session(mut self, session: SessionData) -> Self {
        // TODO: Implement session configuration
        self.session = Some(session);
        self
    }
}

/// HTTP methods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
}

/// Proxy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// Proxy URL (e.g., socks5://user:pass@host:port)
    pub url: String,
    /// Proxy protocol
    pub protocol: ProxyProtocol,
    /// Username for authentication
    pub username: Option<String>,
    /// Password for authentication
    pub password: Option<String>,
    /// Session ID for sticky sessions
    pub session_id: Option<String>,
}

impl ProxyConfig {
    /// Creates a new ProxyConfig from a URL string
    pub fn from_url(url: impl Into<String>) -> Result<Self, crate::GhostError> {
        // TODO: Implement ProxyConfig parsing from URL
        Ok(Self {
            url: url.into(),
            protocol: ProxyProtocol::Http,
            username: None,
            password: None,
            session_id: None,
        })
    }

    /// Parses proxy URL and extracts components
    pub fn parse_url(&self) -> Result<(String, u16), crate::GhostError> {
        // TODO: Implement URL parsing to extract host and port
        Err(crate::GhostError::NotImplemented("parse_url".into()))
    }
}

/// Proxy protocol types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProxyProtocol {
    Http,
    Https,
    Socks5,
    Socks4,
}

/// Session/credential data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    /// Session type
    pub session_type: SessionType,
    /// Raw cookie string
    pub cookies: Option<String>,
    /// Bearer token
    pub bearer_token: Option<String>,
    /// Platform-specific auth tokens
    pub auth_tokens: std::collections::HashMap<String, String>,
    /// Session ID
    pub session_id: Option<String>,
}

impl SessionData {
    /// Creates a new SessionData from a cookie string
    pub fn from_cookies(cookies: impl Into<String>) -> Self {
        // TODO: Implement SessionData construction from cookies
        Self {
            session_type: SessionType::Cookies,
            cookies: Some(cookies.into()),
            bearer_token: None,
            auth_tokens: std::collections::HashMap::new(),
            session_id: None,
        }
    }

    /// Creates a new SessionData from a bearer token
    pub fn from_bearer(token: impl Into<String>) -> Self {
        // TODO: Implement SessionData construction from bearer token
        Self {
            session_type: SessionType::Bearer,
            cookies: None,
            bearer_token: Some(token.into()),
            auth_tokens: std::collections::HashMap::new(),
            session_id: None,
        }
    }

    /// Validates the session data
    pub fn validate(&self) -> Result<(), crate::GhostError> {
        // TODO: Implement session validation
        Ok(())
    }
}

/// Type of session authentication
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionType {
    Cookies,
    Bearer,
    ApiKey,
    Oauth2,
    Guest,
}

/// GhostContext for multi-tenant request configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhostContext {
    /// Tenant identifier
    pub tenant_id: Option<String>,
    /// Proxy configuration
    pub proxy: Option<ProxyConfig>,
    /// Session/credential data
    pub session: Option<SessionData>,
    /// Strategy for routing
    pub strategy: Strategy,
    /// Budget limits
    pub budget: Option<BudgetLimits>,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl GhostContext {
    /// Creates a new GhostContext builder
    pub fn builder() -> GhostContextBuilder {
        // TODO: Implement builder pattern for GhostContext
        GhostContextBuilder::default()
    }

    /// Validates the context
    pub fn validate(&self) -> Result<(), crate::GhostError> {
        // TODO: Implement context validation
        Ok(())
    }
}

impl Default for GhostContext {
    fn default() -> Self {
        Self {
            tenant_id: None,
            proxy: None,
            session: None,
            strategy: Strategy::default(),
            budget: None,
            metadata: std::collections::HashMap::new(),
        }
    }
}

/// Builder for GhostContext
#[derive(Debug, Default)]
pub struct GhostContextBuilder {
    tenant_id: Option<String>,
    proxy: Option<ProxyConfig>,
    session: Option<SessionData>,
    strategy: Strategy,
    budget: Option<BudgetLimits>,
    metadata: std::collections::HashMap<String, String>,
}

impl GhostContextBuilder {
    /// Sets the tenant ID
    pub fn tenant_id(mut self, id: impl Into<String>) -> Self {
        // TODO: Implement tenant_id setter with validation
        self.tenant_id = Some(id.into());
        self
    }

    /// Sets the proxy from a URL string
    pub fn proxy(mut self, url: impl Into<String>) -> Self {
        // TODO: Implement proxy setter with URL parsing
        self.proxy = ProxyConfig::from_url(url).ok();
        self
    }

    /// Sets the proxy configuration directly
    pub fn proxy_config(mut self, proxy: ProxyConfig) -> Self {
        // TODO: Implement proxy config setter
        self.proxy = Some(proxy);
        self
    }

    /// Sets the session from a cookie string
    pub fn session(mut self, cookies: impl Into<String>) -> Self {
        // TODO: Implement session setter with cookie parsing
        self.session = Some(SessionData::from_cookies(cookies));
        self
    }

    /// Sets the session data directly
    pub fn session_data(mut self, session: SessionData) -> Self {
        // TODO: Implement session data setter
        self.session = Some(session);
        self
    }

    /// Sets the routing strategy
    pub fn strategy(mut self, strategy: Strategy) -> Self {
        // TODO: Implement strategy setter
        self.strategy = strategy;
        self
    }

    /// Sets the budget limits
    pub fn budget(mut self, limits: BudgetLimits) -> Self {
        // TODO: Implement budget setter
        self.budget = Some(limits);
        self
    }

    /// Adds metadata key-value pair
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        // TODO: Implement metadata setter
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Builds the GhostContext
    pub fn build(self) -> GhostContext {
        // TODO: Implement GhostContext construction with validation
        GhostContext {
            tenant_id: self.tenant_id,
            proxy: self.proxy,
            session: self.session,
            strategy: self.strategy,
            budget: self.budget,
            metadata: self.metadata,
        }
    }
}

/// Routing strategy for request handling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Strategy {
    /// Route based on health scores (prefer healthy workers)
    #[default]
    HealthFirst,
    /// Route to fastest responding worker
    Fastest,
    /// Route to least expensive option
    CostOptimized,
    /// Try official API first, fallback to scrapers
    OfficialFirst,
    /// Only use official API
    OfficialOnly,
    /// Only use scrapers, never official API
    ScrapersOnly,
    /// Round-robin distribution
    RoundRobin,
}

impl Strategy {
    /// Returns the fallback strategy for this strategy
    pub fn fallback(&self) -> Option<Strategy> {
        // TODO: Implement fallback strategy determination
        match self {
            Strategy::HealthFirst => Some(Strategy::OfficialFirst),
            Strategy::Fastest => Some(Strategy::HealthFirst),
            Strategy::CostOptimized => Some(Strategy::HealthFirst),
            Strategy::OfficialFirst => None,
            Strategy::OfficialOnly => None,
            Strategy::ScrapersOnly => None,
            Strategy::RoundRobin => Some(Strategy::HealthFirst),
        }
    }
}

/// Budget limits for tenant requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetLimits {
    /// Maximum requests per hour
    pub max_requests_per_hour: u32,
    /// Maximum estimated cost per day (USD)
    pub max_cost_per_day: f64,
    /// Alert at this percentage of budget usage
    pub alert_at_percent: u8,
}

impl BudgetLimits {
    /// Creates a new BudgetLimits with specified values
    pub fn new(max_requests_per_hour: u32, max_cost_per_day: f64, alert_at_percent: u8) -> Self {
        // TODO: Implement BudgetLimits construction with validation
        Self {
            max_requests_per_hour,
            max_cost_per_day,
            alert_at_percent,
        }
    }

    /// Validates the budget limits
    pub fn validate(&self) -> Result<(), crate::GhostError> {
        // TODO: Implement budget limits validation
        Ok(())
    }
}

impl Default for BudgetLimits {
    fn default() -> Self {
        Self {
            max_requests_per_hour: 1000,
            max_cost_per_day: 50.0,
            alert_at_percent: 80,
        }
    }
}
