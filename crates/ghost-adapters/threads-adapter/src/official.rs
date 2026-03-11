//! Official Threads API Client (Tier 3 Fallback)
//!
//! This module implements the official Meta Threads API as the last-resort fallback.
//! Based on Meta Threads API documentation: https://developers.facebook.com/docs/threads
//!
//! The API follows a two-step container-based publishing workflow:
//! 1. Create a media container
//! 2. Publish the container

use std::time::Duration;

use chrono::{DateTime, Utc};
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

use ghost_schema::{
    AdapterError, GhostError, GhostMedia, GhostPost, GhostUser,
    MediaType, Platform, ThreadsAuth, ThreadsPostType,
};

// ============================================================================
// Constants
// ============================================================================

/// Base URL for the Threads Graph API
pub const THREADS_API_BASE_URL: &str = "https://graph.threads.net";

/// Default API version
pub const DEFAULT_API_VERSION: &str = "v1.0";

/// OAuth authorization URL
pub const THREADS_OAUTH_URL: &str = "https://threads.net/oauth/authorize";

/// OAuth token URL
pub const THREADS_TOKEN_URL: &str = "https://graph.threads.net/oauth/access_token";

/// Default timeout for API requests (seconds)
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Maximum posts per 24-hour period (rate limit)
pub const MAX_POSTS_PER_DAY: u32 = 250;

/// Maximum text length per post
pub const MAX_TEXT_LENGTH: usize = 500;

/// Maximum carousel items
pub const MAX_CAROUSEL_ITEMS: usize = 20;

/// Maximum media file size (images: 8MB, videos: 1GB)
pub const MAX_IMAGE_SIZE_BYTES: u64 = 8 * 1024 * 1024; // 8MB
pub const MAX_VIDEO_SIZE_BYTES: u64 = 1024 * 1024 * 1024; // 1GB

/// Media container expiration time (24 hours)
pub const CONTAINER_EXPIRATION_SECS: u64 = 24 * 60 * 60;

// ============================================================================
// Permission Scopes
// ============================================================================

/// Threads API permission scopes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThreadsScope {
    /// Basic access to read profile data, posts, and basic interactions
    ThreadsBasic,
    /// Permission to publish posts
    ThreadsContentPublish,
    /// Permission to delete posts
    ThreadsDelete,
    /// Access to analytics and insights data
    ThreadsReadInsights,
    /// Permission to manage replies
    ThreadsManageReplies,
    /// Permission to use keyword search
    ThreadsSearch,
}

impl ThreadsScope {
    /// Returns the API name for this scope
    pub fn as_str(&self) -> &'static str {
        match self {
            ThreadsScope::ThreadsBasic => "threads_basic",
            ThreadsScope::ThreadsContentPublish => "threads_content_publish",
            ThreadsScope::ThreadsDelete => "threads_delete",
            ThreadsScope::ThreadsReadInsights => "threads_read_insights",
            ThreadsScope::ThreadsManageReplies => "threads_manage_replies",
            ThreadsScope::ThreadsSearch => "threads_search",
        }
    }
}

impl std::fmt::Display for ThreadsScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ============================================================================
// API Response Types
// ============================================================================

/// Generic API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// Response data
    pub data: Option<T>,
    /// Pagination info
    pub paging: Option<Paging>,
    /// Error if present
    pub error: Option<ApiError>,
}

/// Pagination information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paging {
    /// Cursors for pagination
    pub cursors: Option<Cursors>,
    /// Next page URL
    pub next: Option<String>,
    /// Previous page URL
    pub previous: Option<String>,
}

/// Cursor tokens for bidirectional pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cursors {
    /// Cursor for results before this point
    pub before: Option<String>,
    /// Cursor for results after this point
    pub after: Option<String>,
}

/// API error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    /// Error message
    pub message: String,
    /// Error type
    #[serde(rename = "type")]
    pub error_type: String,
    /// Error code
    pub code: i32,
    /// Error subcode
    pub error_subcode: Option<i32>,
    /// User-friendly error message
    pub error_user_msg: Option<String>,
    /// Trace ID for debugging
    pub fbtrace_id: Option<String>,
}

// ============================================================================
// Media Container Types
// ============================================================================

/// Media container for creating posts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaContainer {
    /// Container ID
    pub id: String,
}

/// Media container status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerStatus {
    /// Container ID
    pub id: String,
    /// Status: IN_PROGRESS, PUBLISHED, ERROR, EXPIRED
    pub status: ContainerStatusType,
    /// Error message if status is ERROR
    pub error_message: Option<String>,
}

/// Container status type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ContainerStatusType {
    /// Media is being processed
    InProgress,
    /// Container has been published
    Published,
    /// Processing failed
    Error,
    /// Container expired (not published within 24 hours)
    Expired,
}

// ============================================================================
// Threads Post Response
// ============================================================================

