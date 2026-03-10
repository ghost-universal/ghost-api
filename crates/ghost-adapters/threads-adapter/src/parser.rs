//! Threads content parser
//!
//! Types imported from ghost-schema - the single source of truth.

use ghost_schema::{
    GhostError, GhostPost, GhostUser, GhostMedia, MediaType,
    Platform, ThreadsUserMetadata, ThreadsPostMetadata, ThreadsPostType,
};

/// Parses a post from Threads
pub struct PostParser;

impl PostParser {
    /// Creates a new post parser
    pub fn new() -> Self {
        Self
    }

    /// Parses post data into GhostPost
    pub fn parse(&self, data: &serde_json::Value) -> Result<GhostPost, GhostError> {
        // Get post ID
        let id = data.get("id")
            .or_else(|| data.get("pk"))
            .or_else(|| data.get("post_id"))
            .and_then(|v| v.as_str())
            .or_else(|| data.get("pk").and_then(|p| p.as_u64()).map(|p| p.to_string()))
            .ok_or_else(|| GhostError::ParseError("Missing post ID".into()))?;

        // Get text content
        let text = data.get("text")
            .or_else(|| data.get("caption"))
            .or_else(|| data.get("content"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // Parse author
        let author = self.parse_author(data)?;

        // Parse media
        let media = self.extract_media(data)?;

        // Parse metrics
        let metrics = self.parse_metrics(data);

        // Parse created_at
        let created_at = data.get("taken_at")
            .or_else(|| data.get("created_at"))
            .or_else(|| data.get("timestamp"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        Ok(GhostPost {
            id,
            platform: Platform::Threads,
            text,
            author,
            media,
            created_at,
            like_count: metrics.likes,
            repost_count: metrics.reposts,
            reply_count: metrics.replies,
            view_count: metrics.views,
            quote_count: metrics.quotes,
            in_reply_to: None,
            quoted_post: None,
            raw_metadata: Some(data.clone()),
        })
    }

    /// Parse author from post data
    fn parse_author(&self, data: &serde_json::Value) -> Result<GhostUser, GhostError> {
        // Try user field
        if let Some(user) = data.get("user") {
            return UserParser::new().parse(user);
        }

        // Try owner field
        if let Some(owner) = data.get("owner") {
            return UserParser::new().parse(owner);
        }

        // Try author field
        if let Some(author) = data.get("author") {
            return UserParser::new().parse(author);
        }

        // Try caption_user
        if let Some(user) = data.get("caption_user") {
            return UserParser::new().parse(user);
        }

        // Create minimal user from username if available
        let username = data.get("username")
            .or_else(|| data.get("user_name"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(GhostUser::new("", Platform::Threads, username))
    }

    /// Extracts media from post
    pub fn extract_media(&self, data: &serde_json::Value) -> Result<Vec<GhostMedia>, GhostError> {
        let mut media_list = Vec::new();

        // Try video versions
        if let Some(video_versions) = data.get("video_versions").and_then(|v| v.as_array()) {
            if let Some(video) = video_versions.first() {
                if let Ok(gm) = self.parse_video_media(video) {
                    media_list.push(gm);
                }
            }
        }

        // Try image versions
        if media_list.is_empty() {
            if let Some(image_versions) = data.get("image_versions2").and_then(|v| v.as_array()) {
                if let Some(image) = image_versions.first() {
                    if let Ok(gm) = self.parse_image_media(image) {
                        media_list.push(gm);
                    }
                }
            }
        }

        // Try carousel media
        if media_list.is_empty() {
            if let Some(carousel) = data.get("carousel_media").and_then(|c| c.as_array()) {
                for item in carousel {
                    // Try video first
                    if let Some(video_versions) = item.get("video_versions").and_then(|v| v.as_array()) {
                        if let Some(video) = video_versions.first() {
                            if let Ok(gm) = self.parse_video_media(video) {
                                media_list.push(gm);
                                continue;
                            }
                        }
                    }
                    // Fall back to image
                    if let Some(image_versions) = item.get("image_versions2").and_then(|v| v.as_array()) {
                        if let Some(image) = image_versions.first() {
                            if let Ok(gm) = self.parse_image_media(image) {
                                media_list.push(gm);
                            }
                        }
                    }
                }
            }
        }

        // Try media field (API format)
        if media_list.is_empty() {
            if let Some(media_url) = data.get("media_url").and_then(|u| u.as_str()) {
                let media_type = data.get("media_type")
                    .and_then(|t| t.as_str())
                    .unwrap_or("IMAGE");
                
                media_list.push(GhostMedia {
                    id: String::new(),
                    media_type: if media_type == "VIDEO" { MediaType::Video } else { MediaType::Image },
                    url: media_url.to_string(),
                    preview_url: data.get("thumbnail_url").and_then(|u| u.as_str()).map(|s| s.to_string()),
                    width: None,
                    height: None,
                    duration_secs: None,
                    alt_text: None,
                    content_type: None,
                    size_bytes: None,
                });
            }
        }

        Ok(media_list)
    }

    /// Parse video media
    fn parse_video_media(&self, video: &serde_json::Value) -> Result<GhostMedia, GhostError> {
        let url = video.get("url")
            .and_then(|u| u.as_str())
            .ok_or_else(|| GhostError::ParseError("Missing video URL".into()))?;

        Ok(GhostMedia {
            id: String::new(),
            media_type: MediaType::Video,
            url: url.to_string(),
            preview_url: None,
            width: video.get("width").and_then(|w| w.as_u64()).map(|w| w as u32),
            height: video.get("height").and_then(|h| h.as_u64()).map(|h| h as u32),
            duration_secs: video.get("duration").and_then(|d| d.as_f64()),
            alt_text: None,
            content_type: Some("video/mp4".to_string()),
            size_bytes: None,
        })
    }

    /// Parse image media
    fn parse_image_media(&self, image: &serde_json::Value) -> Result<GhostMedia, GhostError> {
        let url = image.get("url")
            .and_then(|u| u.as_str())
            .ok_or_else(|| GhostError::ParseError("Missing image URL".into()))?;

        Ok(GhostMedia {
            id: String::new(),
            media_type: MediaType::Image,
            url: url.to_string(),
            preview_url: None,
            width: image.get("width").and_then(|w| w.as_u64()).map(|w| w as u32),
            height: image.get("height").and_then(|h| h.as_u64()).map(|h| h as u32),
            duration_secs: None,
            alt_text: None,
            content_type: Some("image/jpeg".to_string()),
            size_bytes: None,
        })
    }

    /// Parse metrics from post
    fn parse_metrics(&self, data: &serde_json::Value) -> PostMetrics {
        PostMetrics {
            likes: data.get("like_count")
                .or_else(|| data.get("likes"))
                .or_else(|| data.get("like_and_impulse_counts").and_then(|c| c.get("likes")))
                .and_then(|v| v.as_u64()),
            replies: data.get("text_post_app_comment_count")
                .or_else(|| data.get("reply_count"))
                .or_else(|| data.get("comment_count"))
                .and_then(|v| v.as_u64()),
            reposts: data.get("repost_count")
                .or_else(|| data.get("reshare_count"))
                .and_then(|v| v.as_u64()),
            quotes: data.get("quote_count")
                .and_then(|v| v.as_u64()),
            views: data.get("play_count")
                .or_else(|| data.get("view_count"))
                .and_then(|v| v.as_u64()),
        }
    }

    /// Extracts metadata
    pub fn extract_metadata(&self, data: &serde_json::Value) -> Result<ThreadsPostMetadata, GhostError> {
        Ok(ThreadsPostMetadata {
            post_type: self.determine_post_type(data),
            has_audio: data.get("has_audio").and_then(|a| a.as_bool()).unwrap_or(false),
            is_reel: data.get("product_type").and_then(|p| p.as_str()) == Some("threads_reel"),
            lang: None,
            reply_audience: None,
            hide_status: None,
            shortcode: data.get("code").and_then(|c| c.as_str()).map(|s| s.to_string()),
            media_product_type: data.get("product_type").and_then(|p| p.as_str()).map(|s| s.to_string()),
            is_shared_to_feed: data.get("is_shared_to_feed").and_then(|s| s.as_bool()),
            hashtags: vec![],
            mentions: vec![],
            links: vec![],
        })
    }

    /// Determines post type
    pub fn determine_post_type(&self, data: &serde_json::Value) -> ThreadsPostType {
        // Check for video
        if data.get("video_versions").is_some() {
            return if data.get("product_type").and_then(|p| p.as_str()) == Some("threads_reel") {
                ThreadsPostType::Reel
            } else {
                ThreadsPostType::Video
            };
        }

        // Check for carousel
        if data.get("carousel_media").is_some() {
            return ThreadsPostType::Carousel;
        }

        // Check for image
        if data.get("image_versions2").is_some() {
            return ThreadsPostType::Image;
        }

        // Check media type field
        if let Some(media_type) = data.get("media_type").and_then(|t| t.as_str()) {
            return ThreadsPostType::from_api(media_type);
        }

        ThreadsPostType::Text
    }
}

impl Default for PostParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Parsed post metrics
struct PostMetrics {
    likes: Option<u64>,
    replies: Option<u64>,
    reposts: Option<u64>,
    quotes: Option<u64>,
    views: Option<u64>,
}

/// Parses a user profile from Threads
pub struct UserParser;

impl UserParser {
    /// Creates a new user parser
    pub fn new() -> Self {
        Self
    }

    /// Parses user data into GhostUser
    pub fn parse(&self, data: &serde_json::Value) -> Result<GhostUser, GhostError> {
        // Get ID
        let id = data.get("pk")
            .and_then(|p| p.as_str())
            .or_else(|| data.get("id").and_then(|i| i.as_str()))
            .or_else(|| data.get("pk").and_then(|p| p.as_u64()).map(|p| p.to_string()))
            .ok_or_else(|| GhostError::ParseError("Missing user ID".into()))?;

        // Get username
        let username = data.get("username")
            .or_else(|| data.get("handle"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| GhostError::ParseError("Missing username".into()))?;

        // Get display name
        let display_name = data.get("full_name")
            .or_else(|| data.get("name"))
            .or_else(|| data.get("display_name"))
            .and_then(|n| n.as_str())
            .map(|s| s.to_string());

        // Get bio
        let bio = data.get("biography")
            .or_else(|| data.get("bio"))
            .or_else(|| data.get("description"))
            .and_then(|b| b.as_str())
            .map(|s| s.to_string());

        // Get avatar
        let avatar_url = data.get("profile_pic_url")
            .or_else(|| data.get("profile_picture_url"))
            .or_else(|| data.get("threads_profile_picture_url"))
            .and_then(|u| u.as_str())
            .map(|s| s.to_string());

        // Get verification status
        let is_verified = data.get("is_verified")
            .or_else(|| data.get("verified"))
            .and_then(|v| v.as_bool());

        // Get private status
        let is_private = data.get("is_private")
            .or_else(|| data.get("private"))
            .and_then(|p| p.as_bool());

        // Get follower count
        let followers_count = data.get("follower_count")
            .or_else(|| data.get("followers_count"))
            .and_then(|v| v.as_u64());

        Ok(GhostUser {
            id,
            platform: Platform::Threads,
            username,
            display_name,
            bio,
            avatar_url,
            banner_url: None,
            profile_url: Some(format!("https://www.threads.net/@{}", username)),
            location: None,
            website: None,
            followers_count,
            following_count: None,
            posts_count: None,
            is_verified,
            is_private,
            is_bot: None,
            created_at: None,
            raw_metadata: Some(data.clone()),
        })
    }

    /// Extracts metadata
    pub fn extract_metadata(&self, data: &serde_json::Value) -> Result<ThreadsUserMetadata, GhostError> {
        Ok(ThreadsUserMetadata {
            is_verified: data.get("is_verified").and_then(|v| v.as_bool()).unwrap_or(false),
            is_business_account: data.get("is_business_account").and_then(|b| b.as_bool()).unwrap_or(false),
            is_creator_account: data.get("is_creator_account").and_then(|c| c.as_bool()).unwrap_or(false),
            has_linked_instagram: data.get("has_linked_instagram").and_then(|h| h.as_bool()).unwrap_or(false),
            profile_deep_link: data.get("profile_deep_link").and_then(|l| l.as_str()).map(|s| s.to_string()),
            bio_links: vec![],
            followers_count: data.get("follower_count").and_then(|f| f.as_u64()),
            following_count: None,
        })
    }
}

impl Default for UserParser {
    fn default() -> Self {
        Self::new()
    }
}
