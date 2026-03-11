//! Threads Relay-style JSON parsing
//!
//! Threads uses Meta's Relay GraphQL format with specific structure

use ghost_schema::GhostError;
use serde::{Deserialize, Serialize};

/// Relay-style response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayResponse {
    /// Response data
    pub data: Option<serde_json::Value>,
    /// Response extensions (timing, etc.)
    pub extensions: Option<serde_json::Value>,
    /// Errors if any
    pub errors: Option<Vec<RelayError>>,
}

impl RelayResponse {
    /// Parses a Relay response from JSON
    pub fn from_json(json: &str) -> Result<Self, GhostError> {
        serde_json::from_str(json).map_err(|e| GhostError::ParseError(e.to_string()))
    }

    /// Parses from a serde_json::Value
    pub fn from_value(value: serde_json::Value) -> Result<Self, GhostError> {
        serde_json::from_value(value).map_err(|e| GhostError::ParseError(e.to_string()))
    }

    /// Checks if response has errors
    pub fn has_errors(&self) -> bool {
        self.errors.as_ref().map(|e| !e.is_empty()).unwrap_or(false)
    }

    /// Extracts data from response
    pub fn extract_data(&self) -> Option<&serde_json::Value> {
        self.data.as_ref()
    }

    /// Get first error message
    pub fn first_error(&self) -> Option<&str> {
        self.errors.as_ref()?.first().map(|e| e.message.as_str())
    }
}

/// Relay error structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayError {
    /// Error message
    pub message: String,
    /// Error code
    pub code: Option<i32>,
    /// Error path in query
    pub path: Option<Vec<String>>,
    /// Error extensions
    pub extensions: Option<RelayErrorExtensions>,
}

impl RelayError {
    /// Converts to GhostError
    pub fn to_ghost_error(&self) -> GhostError {
        GhostError::AdapterError(self.message.clone())
    }
}

/// Relay error extensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayErrorExtensions {
    /// Error classification
    pub classification: Option<String>,
    /// Error severity
    pub severity: Option<String>,
    /// HTTP status code
    pub status: Option<u16>,
}

/// Relay pagination cursor
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayCursor {
    /// Cursor string
    pub cursor: Option<String>,
    /// Has next page
    pub has_next_page: Option<bool>,
    /// Has previous page
    pub has_previous_page: Option<bool>,
    /// Start cursor
    pub start_cursor: Option<String>,
    /// End cursor
    pub end_cursor: Option<String>,
}

#[allow(dead_code)]
impl RelayCursor {
    /// Parses cursor from response
    pub fn from_value(value: &serde_json::Value) -> Option<Self> {
        serde_json::from_value(value.clone()).ok()
    }

    /// Returns the next cursor
    pub fn next(&self) -> Option<&str> {
        self.cursor.as_deref()
    }

    /// Check if there are more pages
    pub fn has_more(&self) -> bool {
        self.has_next_page.unwrap_or(false)
    }
}

/// Threads-specific GraphQL query IDs
pub struct ThreadsQueries;

impl ThreadsQueries {
    /// User profile query ID
    pub const USER_PROFILE: &'static str = "barcelona_user_profile";

    /// Post detail query ID
    pub const POST_DETAIL: &'static str = "barcelona_post_detail";

    /// Timeline query ID
    pub const TIMELINE: &'static str = "barcelona_timeline";

    /// Search query ID
    pub const SEARCH: &'static str = "barcelona_search";

    /// Returns the GraphQL base URL
    pub fn base_url() -> &'static str {
        "https://www.threads.net/api/graphql"
    }

    /// Builds a GraphQL request
    pub fn build_request(query_id: &str, variables: &serde_json::Value) -> String {
        serde_json::to_string(&serde_json::json!({
            "query_id": query_id,
            "variables": variables
        }))
        .unwrap_or_default()
    }

    /// Build variables for user profile query
    pub fn user_profile_vars(user_id: &str) -> serde_json::Value {
        serde_json::json!({
            "userID": user_id,
            "__relay_internal__pv__BarcelonaProfileDetailedInfoSectionDeferredViewActionsrelayprovider": true
        })
    }

    /// Build variables for post detail query
    pub fn post_detail_vars(post_id: &str) -> serde_json::Value {
        serde_json::json!({
            "postID": post_id
        })
    }

    /// Build variables for timeline query
    pub fn timeline_vars(cursor: Option<&str>) -> serde_json::Value {
        if let Some(c) = cursor {
            serde_json::json!({
                "cursor": c,
                "latest_timestamp": null
            })
        } else {
            serde_json::json!({
                "latest_timestamp": null
            })
        }
    }
}

/// Threads request headers
pub struct ThreadsHeaders;

