//! Relay-style GraphQL response parsing for Threads

use ghost_schema::{GhostError, GhostPost, GhostUser, GhostMedia, MediaType, Platform};

/// Parses a user from Relay response
pub fn parse_user(data: &serde_json::Value) -> Result<GhostUser, GhostError> {
    // TODO: Implement Relay user parsing
    let user_data = data
        .get("data")
        .or_else(|| data.get("user"))
        .or_else(|| data.get("result"))
        .ok_or_else(|| GhostError::AdapterError("Missing user data in Relay response".into()))?;

    let user = GhostUser {
        id: user_data
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        platform: Platform::Threads,
        username: user_data
            .get("username")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        display_name: user_data.get("name").and_then(|v| v.as_str()).map(String::from),
        bio: user_data.get("biography").and_then(|v| v.as_str()).map(String::from),
        avatar_url: user_data
            .get("profile_picture_url")
            .and_then(|v| v.as_str())
            .map(String::from),
        banner_url: user_data
            .get("header_image_url")
            .and_then(|v| v.as_str())
            .map(String::from),
        followers_count: user_data.get("followers_count").and_then(|v| v.as_u64()),
        following_count: user_data.get("following_count").and_then(|v| v.as_u64()),
        posts_count: user_data.get("post_count").and_then(|v| v.as_u64()),
        is_verified: user_data.get("is_verified").and_then(|v| v.as_bool()),
        is_private: user_data.get("is_private").and_then(|v| v.as_bool()),
        created_at: None,
        raw_metadata: Some(user_data.clone()),
    };

    Ok(user)
}

/// Parses a post from Relay response
pub fn parse_post(data: &serde_json::Value) -> Result<GhostPost, GhostError> {
    // TODO: Implement Relay post parsing
    let post_data = data
        .get("data")
        .or_else(|| data.get("post"))
        .or_else(|| data.get("result"))
        .ok_or_else(|| GhostError::AdapterError("Missing post data in Relay response".into()))?;

    let post = GhostPost {
        id: post_data
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        platform: Platform::Threads,
        text: post_data
            .get("text")
            .or_else(|| post_data.get("caption"))
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        author: parse_relay_author(post_data)?,
        media: parse_relay_media(post_data)?,
        created_at: post_data
            .get("timestamp")
            .or_else(|| post_data.get("taken_at"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0),
        like_count: post_data.get("like_count").and_then(|v| v.as_u64()),
        repost_count: post_data.get("repost_count").and_then(|v| v.as_u64()),
        reply_count: post_data.get("reply_count").and_then(|v| v.as_u64()),
        in_reply_to: post_data
            .get("reply_to_id")
            .and_then(|v| v.as_str())
            .map(String::from),
        quoted_post: None,
        raw_metadata: Some(post_data.clone()),
    };

    Ok(post)
}

/// Parses author from Relay post
fn parse_relay_author(post_data: &serde_json::Value) -> Result<GhostUser, GhostError> {
    // TODO: Implement Relay author parsing
    if let Some(author) = post_data.get("owner").or_else(|| post_data.get("author")) {
        parse_user(author)
    } else {
        Ok(GhostUser::default())
    }
}

/// Parses media from Relay post
fn parse_relay_media(post_data: &serde_json::Value) -> Result<Vec<GhostMedia>, GhostError> {
    // TODO: Implement Relay media parsing
    let mut media = Vec::new();

    // Check for carousel
    if let Some(carousel) = post_data.get("carousel_media") {
        if let Some(items) = carousel.as_array() {
            for item in items {
                if let Some(m) = parse_relay_media_item(item) {
                    media.push(m);
                }
            }
        }
    }

    // Check for single image
    if media.is_empty() {
        if let Some(m) = parse_relay_media_item(post_data) {
            media.push(m);
        }
    }

    Ok(media)
}

/// Parses a Relay media item
fn parse_relay_media_item(item: &serde_json::Value) -> Option<GhostMedia> {
    // TODO: Implement Relay media item parsing
    let media_type = if item.get("video_url").is_some() {
        MediaType::Video
    } else if item.get("image_url").is_some() || item.get("image_versions").is_some() {
        MediaType::Image
    } else {
        MediaType::Unknown
    };

    let url = item
        .get("video_url")
        .or_else(|| item.get("image_url"))
        .or_else(|| item.get("image_versions"))
        .and_then(|u| u.as_str())
        .unwrap_or_default();

    Some(GhostMedia::new(media_type, url))
}

/// Parses multiple threads from Relay response
pub fn parse_threads(data: &serde_json::Value) -> Result<Vec<GhostPost>, GhostError> {
    // TODO: Implement Relay threads parsing
    let mut posts = Vec::new();

    // Handle edges (Relay connection pattern)
    if let Some(edges) = data.get("edges").and_then(|e| e.as_array()) {
        for edge in edges {
            if let Some(node) = edge.get("node") {
                let post = parse_post(node)?;
                posts.push(post);
            }
        }
    }

    // Handle items array
    if let Some(items) = data.get("items").and_then(|i| i.as_array()) {
        for item in items {
            let post = parse_post(item)?;
            posts.push(post);
        }
    }

    Ok(posts)
}

/// Parses a Relay connection (paginated list)
pub fn parse_connection<T>(
    data: &serde_json::Value,
    parser: impl Fn(&serde_json::Value) -> Result<T, GhostError>,
) -> Result<(Vec<T>, Option<String>), GhostError> {
    // TODO: Implement Relay connection parsing
    let mut items = Vec::new();
    let mut cursor = None;

    // Extract items from edges
    if let Some(edges) = data.get("edges").and_then(|e| e.as_array()) {
        for edge in edges {
            if let Some(node) = edge.get("node") {
                let item = parser(node)?;
                items.push(item);
            }
        }
    }

    // Extract cursor from page_info
    if let Some(page_info) = data.get("page_info") {
        cursor = page_info
            .get("end_cursor")
            .and_then(|c| c.as_str())
            .map(String::from);
    }

    Ok((items, cursor))
}

/// Relay page info structure
#[derive(Debug, Clone)]
pub struct PageInfo {
    /// Whether there is a next page
    pub has_next_page: bool,
    /// Whether there is a previous page
    pub has_previous_page: bool,
    /// Cursor for next page
    pub start_cursor: Option<String>,
    /// Cursor for previous page
    pub end_cursor: Option<String>,
}

impl PageInfo {
    /// Parses page info from Relay response
    pub fn parse(data: &serde_json::Value) -> Option<Self> {
        // TODO: Implement page info parsing
        Some(Self {
            has_next_page: data.get("has_next_page").and_then(|v| v.as_bool()).unwrap_or(false),
            has_previous_page: data.get("has_previous_page").and_then(|v| v.as_bool()).unwrap_or(false),
            start_cursor: data.get("start_cursor").and_then(|v| v.as_str()).map(String::from),
            end_cursor: data.get("end_cursor").and_then(|v| v.as_str()).map(String::from),
        })
    }
}
