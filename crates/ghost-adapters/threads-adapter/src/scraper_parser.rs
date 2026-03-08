//! Threads Scraper Output Parser
//!
//! Parses the output from py-threads wrapper (threads-scraper) into GhostPost types.
//! 
//! The scraper output is minimal JSON - this adapter handles all field mapping
//! and provides defaults for missing fields per the ghost-mapping.md specification.

use ghost_schema::{
    GhostError, GhostPost, GhostUser, GhostMedia, MediaType,
    Platform, PayloadBlob, AdapterParseResult,
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
        // Check for error in blob
        if blob.error.is_some() || blob.status_code >= 400 {
            let error_msg = blob.error.as_deref().unwrap_or("Unknown scraper error");
            return Err(GhostError::ParseError(format!("Scraper error: {}", error_msg)));
        }
        
        // Parse JSON array
        let posts: Vec<ScraperPost> = serde_json::from_slice(&blob.data)
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
            raw_metadata: Some(serde_json::to_value(&blob.metadata).unwrap_or(serde_json::Value::Null)),
            source_url: blob.source_url.clone(),
        };
        
        Ok(result)
    }
    
    /// Map a scraper post to GhostPost with defaults for missing fields
    fn map_scraper_post_to_ghost(&self, post: ScraperPost) -> GhostPost {
        // Construct minimal GhostUser from username only
        // The scraper doesn't extract full user data
        let author = GhostUser {
            id: String::new(),  // Not available from scraper
            platform: Platform::Threads,
            username: post.author.clone(),
            display_name: None,  // Not available
            avatar_url: None,    // Not available
            is_verified: None,   // Not available
            profile_url: Some(format!("https://www.threads.net/@{}", post.author)),
            bio: None,
            follower_count: None,
            following_count: None,
            post_count: None,
            created_at: None,
            raw_metadata: None,
        };
        
        // Build post URL if code is available
        let post_url = post.code.as_ref().map(|code| {
            format!("https://www.threads.net/@{}/post/{}", post.author, code)
        });
        
        GhostPost {
            id: post.id,
            platform: Platform::Threads,
            text: post.text.unwrap_or_default(),
            author,
            media: vec![],  // Not extracted by scraper
            created_at: current_timestamp(),  // CRITICAL: Not available from scraper - use current time
            like_count: post.likes,
            repost_count: None,  // Not in scraper output
            reply_count: post.reply_count,
            view_count: None,    // Requires insights API
            quote_count: None,   // Not in scraper output
            in_reply_to: None,   // Could be derived from BFS context
            quoted_post: None,
            post_url,
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
    let parser = ScraperParser::new(&blob.source_url);
    parser.parse(blob)
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
    fn test_payload_blob_parsing() {
        let json = r#"[{"id": "1", "author": "user", "text": "hello"}]"#;
        
        let blob = PayloadBlob {
            data: json.as_bytes().to_vec(),
            content_type: "application/json".to_string(),
            source_url: "https://test.com".to_string(),
            status_code: 200,
            headers: Default::default(),
            metadata: serde_json::json!({"posts_found": 1}),
            error: None,
        };
        
        let result = parse_scraper_blob(&blob).unwrap();
        assert_eq!(result.posts.len(), 1);
    }
    
    #[test]
    fn test_error_blob() {
        let blob = PayloadBlob {
            data: br#"{"error": "timeout"}"#.to_vec(),
            content_type: "application/json".to_string(),
            source_url: "https://test.com".to_string(),
            status_code: 500,
            headers: Default::default(),
            metadata: Default::default(),
            error: Some("timeout".to_string()),
        };
        
        let result = parse_scraper_blob(&blob);
        assert!(result.is_err());
    }
}
