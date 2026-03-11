//! X platform adapter implementation
//!
//! Types imported from ghost-schema - the single source of truth.

use ghost_schema::{
    GhostError, GhostPost, GhostUser, Platform, PayloadBlob, PayloadContentType,
    AdapterParseResult, AdapterError, XError, TrendingTopic,
    XTweetResponse, XTweetData, XUserData, XIncludes,
};

use crate::parser::{PostParser, UserParser};

/// X (Twitter) platform adapter
pub struct XAdapter {
    /// Platform identifier
    #[allow(dead_code)]
    platform: Platform,
    /// Post parser
    post_parser: PostParser,
    /// User parser
    user_parser: UserParser,
}

impl XAdapter {
    /// Creates a new X adapter
    pub fn new() -> Self {
        Self {
            platform: Platform::X,
            post_parser: PostParser::new(),
            user_parser: UserParser::new(),
        }
    }

    /// Parses a payload into Ghost types
    pub fn parse(&self, blob: &PayloadBlob) -> Result<AdapterParseResult, GhostError> {
        match blob.content_type {
            PayloadContentType::Json => self.parse_json(blob),
            PayloadContentType::Html => self.parse_html(blob),
            _ => Err(GhostError::AdapterError("Unsupported content type".into())),
        }
    }

    /// Parses JSON response (GraphQL or API v2)
    fn parse_json(&self, blob: &PayloadBlob) -> Result<AdapterParseResult, GhostError> {
        let json_str = blob.as_text()
            .map_err(|e| GhostError::ParseError(format!("Failed to decode as text: {}", e)))?;

        let json: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| GhostError::ParseError(format!("Invalid JSON: {}", e)))?;

        // Check for errors first
        if let Some(error) = self.detect_error(&json) {
            return Ok(AdapterParseResult::with_error(self.map_x_error(error)));
        }

