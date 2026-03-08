//! GraphQL response parsing for X API

use ghost_schema::{GhostError, GhostPost, GhostUser, GhostMedia, MediaType, Platform};

/// Parses a user result from GraphQL
pub fn parse_user_result(data: &serde_json::Value) -> Result<GhostUser, GhostError> {
    // TODO: Implement GraphQL user parsing
    let legacy = data
        .get("legacy")
        .ok_or_else(|| GhostError::AdapterError("Missing legacy field in user data".into()))?;

    let user = GhostUser {
        id: data
            .get("rest_id")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        platform: Platform::X,
        username: legacy
            .get("screen_name")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        display_name: legacy.get("name").and_then(|v| v.as_str()).map(String::from),
        bio: legacy.get("description").and_then(|v| v.as_str()).map(String::from),
        avatar_url: legacy
            .get("profile_image_url_https")
            .and_then(|v| v.as_str())
            .map(String::from),
        banner_url: legacy
            .get("profile_banner_url")
            .and_then(|v| v.as_str())
            .map(String::from),
        followers_count: legacy.get("followers_count").and_then(|v| v.as_u64()),
        following_count: legacy.get("friends_count").and_then(|v| v.as_u64()),
        posts_count: legacy.get("statuses_count").and_then(|v| v.as_u64()),
        is_verified: legacy.get("verified").and_then(|v| v.as_bool()),
        is_private: legacy.get("protected").and_then(|v| v.as_bool()),
        created_at: legacy
            .get("created_at")
            .and_then(|v| v.as_str())
            .and_then(parse_twitter_date),
        raw_metadata: Some(data.clone()),
    };

    Ok(user)
}

/// Parses a tweet result from GraphQL
pub fn parse_tweet_result(data: &serde_json::Value) -> Result<GhostPost, GhostError> {
    // TODO: Implement GraphQL tweet parsing
    let legacy = data
        .get("legacy")
        .ok_or_else(|| GhostError::AdapterError("Missing legacy field in tweet data".into()))?;

    let post = GhostPost {
        id: data
            .get("rest_id")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        platform: Platform::X,
        text: legacy
            .get("full_text")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        author: parse_tweet_author(data)?,
        media: parse_tweet_media(legacy)?,
        created_at: legacy
            .get("created_at")
            .and_then(|v| v.as_str())
            .and_then(parse_twitter_date)
            .unwrap_or(0),
        like_count: legacy.get("favorite_count").and_then(|v| v.as_u64()),
        repost_count: legacy.get("retweet_count").and_then(|v| v.as_u64()),
        reply_count: legacy.get("reply_count").and_then(|v| v.as_u64()),
        in_reply_to: legacy
            .get("in_reply_to_status_id_str")
            .and_then(|v| v.as_str())
            .map(String::from),
        quoted_post: parse_quoted_tweet(legacy)?,
        raw_metadata: Some(data.clone()),
    };

    Ok(post)
}

/// Parses the author from a tweet
fn parse_tweet_author(data: &serde_json::Value) -> Result<GhostUser, GhostError> {
    // TODO: Implement tweet author parsing
    let core = data
        .get("core")
        .ok_or_else(|| GhostError::AdapterError("Missing core field in tweet data".into()))?;

    let user_results = core
        .get("user_results")
        .ok_or_else(|| GhostError::AdapterError("Missing user_results in core".into()))?;

    let result = user_results
        .get("result")
        .ok_or_else(|| GhostError::AdapterError("Missing result in user_results".into()))?;

    parse_user_result(result)
}

/// Parses media from a tweet
fn parse_tweet_media(legacy: &serde_json::Value) -> Result<Vec<GhostMedia>, GhostError> {
    // TODO: Implement tweet media parsing
    let entities = legacy.get("entities").and_then(|e| e.get("media"));
    let extended = legacy
        .get("extended_entities")
        .and_then(|e| e.get("media"));

    let media_list = extended.or(entities);

    if let Some(media_array) = media_list.and_then(|m| m.as_array()) {
        let mut media = Vec::new();
        for item in media_array {
            if let Some(m) = parse_media_item(item) {
                media.push(m);
            }
        }
        Ok(media)
    } else {
        Ok(Vec::new())
    }
}