impl ThreadsHeaders {
    /// Get default headers for requests
    pub fn default_headers() -> Vec<(&'static str, &'static str)> {
        vec![
            ("accept", "*/*"),
            ("accept-language", "en-US,en;q=0.9"),
            ("content-type", "application/x-www-form-urlencoded"),
            ("origin", "https://www.threads.net"),
            ("referer", "https://www.threads.net/"),
        ]
    }

    /// Get headers with authentication
    pub fn with_auth(lsd_token: &str, session_id: &str) -> Vec<(&'static str, String)> {
        vec![
            ("accept", "*/*".to_string()),
            ("accept-language", "en-US,en;q=0.9".to_string()),
            ("content-type", "application/x-www-form-urlencoded".to_string()),
            ("x-fb-lsd", lsd_token.to_string()),
            ("cookie", format!("sessionid={}", session_id)),
        ]
    }
}

/// Threads request builder
pub struct ThreadsRequestBuilder {
    query_id: String,
    variables: serde_json::Value,
    lsd_token: Option<String>,
    session_id: Option<String>,
}

impl ThreadsRequestBuilder {
    /// Create a new request builder
    pub fn new(query_id: impl Into<String>) -> Self {
        Self {
            query_id: query_id.into(),
            variables: serde_json::Value::Null,
            lsd_token: None,
            session_id: None,
        }
    }

    /// Set variables
    pub fn variables(mut self, vars: serde_json::Value) -> Self {
        self.variables = vars;
        self
    }

    /// Set LSD token
    pub fn lsd_token(mut self, token: impl Into<String>) -> Self {
        self.lsd_token = Some(token.into());
        self
    }

    /// Set session ID
    pub fn session_id(mut self, id: impl Into<String>) -> Self {
        self.session_id = Some(id.into());
        self
    }

    /// Build the request body
    pub fn build_body(&self) -> String {
        let mut body = format!("variables={}&__a=1&__hsi=1", 
            urlencoding::encode(&serde_json::to_string(&self.variables).unwrap_or_default())
        );

        if let Some(ref lsd) = self.lsd_token {
            body.push_str(&format!("&lsd={}", urlencoding::encode(lsd)));
        }

        body
    }

    /// Get request headers
    pub fn build_headers(&self) -> Vec<(&'static str, String)> {
        let mut headers = vec![
            ("content-type", "application/x-www-form-urlencoded".to_string()),
        ];

        if let Some(ref lsd) = self.lsd_token {
            headers.push(("x-fb-lsd", lsd.clone()));
        }

        if let Some(ref session) = self.session_id {
            headers.push(("cookie", format!("sessionid={}", session)));
        }

        headers
    }

    /// Get the URL for this request
    pub fn build_url(&self) -> String {
        format!("{}/?query_id={}", ThreadsQueries::base_url(), self.query_id)
    }
}

/// URL encoding module (simple implementation)
mod urlencoding {
    use super::percent_encoding::{percent_encode, NON_ALPHANUMERIC};
    
    pub fn encode(s: &str) -> String {
        percent_encode(s.as_bytes(), NON_ALPHANUMERIC).to_string()
    }
}

// Add percent_encoding dependency inline for simplicity
mod percent_encoding {
    pub const NON_ALPHANUMERIC: &AsciiSet = &AsciiSet::new()
        .add_range(0x00..0x20)  // Controls
        .add(b' ').add(b'"').add(b'#').add(b'<').add(b'>').add(b'`')
        .add(b'?').add(b'{').add(b'}').add(b'%').add(b'&').add(b'=').add(b'+');

    pub struct AsciiSet {
        bits: [u64; 4],
    }

    impl AsciiSet {
        pub const fn new() -> Self {
            Self { bits: [0; 4] }
        }

        pub const fn add(mut self, byte: u8) -> Self {
            self.bits[byte as usize / 64] |= 1 << (byte as usize % 64);
            self
        }

        pub const fn add_range(mut self, range: std::ops::Range<u8>) -> Self {
            let mut i = range.start;
            while i < range.end {
                self.bits[i as usize / 64] |= 1 << (i as usize % 64);
                i += 1;
            }
            self
        }

        pub const fn contains(&self, byte: u8) -> bool {
            (self.bits[byte as usize / 64] & (1 << (byte as usize % 64))) != 0
        }
    }

    pub fn percent_encode<'a>(input: &'a [u8], set: &'a AsciiSet) -> PercentEncode<'a> {
        PercentEncode { bytes: input.iter(), set }
    }

    pub struct PercentEncode<'a> {
        bytes: std::slice::Iter<'a, u8>,
        set: &'a AsciiSet,
    }

    impl<'a> std::fmt::Display for PercentEncode<'a> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            for &byte in self.bytes.clone() {
                if self.set.contains(byte) {
                    write!(f, "%{:02X}", byte)?;
                } else {
                    write!(f, "{}", byte as char)?;
                }
            }
            Ok(())
        }
    }
}