        // Detect response type and parse accordingly
        if let Some(data) = json.get("data") {
            // GraphQL or API v2 format
            self.parse_data_response(data, json.get("includes"))
        } else if json.is_array() {
            // Array of tweets
            self.parse_tweets_array(&json)
        } else if json.get("id").is_some() && json.get("text").is_some() {
            // Single tweet object
            self.parse_single_tweet(&json)
        } else if json.get("username").is_some() || json.get("screen_name").is_some() {
            // User object
            self.parse_user_object(&json)
        } else {
            Err(GhostError::AdapterError("Unknown JSON response format".into()))
        }
    }

    /// Parse a data response (GraphQL/API v2)
    fn parse_data_response(
        &self,
        data: &serde_json::Value,
        includes: Option<&serde_json::Value>,
    ) -> Result<AdapterParseResult, GhostError> {
        // Check if it's a tweet response
        if let Some(tweet_result) = data.get("tweetResult") {
            let result = tweet_result.get("result")
                .ok_or_else(|| GhostError::ParseError("Missing tweet result".into()))?;
            let post = self.post_parser.parse(result)?;
            return Ok(AdapterParseResult::with_post(post));
        }

        // Check if it's a user response
        if let Some(user_result) = data.get("userResult") {
            let result = user_result.get("result")
                .ok_or_else(|| GhostError::ParseError("Missing user result".into()))?;
            let user = self.user_parser.parse(result)?;
            return Ok(AdapterParseResult::with_user(user));
        }

        // Check for timeline instructions
        if let Some(instructions) = data.get("timeline").and_then(|t| t.get("instructions")) {
            return self.parse_timeline_instructions(instructions);
        }

        // Try parsing as tweet data directly
        if data.get("id").is_some() {
            let post = self.post_parser.parse(data)?;
            return Ok(AdapterParseResult::with_post(post));
        }

        // Try parsing as user data directly
        if data.get("username").is_some() || data.get("id_str").is_some() {
            let user = self.user_parser.parse(data)?;
            return Ok(AdapterParseResult::with_user(user));
        }

        // Check for standard API v2 format
        if let Ok(response) = serde_json::from_value::<XTweetResponse>(data.clone()) {
            let includes: Option<XIncludes> = includes
                .and_then(|v| serde_json::from_value(v.clone()).ok());
            return self.parse_tweet_response(response, includes);
        }

        Err(GhostError::AdapterError("Could not parse data response".into()))
    }

    /// Parse standard X API v2 tweet response
    fn parse_tweet_response(
        &self,
        response: XTweetResponse,
        includes: Option<XIncludes>,
    ) -> Result<AdapterParseResult, GhostError> {
        // Check for errors
        if let Some(errors) = &response.errors {
            if !errors.is_empty() {
                return Ok(AdapterParseResult::with_error(AdapterError::ParseError {
                    message: errors[0].detail.clone().unwrap_or_else(|| "Unknown error".to_string()),
                    platform: Platform::X,
                }));
            }
        }

        // Parse the main tweet data
        if let Some(data) = &response.data {
            let post = self.parse_tweet_data(data, &includes)?;
            return Ok(AdapterParseResult::with_post(post).source(""));
        }

        Err(GhostError::ParseError("No tweet data in response".into()))
    }

    /// Parse tweet data with includes
    fn parse_tweet_data(
        &self,
        data: &XTweetData,
        includes: &Option<XIncludes>,
    ) -> Result<GhostPost, GhostError> {
        // Get author from includes if available
        let author = if let Some(includes) = includes {
            if let Some(ref users) = includes.users {
                users.iter()
                    .find(|u| &u.id == data.author_id.as_ref().unwrap_or(&String::new()))
                    .map(|u| self.parse_user_data(u))
                    .transpose()?
            } else {
                None
            }
        } else {
            None
        };

        // Create post
        let post = GhostPost {
            id: data.id.clone(),
            platform: Platform::X,
            text: data.text.clone(),
            author: author.unwrap_or_else(|| GhostUser::new("", Platform::X, "")),
            media: vec![],
            created_at: data.created_at.as_ref()
                .and_then(|s| parse_iso8601_timestamp(s))
                .unwrap_or(0),
            like_count: data.public_metrics.as_ref().map(|m| m.like_count),
            repost_count: data.public_metrics.as_ref().map(|m| m.retweet_count),
            reply_count: data.public_metrics.as_ref().map(|m| m.reply_count),
            view_count: data.public_metrics.as_ref().map(|m| m.impression_count),
            quote_count: data.public_metrics.as_ref().map(|m| m.quote_count),
            in_reply_to: data.referenced_tweets.as_ref()
                .and_then(|refs| refs.iter().find(|r| r.reference_type == "replied_to"))
                .map(|r| r.id.clone()),
            quoted_post: None,
            raw_metadata: Some(serde_json::to_value(data)?),
        };

        Ok(post)
    }

    /// Parse user data
    fn parse_user_data(&self, data: &XUserData) -> Result<GhostUser, GhostError> {
        Ok(GhostUser {
            id: data.id.clone(),
            platform: Platform::X,
            username: data.username.clone(),
            display_name: Some(data.name.clone()),
            bio: data.description.clone(),
            avatar_url: data.profile_image_url.clone(),
            banner_url: None,
            profile_url: Some(format!("https://x.com/{}", data.username)),
            location: data.location.clone(),
            website: data.url.clone(),
            followers_count: data.public_metrics.as_ref().map(|m| m.followers_count),
            following_count: data.public_metrics.as_ref().map(|m| m.following_count),
            posts_count: data.public_metrics.as_ref().map(|m| m.tweet_count),
            is_verified: data.verified,
            is_private: data.protected,
            is_bot: None,
            created_at: data.created_at.as_ref()
                .and_then(|s| parse_iso8601_timestamp(s)),
            raw_metadata: Some(serde_json::to_value(data)?),
        })
    }

    /// Parse timeline instructions
    fn parse_timeline_instructions(
        &self,
        instructions: &serde_json::Value,
    ) -> Result<AdapterParseResult, GhostError> {
        let mut posts = Vec::new();
        let mut cursor_top: Option<String> = None;
        let mut cursor_bottom: Option<String> = None;

        if let Some(instructions_arr) = instructions.as_array() {
            for instruction in instructions_arr {
                // Handle timeline entries
                if let Some(entries) = instruction.get("entries").and_then(|e| e.as_array()) {
                    for entry in entries {
                        // Extract cursor values
                        if let Some(entry_id) = entry.get("entryId").and_then(|e| e.as_str()) {
                            if entry_id.starts_with("cursor-top") {
                                cursor_top = entry.get("content")
                                    .and_then(|c| c.get("value"))
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string());
                            } else if entry_id.starts_with("cursor-bottom") {
                                cursor_bottom = entry.get("content")
                                    .and_then(|c| c.get("value"))
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string());
                            }
                        }

                        // Parse tweet entries
                        if let Some(content) = entry.get("content") {
                            if let Some(item_content) = content.get("itemContent") {
                                if let Some(tweet_results) = item_content.get("tweet_results") {
                                    if let Some(result) = tweet_results.get("result") {
                                        if let Ok(post) = self.post_parser.parse(result) {
                                            posts.push(post);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut result = AdapterParseResult::with_posts(posts);
        result.cursor_top = cursor_top;
        result.cursor_bottom = cursor_bottom;
        Ok(result)
    }

    /// Parse an array of tweets
    fn parse_tweets_array(&self, json: &serde_json::Value) -> Result<AdapterParseResult, GhostError> {
        let posts = json.as_array()
            .ok_or_else(|| GhostError::ParseError("Expected array".into()))?
            .iter()
            .filter_map(|tweet| self.post_parser.parse(tweet).ok())
            .collect();

        Ok(AdapterParseResult::with_posts(posts))
    }

    /// Parse a single tweet object
    fn parse_single_tweet(&self, json: &serde_json::Value) -> Result<AdapterParseResult, GhostError> {
        let post = self.post_parser.parse(json)?;
        Ok(AdapterParseResult::with_post(post))
    }

    /// Parse a user object
    fn parse_user_object(&self, json: &serde_json::Value) -> Result<AdapterParseResult, GhostError> {
        let user = self.user_parser.parse(json)?;
        Ok(AdapterParseResult::with_user(user))
    }

    /// Parses HTML response
    fn parse_html(&self, blob: &PayloadBlob) -> Result<AdapterParseResult, GhostError> {
        // HTML parsing for scraping mode
        // This would use a proper HTML parser like scraper or select
        // For now, return empty result with note
        tracing::warn!("HTML parsing not fully implemented");
        
        Ok(AdapterParseResult::new().source(
            blob.source_url.as_deref().unwrap_or("")
        ))
    }

    /// Parses a single post
    pub fn parse_post(&self, data: &serde_json::Value) -> Result<GhostPost, GhostError> {
        self.post_parser.parse(data)
    }

    /// Parses a user profile
    pub fn parse_user(&self, data: &serde_json::Value) -> Result<GhostUser, GhostError> {
        self.user_parser.parse(data)
    }

    /// Parses trending topics
    pub fn parse_trending(&self, data: &serde_json::Value) -> Result<Vec<TrendingTopic>, GhostError> {
        let mut topics = Vec::new();

        if let Some(instructions) = data.get("timeline").and_then(|t| t.get("instructions")) {
            if let Some(instructions_arr) = instructions.as_array() {
                for instruction in instructions_arr {
                    if let Some(entries) = instruction.get("entries").and_then(|e| e.as_array()) {
                        for entry in entries {
                            if let Some(content) = entry.get("content") {
                                if let Some(item_content) = content.get("itemContent") {
                                    if let Some(trend) = item_content.get("trend") {
                                        let name = trend.get("name")
                                            .and_then(|n| n.as_str())
                                            .unwrap_or("");
                                        
                                        topics.push(TrendingTopic::new(name, Platform::X));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(topics)
    }

    /// Parses search results
    pub fn parse_search(&self, data: &serde_json::Value) -> Result<Vec<GhostPost>, GhostError> {
        let result = self.parse_json_from_value(data)?;
        Ok(result.into_posts())
    }

    /// Parses timeline
    pub fn parse_timeline(&self, data: &serde_json::Value) -> Result<Vec<GhostPost>, GhostError> {
        let result = self.parse_json_from_value(data)?;
        Ok(result.into_posts())
    }

    /// Parse JSON from value
    fn parse_json_from_value(&self, json: &serde_json::Value) -> Result<AdapterParseResult, GhostError> {
        if let Some(error) = self.detect_error(json) {
            return Ok(AdapterParseResult::with_error(self.map_x_error(error)));
        }
        self.parse_data_response(json, None)
    }

    /// Detects errors in the response
    pub fn detect_error(&self, data: &serde_json::Value) -> Option<XError> {
        // Check for explicit errors
        if let Some(errors) = data.get("errors").and_then(|e| e.as_array()) {
            if !errors.is_empty() {
                let error = &errors[0];
                return Some(XError::ParseError {
                    message: error.get("detail")
                        .or_else(|| error.get("message"))
                        .and_then(|m| m.as_str())
                        .unwrap_or("Unknown error")
                        .to_string(),
                });
            }
        }

        // Check for rate limit
        if data.get("title").and_then(|t| t.as_str()) == Some("Too Many Requests") {
            let retry_after = data.get("retry_after")
                .and_then(|r| r.as_u64());
            return Some(XError::rate_limited(retry_after));
        }

        // Check for 403/401 style errors
        if let Some(status) = data.get("status").and_then(|s| s.as_u64()) {
            match status {
                429 => {
                    let retry_after = data.get("retry_after").and_then(|r| r.as_u64());
                    return Some(XError::rate_limited(retry_after));
                }
                403 | 401 => {
                    return Some(XError::LoginRequired);
                }
                404 => {
                    return Some(XError::NotFound {
                        resource_type: "unknown".to_string(),
                        resource_id: "unknown".to_string(),
                    });
                }
                _ => {}
            }
        }

        None
    }

    /// Map XError to AdapterError
    fn map_x_error(&self, error: XError) -> AdapterError {
        match error {
            XError::RateLimited { retry_after } => AdapterError::RateLimited {
                retry_after,
                platform: Platform::X,
            },
            XError::AccountSuspended { user_id } => AdapterError::AccountSuspended {
                user_id,
                platform: Platform::X,
            },
            XError::NotFound { resource_type, resource_id } => AdapterError::NotFound {
                resource_type,
                resource_id,
                platform: Platform::X,
            },
            XError::ProtectedAccount { user_id } => AdapterError::ProtectedAccount {
                user_id,
                platform: Platform::X,
            },
            XError::LoginRequired => AdapterError::LoginRequired {
                platform: Platform::X,
            },
            XError::SuspiciousActivity { challenge_url } => AdapterError::SuspiciousActivity {
                challenge_url,
                platform: Platform::X,
            },
            XError::ParseError { message } => AdapterError::ParseError {
                message,
                platform: Platform::X,
            },
        }
    }
}

impl Default for XAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse ISO 8601 timestamp to Unix timestamp
fn parse_iso8601_timestamp(s: &str) -> Option<i64> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.timestamp())
        .ok()
}