/// Parses a single media item
fn parse_media_item(item: &serde_json::Value) -> Option<GhostMedia> {
    // TODO: Implement media item parsing
    let media_type = match item.get("type").and_then(|t| t.as_str()) {
        Some("photo") => MediaType::Image,
        Some("video") => MediaType::Video,
        Some("animated_gif") => MediaType::Gif,
        _ => MediaType::Unknown,
    };

    let url = item
        .get("media_url_https")
        .and_then(|u| u.as_str())
        .unwrap_or_default();

    Some(GhostMedia::new(media_type, url))
}

/// Parses a quoted tweet
fn parse_quoted_tweet(legacy: &serde_json::Value) -> Result<Option<Box<GhostPost>>, GhostError> {
    // TODO: Implement quoted tweet parsing
    if let Some(quoted) = legacy.get("quoted_status_result") {
        let result = quoted
            .get("result")
            .ok_or_else(|| GhostError::AdapterError("Missing result in quoted_status".into()))?;
        let post = parse_tweet_result(result)?;
        Ok(Some(Box::new(post)))
    } else {
        Ok(None)
    }
}

/// Parses a search timeline from GraphQL
pub fn parse_search_timeline(data: &serde_json::Value) -> Result<Vec<GhostPost>, GhostError> {
    // TODO: Implement search timeline parsing
    let mut posts = Vec::new();

    if let Some(instructions) = data.get("timeline").and_then(|t| t.get("instructions")) {
        if let Some(instructions_array) = instructions.as_array() {
            for instruction in instructions_array {
                if let Some(entries) = instruction.get("entries").and_then(|e| e.as_array()) {
                    for entry in entries {
                        if let Some(post) = parse_timeline_entry(entry)? {
                            posts.push(post);
                        }
                    }
                }
            }
        }
    }

    Ok(posts)
}

/// Parses a timeline entry
fn parse_timeline_entry(entry: &serde_json::Value) -> Result<Option<GhostPost>, GhostError> {
    // TODO: Implement timeline entry parsing
    let content = entry.get("content");

    if let Some(item_content) = content.and_then(|c| c.get("itemContent")) {
        if let Some(tweet_results) = item_content.get("tweet_results") {
            if let Some(result) = tweet_results.get("result") {
                let post = parse_tweet_result(result)?;
                return Ok(Some(post));
            }
        }
    }

    Ok(None)
}

/// Parses Twitter date format
fn parse_twitter_date(date_str: &str) -> Option<i64> {
    // TODO: Implement Twitter date parsing
    // Format: "Wed Oct 10 20:19:24 +0000 2018"
    None
}

/// GraphQL query ID mappings
pub struct GraphQLQueries {
    /// Query IDs for different endpoints
    queries: std::collections::HashMap<String, String>,
}

impl GraphQLQueries {
    /// Creates query ID map
    pub fn new() -> Self {
        // TODO: Implement query ID map construction
        let mut queries = std::collections::HashMap::new();

        // X frequently changes these query IDs
        queries.insert("UserByScreenName".to_string(), "u7wQyGsHwAMSLsFNvuNcTA".to_string());
        queries.insert("UserByRestId".to_string(), "s9ur3DoHhGNaLQrHjWuKSA".to_string());
        queries.insert("TweetResultByRestId".to_string(), "HICJvKl-U5Y0x8VY3J0X5Q".to_string());
        queries.insert("SearchTimeline".to_string(), "nK1dw4oV-DnIlWqyXwCdaQ".to_string());

        Self { queries }
    }

    /// Gets a query ID
    pub fn get(&self, query_name: &str) -> Option<&String> {
        // TODO: Implement query ID retrieval
        self.queries.get(query_name)
    }

    /// Updates a query ID
    pub fn update(&mut self, query_name: impl Into<String>, query_id: impl Into<String>) {
        // TODO: Implement query ID update
        self.queries.insert(query_name.into(), query_id.into());
    }

    /// Loads query IDs from JSON
    pub fn from_json(json: &str) -> Result<Self, GhostError> {
        // TODO: Implement JSON loading
        let queries: std::collections::HashMap<String, String> = serde_json::from_str(json)
            .map_err(|e| GhostError::ParseError(e.to_string()))?;
        Ok(Self { queries })
    }
}

impl Default for GraphQLQueries {
    fn default() -> Self {
        Self::new()
    }
}
