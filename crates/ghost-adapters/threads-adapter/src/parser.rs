//! Parsing utilities for Threads responses

use ghost_schema::{GhostError, GhostPost, GhostUser, GhostMedia, MediaType, Platform};

/// Parses a thread (multiple posts in a conversation)
pub fn parse_thread(data: &serde_json::Value) -> Result<Vec<GhostPost>, GhostError> {
    // TODO: Implement thread parsing
    let mut posts = Vec::new();

    if let Some(items) = data.get("thread_items").and_then(|t| t.as_array()) {
        for item in items {
            if let Some(post) = parse_thread_item(item)? {
                posts.push(post);
            }
        }
    }

    Ok(posts)
}

/// Parses a single thread item
fn parse_thread_item(item: &serde_json::Value) -> Result<Option<GhostPost>, GhostError> {
    // TODO: Implement thread item parsing
    if let Some(post) = item.get("post") {
        let parsed = parse_single_post(post)?;
        Ok(Some(parsed))
    } else {
        Ok(None)
    }
}

/// Parses a single post
pub fn parse_single_post(data: &serde_json::Value) -> Result<GhostPost, GhostError> {
    // TODO: Implement single post parsing
    let post = GhostPost {
        id: data
            .get("id")
            .or_else(|| data.get("pk"))
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        platform: Platform::Threads,
        text: data
            .get("text")
            .or_else(|| data.get("caption"))
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        author: parse_post_author(data)?,
        media: parse_post_media(data)?,
        created_at: data
            .get("taken_at")
            .or_else(|| data.get("timestamp"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0),
        like_count: data.get("like_count").and_then(|v| v.as_u64()),
        repost_count: data.get("repost_count").and_then(|v| v.as_u64()),
        reply_count: data.get("reply_count").or_else(|| data.get("text_post_app_info"))
            .and_then(|t| t.get("reply_count"))
            .and_then(|v| v.as_u64()),
        in_reply_to: data
            .get("reply_to")
            .and_then(|r| r.get("id"))
            .and_then(|v| v.as_str())
            .map(String::from),
        quoted_post: parse_quoted_post(data)?,
        raw_metadata: Some(data.clone()),
    };

    Ok(post)
}

/// Parses the author from a post
fn parse_post_author(data: &serde_json::Value) -> Result<GhostUser, GhostError> {
    // TODO: Implement post author parsing
    let user_data = data
        .get("user")
        .or_else(|| data.get("owner"));

    if let Some(user) = user_data {
        parse_single_user(user)
    } else {
        Ok(GhostUser::default())
    }
}

/// Parses a single user
pub fn parse_single_user(data: &serde_json::Value) -> Result<GhostUser, GhostError> {
    // TODO: Implement single user parsing
    let user = GhostUser {
        id: data
            .get("id")
            .or_else(|| data.get("pk"))
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        platform: Platform::Threads,
        username: data
            .get("username")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        display_name: data.get("full_name").and_then(|v| v.as_str()).map(String::from),
        bio: data.get("biography").and_then(|v| v.as_str()).map(String::from),
        avatar_url: data
            .get("profile_pic_url")
            .and_then(|v| v.as_str())
            .map(String::from),
        banner_url: None,
        followers_count: data.get("follower_count").and_then(|v| v.as_u64()),
        following_count: data.get("following_count").and_then(|v| v.as_u64()),
        posts_count: data.get("media_count").and_then(|v| v.as_u64()),
        is_verified: data.get("is_verified").and_then(|v| v.as_bool()),
        is_private: data.get("is_private").and_then(|v| v.as_bool()),
        created_at: None,
        raw_metadata: Some(data.clone()),
    };

    Ok(user)
}

/// Parses media from a post
fn parse_post_media(data: &serde_json::Value) -> Result<Vec<GhostMedia>, GhostError> {
    // TODO: Implement post media parsing
    let mut media = Vec::new();

    // Check for carousel media
    if let Some(carousel) = data.get("carousel_media").and_then(|c| c.as_array()) {
        for item in carousel {
            if let Some(m) = parse_media_item(item) {
                media.push(m);
            }
        }
    }

    // Check for single media
    if media.is_empty() {
        if let Some(m) = parse_media_item(data) {
            media.push(m);
        }
    }

    Ok(media)
}

/// Parses a single media item
fn parse_media_item(item: &serde_json::Value) -> Option<GhostMedia> {
    // TODO: Implement media item parsing
    let media_type = match item.get("media_type").and_then(|t| t.as_u64()) {
        Some(1) => MediaType::Image,
        Some(2) => MediaType::Video,
        Some(8) => MediaType::Gif,
        _ => MediaType::Unknown,
    };

    let url = item
        .get("image_versions2")
        .and_then(|i| i.get("url"))
        .or_else(|| item.get("video_url"))
        .or_else(|| item.get("url"))
        .and_then(|u| u.as_str())
        .unwrap_or_default();

    Some(GhostMedia::new(media_type, url))
}

/// Parses quoted post
fn parse_quoted_post(data: &serde_json::Value) -> Result<Option<Box<GhostPost>>, GhostError> {
    // TODO: Implement quoted post parsing
    if let Some(quoted) = data.get("quoted_post") {
        let post = parse_single_post(quoted)?;
        Ok(Some(Box::new(post)))
    } else {
        Ok(None)
    }
}

/// Parses a timeline/feed
pub fn parse_timeline(data: &serde_json::Value) -> Result<Vec<GhostPost>, GhostError> {
    // TODO: Implement timeline parsing
    let mut posts = Vec::new();

    if let Some(items) = data.get("items").and_then(|i| i.as_array()) {
        for item in items {
            if let Some(post) = item.get("post") {
                let parsed = parse_single_post(post)?;
                posts.push(parsed);
            }
        }
    }

    Ok(posts)
}

/// Parses search results
pub fn parse_search_results(data: &serde_json::Value) -> Result<Vec<GhostPost>, GhostError> {
    // TODO: Implement search results parsing
    let mut posts = Vec::new();

    if let Some(results) = data.get("results").and_then(|r| r.as_array()) {
        for result in results {
            if let Some(post) = parse_search_result_item(result)? {
                posts.push(post);
            }
        }
    }

    Ok(posts)
}

/// Parses a search result item
fn parse_search_result_item(item: &serde_json::Value) -> Result<Option<GhostPost>, GhostError> {
    // TODO: Implement search result item parsing
    if let Some(thread) = item.get("thread") {
        let posts = parse_thread(thread)?;
        Ok(posts.into_iter().next())
    } else if item.get("post").is_some() {
        let post = parse_single_post(item)?;
        Ok(Some(post))
    } else {
        Ok(None)
    }
}

/// Extracts cursor for pagination
pub fn extract_cursor(data: &serde_json::Value) -> Option<String> {
    // TODO: Implement cursor extraction
    data.get("paging_info")
        .and_then(|p| p.get("max_id"))
        .or_else(|| data.get("cursor"))
        .or_else(|| data.get("end_cursor"))
        .and_then(|v| v.as_str())
        .map(String::from)
}