/// Threads media object from API
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThreadsMedia {
    /// Media ID
    pub id: String,
    /// Media product type (THREADS, THREADS_REEL)
    pub media_product_type: Option<String>,
    /// Media type (TEXT, IMAGE, VIDEO, CAROUSEL)
    pub media_type: Option<String>,
    /// Text content
    pub text: Option<String>,
    /// Short URL identifier
    pub shortcode: Option<String>,
    /// Creation timestamp
    pub timestamp: Option<String>,
    /// Permanent URL
    pub permalink: Option<String>,
    /// Owner information
    pub owner: Option<MediaOwner>,
    /// Media URL
    pub media_url: Option<String>,
    /// Thumbnail URL for videos
    pub thumbnail_url: Option<String>,
    /// Is this a reply
    pub is_reply: Option<bool>,
    /// Is this a quote post
    pub is_quote_post: Option<bool>,
    /// Quoted post details
    pub quoted_post: Option<Box<ThreadsMedia>>,
    /// Hide status
    pub hide_status: Option<String>,
    /// Reply audience
    pub reply_audience: Option<String>,
    /// Has audio (for videos)
    pub has_audio: Option<bool>,
    /// Children for carousel
    pub children: Option<ChildrenData>,
    /// Like count
    #[serde(default)]
    pub likes_count: u64,
    /// Quotes count
    #[serde(default)]
    pub quotes_count: u64,
    /// Reposts count
    #[serde(default)]
    pub reposts_count: u64,
    /// Replies count
    #[serde(default)]
    pub replies_count: u64,
}

/// Media owner information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaOwner {
    /// User ID
    pub id: String,
    /// Username
    pub username: Option<String>,
}

/// Children data for carousel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildrenData {
    /// List of child media
    pub data: Vec<ThreadsMedia>,
}

// ============================================================================
// Threads User Response
// ============================================================================

/// Threads user profile from API
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThreadsUser {
    /// User ID
    pub id: String,
    /// Username
    pub username: String,
    /// Display name
    pub name: Option<String>,
    /// Profile picture URL
    pub threads_profile_picture_url: Option<String>,
    /// Biography
    pub threads_biography: Option<String>,
    /// Profile URL
    pub profile_url: Option<String>,
    /// Verified status
    #[serde(default)]
    pub is_verified: bool,
    /// Followers count
    pub followers_count: Option<u64>,
    /// Following count
    pub following_count: Option<u64>,
    /// Media count
    pub media_count: Option<u64>,
}

// ============================================================================
// Insights Response
// ============================================================================

/// Insights response from API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightsResponse {
    /// List of insight metrics
    pub data: Vec<InsightMetric>,
}

/// Single insight metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightMetric {
    /// Metric name
    pub name: String,
    /// Time period
    pub period: String,
    /// Metric values
    pub values: Vec<InsightValue>,
    /// Title
    pub title: Option<String>,
    /// Description
    pub description: Option<String>,
}

/// Insight value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightValue {
    /// The value
    pub value: serde_json::Value,
    /// End time for the period
    pub end_time: Option<String>,
}

// ============================================================================
// OAuth Token Response
// ============================================================================

/// OAuth token response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    /// Access token
    pub access_token: String,
    /// Token type (usually "bearer")
    pub token_type: Option<String>,
    /// Expires in seconds
    pub expires_in: Option<u64>,
}

/// Long-lived token response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LongLivedTokenResponse {
    /// Access token
    pub access_token: String,
    /// Token type
    pub token_type: Option<String>,
    /// Expires in seconds
    pub expires_in: Option<u64>,
    /// Refresh token for getting new tokens
    pub refresh_token: Option<String>,
}

// ============================================================================
// Official Threads API Client
// ============================================================================

/// Official Threads API client
///
/// This client implements the official Meta Threads API for use as
/// the Tier 3 fallback when all scrapers fail.
pub struct ThreadsOfficialClient {
    /// HTTP client
    client: Client,
    /// App ID (client ID)
    app_id: Option<String>,
    /// App secret
    app_secret: Option<String>,
    /// Redirect URI for OAuth
    redirect_uri: Option<String>,
    /// API version
    api_version: String,
    /// Base URL
    base_url: String,
}

