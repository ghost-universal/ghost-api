//! Threads Scraper Output Parser
//!
//! Parses the output from py-threads wrapper (threads-scraper) into GhostPost types.
//! 
//! The scraper output is minimal JSON - this adapter handles all field mapping
//! and provides defaults for missing fields per the ghost-mapping.md specification.

use ghost_schema::{
    GhostError, GhostPost, GhostUser, GhostMedia, MediaType,
    Platform, PayloadBlob, PayloadContentType, AdapterParseResult,
};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Minimal scraper output structure from threads-scraper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScraperPost {
    /// Unique post ID
    pub id: String,
    
    /// Post code (used in URL generation)
    #[serde(default)]
    pub code: Option<String>,
    
    /// Post text content
    #[serde(default)]
    pub text: Option<String>,
    
    /// Author username
    pub author: String,
    
    /// Like count
    #[serde(default)]
    pub likes: Option<u64>,
    
    /// Reply count
    #[serde(default)]
    pub reply_count: Option<u64>,
}

/// Wrapper for scraper output (array of posts)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScraperOutput(pub Vec<ScraperPost>);

/// Python worker response structure
/// This is what the Python wrapper returns (different from ghost-schema PayloadBlob)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerResponse {
    /// JSON-encoded data
    pub data: String,
    /// Content type
    pub content_type: String,
    /// Source URL
    pub source_url: String,
    /// HTTP status code
    pub status_code: u16,
    /// Response headers
    #[serde(default)]
    pub headers: std::collections::HashMap<String, String>,
    /// Additional metadata
    #[serde(default)]
    pub metadata: serde_json::Value,
    /// Error message if failed
    #[serde(default)]
    pub error: Option<String>,
}

/// Parser for threads-scraper output
pub struct ScraperParser {
    /// Source URL for the scrape
    source_url: String,
}

impl ScraperParser {
    /// Create a new scraper parser
    pub fn new(source_url: impl Into<String>) -> Self {
        Self {
            source_url: source_url.into(),
        }
    }
    
    /// Parse raw PayloadBlob from the scraper into GhostPosts
    pub fn parse(&self, blob: &PayloadBlob) -> Result<AdapterParseResult, GhostError> {
        // Check for error status
        if blob.status_code >= 400 {
            return Err(GhostError::ParseError(format!("Scraper error: HTTP {}", blob.status_code)));
        }
        
        // Parse JSON array
        let posts: Vec<ScraperPost> = blob.as_json()
            .map_err(|e| GhostError::ParseError(format!("Failed to parse scraper output: {}", e)))?;
        
        // Convert to GhostPosts
        let ghost_posts: Vec<GhostPost> = posts
            .into_iter()
            .map(|post| self.map_scraper_post_to_ghost(post))
            .collect();
        
        // Build result
        let result = AdapterParseResult {
            posts: ghost_posts,
            users: vec![],  // Users are embedded in posts
            media: vec![],  // Media is embedded in posts
            raw_metadata: None,
            source_url: self.source_url.clone(),
            cursor: None,
            cursor_top: None,
            cursor_bottom: None,
            error: None,
        };
        
        Ok(result)
    }
    
    /// Parse from Python worker response JSON
    pub fn parse_worker_response(&self, response_json: &str) -> Result<AdapterParseResult, GhostError> {
        let response: WorkerResponse = serde_json::from_str(response_json)
            .map_err(|e| GhostError::ParseError(format!("Failed to parse worker response: {}", e)))?;
        
        // Check for error
        if response.error.is_some() || response.status_code >= 400 {
            let error_msg = response.error.unwrap_or_else(|| format!("HTTP {}", response.status_code));
            return Err(GhostError::ParseError(format!("Worker error: {}", error_msg)));
        }
        
        // Parse posts from data field
        let posts: Vec<ScraperPost> = serde_json::from_str(&response.data)
            .map_err(|e| GhostError::ParseError(format!("Failed to parse scraper output: {}", e)))?;
        
        // Convert to GhostPosts
        let ghost_posts: Vec<GhostPost> = posts
            .into_iter()
            .map(|post| self.map_scraper_post_to_ghost(post))
            .collect();
        
        Ok(AdapterParseResult {
            posts: ghost_posts,
            users: vec![],
            media: vec![],
            raw_metadata: Some(response.metadata),
            source_url: response.source_url,
            cursor: None,
            cursor_top: None,
            cursor_bottom: None,
            error: None,
        })
    }
    
