//! Threads platform adapter implementation
//!
//! Types imported from ghost-schema - the single source of truth.

use ghost_schema::{
    GhostError, GhostPost, GhostUser, Platform, PayloadBlob, PayloadContentType,
    AdapterParseResult, AdapterError, ThreadsError,
};

use crate::parser::{PostParser, UserParser};
use crate::relay::{RelayResponse, ThreadsQueries};
use crate::scraper_parser::ScraperParser;

/// Threads platform adapter
pub struct ThreadsAdapter {
    /// Platform identifier
    platform: Platform,
    /// Post parser
    post_parser: PostParser,
    /// User parser
    user_parser: UserParser,
}

impl ThreadsAdapter {
    /// Creates a new Threads adapter
    pub fn new() -> Self {
        Self {
            platform: Platform::Threads,
            post_parser: PostParser::new(),
            user_parser: UserParser::new(),
        }
    }

    /// Parses a payload into Ghost types
    pub fn parse(&self, blob: &PayloadBlob) -> Result<AdapterParseResult, GhostError> {
        match blob.content_type {
            PayloadContentType::Json => self.parse_json(blob),
            _ => Err(GhostError::AdapterError("Unsupported content type for Threads".into())),
        }
    }

    /// Parses JSON response (Relay or scraper format)
    fn parse_json(&self, blob: &PayloadBlob) -> Result<AdapterParseResult, GhostError> {
        let json_str = blob.as_text()
            .map_err(|e| GhostError::ParseError(format!("Failed to decode as text: {}", e)))?;

        // Try parsing as scraper output first (simpler format)
        if let Ok(result) = self.try_parse_scraper(json_str, blob) {
            return Ok(result);
        }

        // Try parsing as Relay response
        let relay = RelayResponse::from_json(json_str)?;

        // Check for errors
        if relay.has_errors() {
            let error = relay.errors
                .and_then(|errors| errors.into_iter().next())
                .map(|e| ThreadsError::ParseError { message: e.message })
                .unwrap_or_else(|| ThreadsError::ParseError { message: "Unknown error".into() });
            return Ok(AdapterParseResult::with_error(self.map_threads_error(error)));
        }

        // Parse the data
        if let Some(data) = relay.extract_data() {
            self.parse_data(data)
        } else {
            Ok(AdapterParseResult::new())
        }
    }

    /// Try parsing as scraper output
    fn try_parse_scraper(
        &self,
        json_str: &str,
        blob: &PayloadBlob,
    ) -> Result<AdapterParseResult, GhostError> {
        // Scraper output is typically an array of posts
        if json_str.trim_start().starts_with('[') {
            let parser = ScraperParser::new(blob.source_url.as_deref().unwrap_or(""));
            return parser.parse(blob);
        }
        Err(GhostError::ParseError("Not scraper format".into()))
    }

    /// Parse data from Relay response
    fn parse_data(&self, data: &serde_json::Value) -> Result<AdapterParseResult, GhostError> {
        // Check for thread data
        if let Some(thread) = data.get("data").and_then(|d| d.get("thread")) {
            return self.parse_thread_internal(thread);
        }

        // Check for user data
        if let Some(user_data) = data.get("userData") {
            let user = self.user_parser.parse(user_data)?;
            return Ok(AdapterParseResult::with_user(user));
        }

        // Check for timeline
        if let Some(timeline) = data.get("timeline") {
            return self.parse_timeline_internal(timeline);
        }

        // Check for post container
        if let Some(container) = data.get("thread_data").or_else(|| data.get("container")) {
            let post = self.post_parser.parse(container)?;
            return Ok(AdapterParseResult::with_post(post));
        }

        // Check for single post
        if data.get("id").is_some() {
            let post = self.post_parser.parse(data)?;
            return Ok(AdapterParseResult::with_post(post));
        }

        // Check for single user
        if data.get("username").is_some() || data.get("pk").is_some() {
            let user = self.user_parser.parse(data)?;
            return Ok(AdapterParseResult::with_user(user));
        }

        // Try parsing as array of posts
        if let Some(arr) = data.as_array() {
            let posts: Vec<GhostPost> = arr.iter()
                .filter_map(|item| self.post_parser.parse(item).ok())
                .collect();
            if !posts.is_empty() {
                return Ok(AdapterParseResult::with_posts(posts));
            }
        }

        Err(GhostError::AdapterError("Could not parse Threads data".into()))
    }