impl ThreadsOfficialClient {
    /// Creates a new official API client
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
                .user_agent("ghost-api-threads-adapter/0.1.0")
                .build()
                .unwrap_or_else(|_| Client::new()),
            app_id: None,
            app_secret: None,
            redirect_uri: None,
            api_version: DEFAULT_API_VERSION.to_string(),
            base_url: THREADS_API_BASE_URL.to_string(),
        }
    }

    /// Creates a client with app credentials
    pub fn with_credentials(
        app_id: impl Into<String>,
        app_secret: impl Into<String>,
        redirect_uri: impl Into<String>,
    ) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
                .user_agent("ghost-api-threads-adapter/0.1.0")
                .build()
                .unwrap_or_else(|_| Client::new()),
            app_id: Some(app_id.into()),
            app_secret: Some(app_secret.into()),
            redirect_uri: Some(redirect_uri.into()),
            api_version: DEFAULT_API_VERSION.to_string(),
            base_url: THREADS_API_BASE_URL.to_string(),
        }
    }

    /// Sets the app credentials
    pub fn set_credentials(
        &mut self,
        app_id: impl Into<String>,
        app_secret: impl Into<String>,
        redirect_uri: impl Into<String>,
    ) {
        self.app_id = Some(app_id.into());
        self.app_secret = Some(app_secret.into());
        self.redirect_uri = Some(redirect_uri.into());
    }

    /// Sets a custom timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.client = Client::builder()
            .timeout(timeout)
            .user_agent("ghost-api-threads-adapter/0.1.0")
            .build()
            .unwrap_or_else(|_| self.client.clone());
        self
    }

    /// Sets the API version
    pub fn with_api_version(mut self, version: impl Into<String>) -> Self {
        self.api_version = version.into();
        self
    }

    // ========================================================================
    // OAuth Methods
    // ========================================================================

    /// Generates the OAuth authorization URL
    ///
    /// Redirect users to this URL to request permissions
    pub fn get_authorization_url(
        &self,
        scopes: &[ThreadsScope],
        state: Option<&str>,
    ) -> Result<String, GhostError> {
        let app_id = self.app_id.as_ref()
            .ok_or_else(|| GhostError::AuthError("App ID not configured".into()))?;
        let redirect_uri = self.redirect_uri.as_ref()
            .ok_or_else(|| GhostError::AuthError("Redirect URI not configured".into()))?;

        let scope_str = scopes.iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(",");

        let mut url = format!(
            "{}?client_id={}&redirect_uri={}&scope={}&response_type=code",
            THREADS_OAUTH_URL,
            urlencoding::encode(app_id),
            urlencoding::encode(redirect_uri),
            urlencoding::encode(&scope_str),
        );

        if let Some(state) = state {
            url.push_str(&format!("&state={}", urlencoding::encode(state)));
        }

        Ok(url)
    }

    /// Exchanges an authorization code for an access token
    pub async fn exchange_code_for_token(
        &self,
        code: &str,
    ) -> Result<TokenResponse, GhostError> {
        let app_id = self.app_id.as_ref()
            .ok_or_else(|| GhostError::AuthError("App ID not configured".into()))?;
        let app_secret = self.app_secret.as_ref()
            .ok_or_else(|| GhostError::AuthError("App secret not configured".into()))?;
        let redirect_uri = self.redirect_uri.as_ref()
            .ok_or_else(|| GhostError::AuthError("Redirect URI not configured".into()))?;

        let params = [
            ("client_id", app_id.as_str()),
            ("client_secret", app_secret.as_str()),
            ("redirect_uri", redirect_uri.as_str()),
            ("code", code),
        ];

        let response = self.client
            .post(THREADS_TOKEN_URL)
            .form(&params)
            .send()
            .await
            .map_err(|e| GhostError::NetworkError(format!("Token exchange failed: {}", e)))?;

        self.handle_response(response).await
    }

    /// Exchanges a short-lived token for a long-lived token
    pub async fn get_long_lived_token(
        &self,
        short_lived_token: &str,
    ) -> Result<LongLivedTokenResponse, GhostError> {
        let app_secret = self.app_secret.as_ref()
            .ok_or_else(|| GhostError::AuthError("App secret not configured".into()))?;

        let url = format!(
            "{}/access_token?grant_type=th_exchange_token&client_secret={}&access_token={}",
            self.base_url,
            urlencoding::encode(app_secret),
            urlencoding::encode(short_lived_token),
        );

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| GhostError::NetworkError(format!("Long-lived token request failed: {}", e)))?;

        self.handle_response(response).await
    }

    /// Refreshes a long-lived token
    pub async fn refresh_token(
        &self,
        token: &str,
    ) -> Result<LongLivedTokenResponse, GhostError> {
        let url = format!(
            "{}/refresh_access_token?grant_type=th_refresh_token&access_token={}",
            self.base_url,
            urlencoding::encode(token),
        );

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| GhostError::NetworkError(format!("Token refresh failed: {}", e)))?;

        self.handle_response(response).await
    }

    // ========================================================================
    // User Methods
    // ========================================================================

    /// Gets a user's profile
    ///
    /// Endpoint: GET /{user-id}
    pub async fn get_user(
        &self,
        user_id: &str,
        access_token: &str,
        fields: Option<&[&str]>,
    ) -> Result<ThreadsUser, GhostError> {
        let default_fields = [
            "id",
            "username",
            "name",
            "threads_profile_picture_url",
            "threads_biography",
            "followers_count",
            "following_count",
            "media_count",
            "is_verified",
        ];
        let fields_str = fields
            .map(|f| f.join(","))
            .unwrap_or_else(|| default_fields.join(","));

        let url = format!(
            "{}/{}/{}?fields={}&access_token={}",
            self.base_url,
            self.api_version,
            user_id,
            urlencoding::encode(&fields_str),
            urlencoding::encode(access_token),
        );

        debug!("Getting user profile: {}", user_id);

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| GhostError::NetworkError(format!("Get user failed: {}", e)))?;

        self.handle_response(response).await
    }

    /// Gets the current authenticated user's profile
    ///
    /// Endpoint: GET /me
    pub async fn get_me(
        &self,
        access_token: &str,
        fields: Option<&[&str]>,
    ) -> Result<ThreadsUser, GhostError> {
        self.get_user("me", access_token, fields).await
    }

    // ========================================================================
    // Post Retrieval Methods
    // ========================================================================

    /// Gets a single post by ID
    ///
    /// Endpoint: GET /{media-id}
    pub async fn get_post(
        &self,
        media_id: &str,
        access_token: &str,
        fields: Option<&[&str]>,
    ) -> Result<ThreadsMedia, GhostError> {
        let default_fields = [
            "id",
            "media_type",
            "media_url",
            "text",
            "timestamp",
            "permalink",
            "owner",
            "is_reply",
            "is_quote_post",
            "likes_count",
            "quotes_count",
            "reposts_count",
            "replies_count",
        ];
        let fields_str = fields
            .map(|f| f.join(","))
            .unwrap_or_else(|| default_fields.join(","));

        let url = format!(
            "{}/{}/{}?fields={}&access_token={}",
            self.base_url,
            self.api_version,
            media_id,
            urlencoding::encode(&fields_str),
            urlencoding::encode(access_token),
        );

        debug!("Getting post: {}", media_id);

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| GhostError::NetworkError(format!("Get post failed: {}", e)))?;

        self.handle_response(response).await
    }

    /// Gets a user's posts
    ///
    /// Endpoint: GET /{user-id}/threads
    pub async fn get_user_posts(
        &self,
        user_id: &str,
        access_token: &str,
        fields: Option<&[&str]>,
        limit: Option<u32>,
        after: Option<&str>,
        before: Option<&str>,
    ) -> Result<ApiResponse<Vec<ThreadsMedia>>, GhostError> {
        let default_fields = [
            "id",
            "media_type",
            "media_url",
            "text",
            "timestamp",
            "permalink",
            "owner",
            "likes_count",
            "replies_count",
        ];
        let fields_str = fields
            .map(|f| f.join(","))
            .unwrap_or_else(|| default_fields.join(","));

        let mut url = format!(
            "{}/{}/{}/threads?fields={}&access_token={}",
            self.base_url,
            self.api_version,
            user_id,
            urlencoding::encode(&fields_str),
            urlencoding::encode(access_token),
        );

        if let Some(limit) = limit {
            url.push_str(&format!("&limit={}", limit));
        }
        if let Some(after) = after {
            url.push_str(&format!("&after={}", urlencoding::encode(after)));
        }
        if let Some(before) = before {
            url.push_str(&format!("&before={}", urlencoding::encode(before)));
        }

        debug!("Getting user posts: {}", user_id);

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| GhostError::NetworkError(format!("Get user posts failed: {}", e)))?;

        self.handle_response(response).await
    }

    /// Gets posts where the user is mentioned
    ///
    /// Endpoint: GET /{user-id}/mentions
    pub async fn get_mentions(
        &self,
        user_id: &str,
        access_token: &str,
        fields: Option<&[&str]>,
        limit: Option<u32>,
    ) -> Result<ApiResponse<Vec<ThreadsMedia>>, GhostError> {
        let default_fields = [
            "id",
            "media_type",
            "text",
            "timestamp",
            "owner",
        ];
        let fields_str = fields
            .map(|f| f.join(","))
            .unwrap_or_else(|| default_fields.join(","));

        let mut url = format!(
            "{}/{}/{}/mentions?fields={}&access_token={}",
            self.base_url,
            self.api_version,
            user_id,
            urlencoding::encode(&fields_str),
            urlencoding::encode(access_token),
        );

        if let Some(limit) = limit {
            url.push_str(&format!("&limit={}", limit));
        }

        debug!("Getting mentions for user: {}", user_id);

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| GhostError::NetworkError(format!("Get mentions failed: {}", e)))?;

        self.handle_response(response).await
    }

    // ========================================================================
    // Search Methods
    // ========================================================================

    /// Searches for posts by keyword
    ///
    /// Endpoint: GET /threads/search
    /// Note: Requires threads_search permission
    pub async fn search_posts(
        &self,
        query: &str,
        access_token: &str,
        fields: Option<&[&str]>,
        limit: Option<u32>,
    ) -> Result<ApiResponse<Vec<ThreadsMedia>>, GhostError> {
        let default_fields = [
            "id",
            "media_type",
            "text",
            "timestamp",
            "owner",
            "permalink",
        ];
        let fields_str = fields
            .map(|f| f.join(","))
            .unwrap_or_else(|| default_fields.join(","));

        let mut url = format!(
            "{}/{}/threads/search?q={}&fields={}&access_token={}",
            self.base_url,
            self.api_version,
            urlencoding::encode(query),
            urlencoding::encode(&fields_str),
            urlencoding::encode(access_token),
        );

        if let Some(limit) = limit {
            url.push_str(&format!("&limit={}", limit));
        }

        debug!("Searching posts: {}", query);

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| GhostError::NetworkError(format!("Search failed: {}", e)))?;

        self.handle_response(response).await
    }

    // ========================================================================
    // Publishing Methods
    // ========================================================================

    /// Creates a media container for publishing
    ///
    /// Step 1 of the two-step publishing workflow
    pub async fn create_media_container(
        &self,
        user_id: &str,
        access_token: &str,
        request: CreateMediaRequest,
    ) -> Result<MediaContainer, GhostError> {
        let url = format!(
            "{}/{}/{}/threads",
            self.base_url,
            self.api_version,
            user_id,
        );

        debug!("Creating media container for user: {}", user_id);

        let response = self.client
            .post(&url)
            .query(&[("access_token", access_token)])
            .json(&request)
            .send()
            .await
            .map_err(|e| GhostError::NetworkError(format!("Create container failed: {}", e)))?;

        self.handle_response(response).await
    }

    /// Publishes a media container
    ///
    /// Step 2 of the two-step publishing workflow
    pub async fn publish_container(
        &self,
        user_id: &str,
        access_token: &str,
        creation_id: &str,
    ) -> Result<MediaContainer, GhostError> {
        let url = format!(
            "{}/{}/{}/threads_publish",
            self.base_url,
            self.api_version,
            user_id,
        );

        debug!("Publishing container: {}", creation_id);

        let response = self.client
            .post(&url)
            .query(&[("access_token", access_token)])
            .query(&[("creation_id", creation_id)])
            .send()
            .await
            .map_err(|e| GhostError::NetworkError(format!("Publish failed: {}", e)))?;

        self.handle_response(response).await
    }

    /// Checks the status of a media container
    pub async fn get_container_status(
        &self,
        container_id: &str,
        access_token: &str,
    ) -> Result<ContainerStatus, GhostError> {
        let url = format!(
            "{}/{}/{}?fields=id,status,error_message&access_token={}",
            self.base_url,
            self.api_version,
            container_id,
            urlencoding::encode(access_token),
        );

        debug!("Checking container status: {}", container_id);

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| GhostError::NetworkError(format!("Status check failed: {}", e)))?;

        self.handle_response(response).await
    }

    /// Publishes a text post (convenience method)
    ///
    /// This handles the two-step workflow internally
    pub async fn publish_text_post(
        &self,
        user_id: &str,
        access_token: &str,
        text: &str,
    ) -> Result<String, GhostError> {
        // Validate text length
        if text.len() > MAX_TEXT_LENGTH {
            return Err(GhostError::ValidationError(
                format!("Text exceeds maximum length of {} characters", MAX_TEXT_LENGTH)
            ));
        }

        // Step 1: Create container
        let request = CreateMediaRequest {
            media_type: "TEXT".to_string(),
            text: Some(text.to_string()),
            ..Default::default()
        };

        let container = self.create_media_container(user_id, access_token, request).await?;
        info!("Created media container: {}", container.id);

        // Step 2: Publish
        let published = self.publish_container(user_id, access_token, &container.id).await?;
        info!("Published post: {}", published.id);

        Ok(published.id)
    }

    /// Publishes an image post (convenience method)
    pub async fn publish_image_post(
        &self,
        user_id: &str,
        access_token: &str,
        image_url: &str,
        text: Option<&str>,
    ) -> Result<String, GhostError> {
        let request = CreateMediaRequest {
            media_type: "IMAGE".to_string(),
            image_url: Some(image_url.to_string()),
            text: text.map(|t| t.to_string()),
            ..Default::default()
        };

        let container = self.create_media_container(user_id, access_token, request).await?;
        let published = self.publish_container(user_id, access_token, &container.id).await?;

        Ok(published.id)
    }

    /// Publishes a video post (convenience method)
    pub async fn publish_video_post(
        &self,
        user_id: &str,
        access_token: &str,
        video_url: &str,
        text: Option<&str>,
    ) -> Result<String, GhostError> {
        let request = CreateMediaRequest {
            media_type: "VIDEO".to_string(),
            video_url: Some(video_url.to_string()),
            text: text.map(|t| t.to_string()),
            ..Default::default()
        };

        let container = self.create_media_container(user_id, access_token, request).await?;

        // Wait for video processing
        let status = self.wait_for_processing(user_id, access_token, &container.id).await?;

        if status.status != ContainerStatusType::Published {
            return Err(GhostError::AdapterError(
                format!("Video processing failed: {:?}", status.error_message)
            ));
        }

        Ok(container.id)
    }

    /// Publishes a carousel post
    pub async fn publish_carousel_post(
        &self,
        user_id: &str,
        access_token: &str,
        children: &[&str], // Container IDs of child media
        text: Option<&str>,
    ) -> Result<String, GhostError> {
        if children.len() < 2 || children.len() > MAX_CAROUSEL_ITEMS {
            return Err(GhostError::ValidationError(
                format!("Carousel must have 2-{} items", MAX_CAROUSEL_ITEMS)
            ));
        }

        let request = CreateMediaRequest {
            media_type: "CAROUSEL".to_string(),
            children: Some(children.iter().map(|s| s.to_string()).collect()),
            text: text.map(|t| t.to_string()),
            ..Default::default()
        };

        let container = self.create_media_container(user_id, access_token, request).await?;
        let published = self.publish_container(user_id, access_token, &container.id).await?;

        Ok(published.id)
    }

    /// Waits for media processing to complete
    async fn wait_for_processing(
        &self,
        user_id: &str,
        access_token: &str,
        container_id: &str,
    ) -> Result<ContainerStatus, GhostError> {
        let max_attempts = 60; // 5 minutes max (60 * 5 seconds)
        let mut attempts = 0;

        loop {
            let status = self.get_container_status(container_id, access_token).await?;

            match status.status {
                ContainerStatusType::Published => {
                    info!("Media processing complete: {}", container_id);
                    return Ok(status);
                }
                ContainerStatusType::Error => {
                    error!("Media processing failed: {:?}", status.error_message);
                    return Err(GhostError::AdapterError(
                        status.error_message.unwrap_or_else(|| "Processing failed".into())
                    ));
                }
                ContainerStatusType::Expired => {
                    return Err(GhostError::AdapterError("Media container expired".into()));
                }
                ContainerStatusType::InProgress => {
                    attempts += 1;
                    if attempts >= max_attempts {
                        return Err(GhostError::Timeout(
                            "Media processing timed out".into()
                        ));
                    }
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }

    // ========================================================================
    // Delete Methods
    // ========================================================================

    /// Deletes a post
    ///
    /// Endpoint: DELETE /{media-id}
    pub async fn delete_post(
        &self,
        media_id: &str,
        access_token: &str,
    ) -> Result<bool, GhostError> {
        let url = format!(
            "{}/{}/{}?access_token={}",
            self.base_url,
            self.api_version,
            media_id,
            urlencoding::encode(access_token),
        );

        debug!("Deleting post: {}", media_id);

        let response = self.client
            .delete(&url)
            .send()
            .await
            .map_err(|e| GhostError::NetworkError(format!("Delete failed: {}", e)))?;

        let result: serde_json::Value = self.handle_response(response).await?;
        Ok(result.get("success").and_then(|v| v.as_bool()).unwrap_or(true))
    }

    // ========================================================================
    // Insights Methods
    // ========================================================================

    /// Gets insights for a post
    ///
    /// Endpoint: GET /{media-id}/insights
    pub async fn get_post_insights(
        &self,
        media_id: &str,
        access_token: &str,
        metrics: Option<&[&str]>,
    ) -> Result<InsightsResponse, GhostError> {
        let default_metrics = ["views", "likes", "replies", "reposts", "quotes"];
        let metrics_str = metrics
            .map(|m| m.join(","))
            .unwrap_or_else(|| default_metrics.join(","));

        let url = format!(
            "{}/{}/{}/insights?metric={}&access_token={}",
            self.base_url,
            self.api_version,
            media_id,
            urlencoding::encode(&metrics_str),
            urlencoding::encode(access_token),
        );

        debug!("Getting post insights: {}", media_id);

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| GhostError::NetworkError(format!("Get insights failed: {}", e)))?;

        self.handle_response(response).await
    }

    /// Gets insights for a user
    ///
    /// Endpoint: GET /{user-id}/insights
    pub async fn get_user_insights(
        &self,
        user_id: &str,
        access_token: &str,
        metrics: Option<&[&str]>,
    ) -> Result<InsightsResponse, GhostError> {
        let default_metrics = ["followers_count", "profile_views"];
        let metrics_str = metrics
            .map(|m| m.join(","))
            .unwrap_or_else(|| default_metrics.join(","));

        let url = format!(
            "{}/{}/{}/insights?metric={}&access_token={}",
            self.base_url,
            self.api_version,
            user_id,
            urlencoding::encode(&metrics_str),
            urlencoding::encode(access_token),
        );

        debug!("Getting user insights: {}", user_id);

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| GhostError::NetworkError(format!("Get user insights failed: {}", e)))?;

        self.handle_response(response).await
    }

    // ========================================================================
    // Reply Management Methods
    // ========================================================================

    /// Hides or unhides a reply
    ///
    /// Endpoint: POST /{media-id}/replies/{reply-id}/hide
    pub async fn hide_reply(
        &self,
        media_id: &str,
        reply_id: &str,
        access_token: &str,
        hide: bool,
    ) -> Result<bool, GhostError> {
        let url = format!(
            "{}/{}/{}/replies/{}/hide",
            self.base_url,
            self.api_version,
            media_id,
            reply_id,
        );

        let response = self.client
            .post(&url)
            .query(&[("access_token", access_token)])
            .query(&[("hide", hide.to_string())])
            .send()
            .await
            .map_err(|e| GhostError::NetworkError(format!("Hide reply failed: {}", e)))?;

        let result: serde_json::Value = self.handle_response(response).await?;
        Ok(result.get("success").and_then(|v| v.as_bool()).unwrap_or(true))
    }

    /// Sets reply controls for a post
    ///
    /// Endpoint: POST /{media-id}/reply_controls
    pub async fn set_reply_controls(
        &self,
        media_id: &str,
        access_token: &str,
        reply_control: ReplyControlType,
    ) -> Result<bool, GhostError> {
        let url = format!(
            "{}/{}/{}/reply_controls",
            self.base_url,
            self.api_version,
            media_id,
        );

        let response = self.client
            .post(&url)
            .query(&[("access_token", access_token)])
            .json(&serde_json::json!({ "reply_control": reply_control.as_str() }))
            .send()
            .await
            .map_err(|e| GhostError::NetworkError(format!("Set reply controls failed: {}", e)))?;

        let result: serde_json::Value = self.handle_response(response).await?;
        Ok(result.get("success").and_then(|v| v.as_bool()).unwrap_or(true))
    }

    // ========================================================================
    // Helper Methods
    // ========================================================================

    /// Handles API response and extracts data or error
    async fn handle_response<T: for<'de> Deserialize<'de>>(
        &self,
        response: Response,
    ) -> Result<T, GhostError> {
        let status = response.status();
        let body = response.text().await
            .map_err(|e| GhostError::NetworkError(format!("Failed to read response: {}", e)))?;

        debug!("API response status: {}", status);
        debug!("API response body: {}", body);

        // Check for HTTP errors
        if !status.is_success() {
            // Try to parse as API error
            if let Ok(api_error) = serde_json::from_str::<ApiResponse<T>>(&body) {
                if let Some(error) = api_error.error {
                    return Err(self.map_api_error(error));
                }
            }

            return Err(GhostError::NetworkError(
                format!("HTTP error {}: {}", status, body)
            ));
        }

        // Parse the response
        serde_json::from_str(&body)
            .map_err(|e| GhostError::ParseError(format!("Failed to parse response: {} - Body: {}", e, body)))
    }

    /// Maps API error to GhostError
    fn map_api_error(&self, error: ApiError) -> GhostError {
        match error.code {
            4 | 17 => GhostError::RateLimited {
                retry_after: None,
                platform: Platform::Threads,
            },
            10 | 200 => GhostError::AuthError(error.message),
            190 => GhostError::SessionExpired(error.message),
            100 => GhostError::ValidationError(error.message),
            _ => GhostError::PlatformError {
                code: error.code as u16,
                message: error.message,
                platform: Platform::Threads,
            },
        }
    }
}