    /// Map a scraper post to GhostPost with defaults for missing fields
    fn map_scraper_post_to_ghost(&self, post: ScraperPost) -> GhostPost {
        // Construct minimal GhostUser from username only
        // The scraper doesn't extract full user data
        // Note: Using correct field names from ghost-schema (followers_count not follower_count)
        let author = GhostUser {
            id: String::new(),  // Not available from scraper - falls back to empty per ghost-mapping.md
            platform: Platform::Threads,
            username: post.author.clone(),
            display_name: None,  // Not available from scraper
            avatar_url: None,    // Not available from scraper
            banner_url: None,
            profile_url: Some(format!("https://www.threads.net/@{}", post.author)),
            bio: None,
            location: None,
            website: None,
            followers_count: None,  // Correct field name
            following_count: None,
            posts_count: None,      // Correct field name  
            is_verified: None,      // Not available from scraper
            is_private: None,
            is_bot: None,
            created_at: None,
            raw_metadata: None,
        };
        
        GhostPost {
            id: post.id.clone(),
            platform: Platform::Threads,
            text: post.text.clone().unwrap_or_default(),
            author,
            media: vec![],  // Not extracted by scraper - empty per architecture
            created_at: current_timestamp(),  // CRITICAL GAP: Not available from scraper - use current time as fallback
            like_count: post.likes,
            repost_count: None,  // Not in scraper output
            reply_count: post.reply_count,
            view_count: None,    // Requires insights API
            quote_count: None,   // Not in scraper output
            in_reply_to: None,   // Could be derived from BFS context in future
            quoted_post: None,
            raw_metadata: Some(serde_json::to_value(&post).unwrap_or(serde_json::Value::Null)),
        }
    }
}

/// Get current Unix timestamp
fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Parse threads-scraper JSON output directly
pub fn parse_scraper_output(json: &str, source_url: &str) -> Result<Vec<GhostPost>, GhostError> {
    let posts: Vec<ScraperPost> = serde_json::from_str(json)
        .map_err(|e| GhostError::ParseError(format!("Failed to parse JSON: {}", e)))?;
    
    let parser = ScraperParser::new(source_url);
    
    Ok(posts
        .into_iter()
        .map(|post| parser.map_scraper_post_to_ghost(post))
        .collect())
}

/// Parse PayloadBlob from scraper
pub fn parse_scraper_blob(blob: &PayloadBlob) -> Result<AdapterParseResult, GhostError> {
    let parser = ScraperParser::new(blob.source_url.as_deref().unwrap_or(""));
    parser.parse(blob)
}

/// Parse Python worker response JSON
pub fn parse_worker_json(json: &str) -> Result<AdapterParseResult, GhostError> {
    let parser = ScraperParser::new("");
    parser.parse_worker_response(json)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_scraper_output() {
        let json = r#"
        [
            {
                "id": "17944322110154832",
                "code": "C_9xYzABC123",
                "text": "This is a test post",
                "author": "test_user",
                "likes": 42,
                "reply_count": 5
            }
        ]
        "#;
        
        let posts = parse_scraper_output(json, "https://example.com").unwrap();
        
        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0].id, "17944322110154832");
        assert_eq!(posts[0].text, "This is a test post");
        assert_eq!(posts[0].author.username, "test_user");
        assert_eq!(posts[0].like_count, Some(42));
        assert_eq!(posts[0].reply_count, Some(5));
        assert_eq!(posts[0].platform, Platform::Threads);
    }
    
    #[test]
    fn test_parse_minimal_post() {
        let json = r#"
        [
            {
                "id": "123456",
                "author": "minimal_user"
            }
        ]
        "#;
        
        let posts = parse_scraper_output(json, "https://example.com").unwrap();
        
        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0].id, "123456");
        assert_eq!(posts[0].text, "");  // Default empty
        assert_eq!(posts[0].author.username, "minimal_user");
        assert_eq!(posts[0].like_count, None);  // Default None
    }
    
    #[test]
    fn test_parse_invalid_json() {
        let json = "not valid json";
        let result = parse_scraper_output(json, "https://example.com");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_worker_response_parsing() {
        let response_json = r#"
        {
            "data": "[{\"id\": \"1\", \"author\": \"user\", \"text\": \"hello\"}]",
            "content_type": "application/json",
            "source_url": "https://test.com",
            "status_code": 200,
            "headers": {},
            "metadata": {"posts_found": 1},
            "error": null
        }
        "#;
        
        let result = parse_worker_json(response_json).unwrap();
        assert_eq!(result.posts.len(), 1);
        assert_eq!(result.posts[0].author.username, "user");
    }
    
    #[test]
    fn test_worker_error_response() {
        let response_json = r#"
        {
            "data": "{\"error\": \"timeout\"}",
            "content_type": "application/json",
            "source_url": "https://test.com",
            "status_code": 500,
            "headers": {},
            "metadata": {},
            "error": "timeout"
        }
        "#;
        
        let result = parse_worker_json(response_json);
        assert!(result.is_err());
    }
}