    /// Parse a thread (conversation) - internal helper
    fn parse_thread_internal(&self, thread: &serde_json::Value) -> Result<AdapterParseResult, GhostError> {
        let mut posts = Vec::new();

        // Parse main post
        if let Some(thread_items) = thread.get("thread_items").and_then(|t| t.as_array()) {
            for item in thread_items {
                if let Some(post_item) = item.get("post") {
                    if let Ok(post) = self.post_parser.parse(post_item) {
                        posts.push(post);
                    }
                }
            }
        }

        // Try parsing thread as single post with replies
        if posts.is_empty() {
            if let Ok(post) = self.post_parser.parse(thread) {
                posts.push(post);
            }
        }

        Ok(AdapterParseResult::with_posts(posts))
    }

    /// Parse timeline - internal helper
    fn parse_timeline_internal(&self, timeline: &serde_json::Value) -> Result<AdapterParseResult, GhostError> {
        let mut posts = Vec::new();

        // Parse timeline items
        if let Some(items) = timeline.get("items").and_then(|i| i.as_array()) {
            for item in items {
                if let Some(post) = item.get("post") {
                    if let Ok(ghost_post) = self.post_parser.parse(post) {
                        posts.push(ghost_post);
                    }
                }
            }
        }

        Ok(AdapterParseResult::with_posts(posts))
    }

    /// Parses a single post
    pub fn parse_post(&self, data: &serde_json::Value) -> Result<GhostPost, GhostError> {
        self.post_parser.parse(data)
    }

    /// Parses a user profile
    pub fn parse_user(&self, data: &serde_json::Value) -> Result<GhostUser, GhostError> {
        self.user_parser.parse(data)
    }

    /// Parses thread (conversation)
    pub fn parse_thread(&self, data: &serde_json::Value) -> Result<Vec<GhostPost>, GhostError> {
        let result = self.parse_thread_internal(data)?;
        Ok(result.into_posts())
    }

    /// Parses search results
    pub fn parse_search(&self, data: &serde_json::Value) -> Result<Vec<GhostPost>, GhostError> {
        // Search results are similar to timeline
        let result = self.parse_timeline(data)?;
        Ok(result.into_posts())
    }

    /// Parses timeline
    pub fn parse_timeline(&self, data: &serde_json::Value) -> Result<AdapterParseResult, GhostError> {
        self.parse_timeline_internal(data)
    }

    /// Detects errors in the response
    pub fn detect_error(&self, data: &serde_json::Value) -> Option<ThreadsError> {
        // Check for error response
        if let Some(error) = data.get("error") {
            let message = error.get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");
            return Some(ThreadsError::ParseError { message: message.to_string() });
        }

        // Check for errors array
        if let Some(errors) = data.get("errors").and_then(|e| e.as_array()) {
            if !errors.is_empty() {
                let message = errors[0].get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Unknown error");
                return Some(ThreadsError::ParseError { message: message.to_string() });
            }
        }

        // Check for rate limit indicator
        if data.get("status").and_then(|s| s.as_str()) == Some("fail") {
            if let Some(message) = data.get("message").and_then(|m| m.as_str()) {
                if message.contains("rate limit") || message.contains("too many") {
                    return Some(ThreadsError::RateLimited { retry_after: None });
                }
            }
        }

        None
    }

    /// Map ThreadsError to AdapterError
    fn map_threads_error(&self, error: ThreadsError) -> AdapterError {
        match error {
            ThreadsError::RateLimited { retry_after } => AdapterError::RateLimited {
                retry_after,
                platform: Platform::Threads,
            },
            ThreadsError::AccountSuspended { user_id } => AdapterError::AccountSuspended {
                user_id,
                platform: Platform::Threads,
            },
            ThreadsError::NotFound { resource_type, resource_id } => AdapterError::NotFound {
                resource_type,
                resource_id,
                platform: Platform::Threads,
            },
            ThreadsError::PrivateAccount { user_id } => AdapterError::ProtectedAccount {
                user_id,
                platform: Platform::Threads,
            },
            ThreadsError::LoginRequired => AdapterError::LoginRequired {
                platform: Platform::Threads,
            },
            ThreadsError::Checkpoint { url } => AdapterError::SuspiciousActivity {
                challenge_url: url,
                platform: Platform::Threads,
            },
            ThreadsError::ParseError { message } => AdapterError::ParseError {
                message,
                platform: Platform::Threads,
            },
        }
    }
}

impl Default for ThreadsAdapter {
    fn default() -> Self {
        Self::new()
    }
}