impl Default for ThreadsOfficialClient {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Request/Response Types
// ============================================================================

/// Request to create a media container
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CreateMediaRequest {
    /// Media type: TEXT, IMAGE, VIDEO, CAROUSEL
    pub media_type: String,
    /// Text content (max 500 characters)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Image URL (for IMAGE type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    /// Video URL (for VIDEO type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_url: Option<String>,
    /// Children container IDs (for CAROUSEL type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<String>>,
    /// Whether this is a reply
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to_id: Option<String>,
    /// Whether this is a quote post
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quoted_post_id: Option<String>,
    /// Reply audience
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_audience: Option<String>,
    /// Whether to share to Instagram feed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub share_to_feed: Option<bool>,
}

/// Reply control type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplyControlType {
    /// Anyone can reply
    Everyone,
    /// Only accounts you follow can reply
    AccountsYouFollow,
    /// Only mentioned accounts can reply
    MentionedOnly,
}

impl ReplyControlType {
    /// Returns the API string for this type
    pub fn as_str(&self) -> &'static str {
        match self {
            ReplyControlType::Everyone => "everyone",
            ReplyControlType::AccountsYouFollow => "accounts_you_follow",
            ReplyControlType::MentionedOnly => "mentioned_only",
        }
    }
}

// ============================================================================
// Conversion Implementations
// ============================================================================

impl From<ThreadsMedia> for GhostPost {
    fn from(media: ThreadsMedia) -> Self {
        let mut post = GhostPost::new(&media.id, Platform::Threads, media.text.unwrap_or_default());

        post.like_count = Some(media.likes_count);
        post.repost_count = Some(media.reposts_count);
        post.reply_count = Some(media.replies_count);
        post.quote_count = Some(media.quotes_count);

        // Parse timestamp
        if let Some(timestamp) = &media.timestamp {
            post.created_at = parse_threads_timestamp(timestamp).unwrap_or(0);
        }

        // Set author
        if let Some(owner) = &media.owner {
            post.author = GhostUser {
                id: owner.id.clone(),
                platform: Platform::Threads,
                username: owner.username.clone().unwrap_or_default(),
                ..Default::default()
            };
        }

        // Set media attachments
        post.media = extract_media_from_response(&media);

        // Store raw metadata
        post.raw_metadata = Some(serde_json::to_value(&media).ok());

        post
    }
}

impl From<ThreadsUser> for GhostUser {
    fn from(user: ThreadsUser) -> Self {
        GhostUser {
            id: user.id,
            platform: Platform::Threads,
            username: user.username,
            display_name: user.name,
            bio: user.threads_biography,
            avatar_url: user.threads_profile_picture_url,
            followers_count: user.followers_count,
            following_count: user.following_count,
            posts_count: user.media_count,
            is_verified: Some(user.is_verified),
            ..Default::default()
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Parses a Threads timestamp string to Unix timestamp
fn parse_threads_timestamp(timestamp: &str) -> Result<i64, GhostError> {
    // Threads uses ISO 8601 format: "2024-01-15T10:30:00+0000"
    let dt = DateTime::parse_from_rfc3339(timestamp)
        .or_else(|_| {
            // Try alternate format
            chrono::DateTime::parse_from_str(timestamp, "%Y-%m-%dT%H:%M:%S%z")
        })
        .or_else(|_| {
            // Try without timezone
            chrono::NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%dT%H:%M:%S")
                .map(|dt| chrono::DateTime::from_naive_utc_and_offset(dt, Utc))
        });

    match dt {
        Ok(datetime) => Ok(datetime.timestamp()),
        Err(_) => Err(GhostError::ParseError(format!("Invalid timestamp: {}", timestamp))),
    }
}

/// Extracts media attachments from a Threads media response
fn extract_media_from_response(media: &ThreadsMedia) -> Vec<GhostMedia> {
    let mut result = Vec::new();

    // Handle single media
    if let Some(url) = &media.media_url {
        let media_type = match media.media_type.as_deref() {
            Some("IMAGE") => MediaType::Image,
            Some("VIDEO") => MediaType::Video,
            _ => MediaType::Unknown,
        };

        result.push(GhostMedia {
            id: media.id.clone(),
            media_type,
            url: url.clone(),
            preview_url: media.thumbnail_url.clone(),
            ..Default::default()
        });
    }

    // Handle carousel
    if let Some(children) = &media.children {
        for child in &children.data {
            if let Some(url) = &child.media_url {
                let media_type = match child.media_type.as_deref() {
                    Some("IMAGE") => MediaType::Image,
                    Some("VIDEO") => MediaType::Video,
                    _ => MediaType::Unknown,
                };

                result.push(GhostMedia {
                    id: child.id.clone(),
                    media_type,
                    url: url.clone(),
                    preview_url: child.thumbnail_url.clone(),
                    ..Default::default()
                });
            }
        }
    }

    result
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = ThreadsOfficialClient::new();
        assert!(client.app_id.is_none());
    }

    #[test]
    fn test_client_with_credentials() {
        let client = ThreadsOfficialClient::with_credentials(
            "test_app_id",
            "test_secret",
            "https://example.com/callback",
        );
        assert_eq!(client.app_id, Some("test_app_id".to_string()));
    }

    #[test]
    fn test_scope_as_str() {
        assert_eq!(ThreadsScope::ThreadsBasic.as_str(), "threads_basic");
        assert_eq!(ThreadsScope::ThreadsContentPublish.as_str(), "threads_content_publish");
    }

    #[test]
    fn test_authorization_url() {
        let client = ThreadsOfficialClient::with_credentials(
            "app123",
            "secret",
            "https://example.com/callback",
        );

        let url = client.get_authorization_url(
            &[ThreadsScope::ThreadsBasic, ThreadsScope::ThreadsContentPublish],
            Some("state123"),
        ).unwrap();

        assert!(url.contains("client_id=app123"));
        assert!(url.contains("threads_basic"));
        assert!(url.contains("threads_content_publish"));
        assert!(url.contains("state=state123"));
    }

    #[test]
    fn test_parse_timestamp() {
        let ts = parse_threads_timestamp("2024-01-15T10:30:00Z").unwrap();
        assert!(ts > 0);
    }

    #[test]
    fn test_reply_control_as_str() {
        assert_eq!(ReplyControlType::Everyone.as_str(), "everyone");
        assert_eq!(ReplyControlType::AccountsYouFollow.as_str(), "accounts_you_follow");
        assert_eq!(ReplyControlType::MentionedOnly.as_str(), "mentioned_only");
    }

    #[test]
    fn test_threads_media_to_ghost_post() {
        let media = ThreadsMedia {
            id: "123456".to_string(),
            text: Some("Hello world!".to_string()),
            timestamp: Some("2024-01-15T10:30:00Z".to_string()),
            likes_count: 42,
            replies_count: 5,
            reposts_count: 10,
            quotes_count: 3,
            ..Default::default()
        };

        let post: GhostPost = media.into();
        assert_eq!(post.id, "123456");
        assert_eq!(post.text, "Hello world!");
        assert_eq!(post.like_count, Some(42));
        assert_eq!(post.platform, Platform::Threads);
    }

    #[test]
    fn test_threads_user_to_ghost_user() {
        let user = ThreadsUser {
            id: "user123".to_string(),
            username: "testuser".to_string(),
            name: Some("Test User".to_string()),
            threads_biography: Some("Bio text".to_string()),
            followers_count: Some(1000),
            is_verified: true,
            ..Default::default()
        };

        let ghost: GhostUser = user.into();
        assert_eq!(ghost.id, "user123");
        assert_eq!(ghost.username, "testuser");
        assert_eq!(ghost.display_name, Some("Test User".to_string()));
        assert_eq!(ghost.followers_count, Some(1000));
        assert_eq!(ghost.is_verified, Some(true));
    }
}
