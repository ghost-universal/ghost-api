//! X content parser
//!
//! Types imported from ghost-schema - the single source of truth.

use ghost_schema::{
    GhostError, GhostPost, GhostUser, GhostMedia, MediaType,
    Platform, XUserMetadata, XPostMetadata, XTweetMetrics,
    HashtagEntity, UserMention, UrlEntity,
};

/// Parses a tweet/post from X
pub struct PostParser;

impl PostParser {
    /// Creates a new post parser
    pub fn new() -> Self {
        Self
    }

    /// Parses tweet data into GhostPost
    pub fn parse(&self, data: &serde_json::Value) -> Result<GhostPost, GhostError> {
        // Handle legacy tweet format - extract ID from multiple possible fields
        let id = if let Some(id_str) = data.get("id_str").and_then(|v| v.as_str()) {
            id_str.to_string()
        } else if let Some(id) = data.get("id").and_then(|v| v.as_str()) {
            id.to_string()
        } else if let Some(rest_id) = data.get("rest_id").and_then(|v| v.as_u64()) {
            rest_id.to_string()
        } else {
            return Err(GhostError::ParseError("Missing tweet ID".into()));
        };

        // Get text content
        let text = data.get("text")
            .or_else(|| data.get("full_text"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // Parse author
        let author = self.parse_author(data)?;

        // Parse metrics
        let metrics = self.parse_metrics(data);

        // Parse created_at
        let created_at = data.get("created_at")
            .and_then(|v| v.as_str())
            .and_then(|s| parse_twitter_timestamp(s))
            .unwrap_or(0);

        // Parse in_reply_to
        let in_reply_to = data.get("in_reply_to_status_id_str")
            .or_else(|| data.get("in_reply_to_tweet_id"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Parse media
        let media = self.extract_media(data)?;

        Ok(GhostPost {
            id,
            platform: Platform::X,
            text,
            author,
            media,
            created_at,
            like_count: metrics.map(|m| m.like_count),
            repost_count: metrics.map(|m| m.retweet_count),
            reply_count: metrics.map(|m| m.reply_count),
            view_count: metrics.map(|m| m.impression_count),
            quote_count: metrics.map(|m| m.quote_count),
            in_reply_to,
            quoted_post: None,
            raw_metadata: Some(data.clone()),
        })
    }

    /// Parse author from tweet data
    fn parse_author(&self, data: &serde_json::Value) -> Result<GhostUser, GhostError> {
        // Try core user data first
        if let Some(user) = data.get("core").and_then(|c| c.get("user_results")).and_then(|r| r.get("result")) {
            return UserParser::new().parse(user);
        }

        // Try legacy user format
        if let Some(user) = data.get("user") {
            return UserParser::new().parse(user);
        }

        // Try legacy screen_name
        let username = data.get("user")
            .and_then(|u| u.get("screen_name"))
            .or_else(|| data.get("screen_name"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(GhostUser::new("", Platform::X, username))
    }

    /// Parse metrics from tweet data
    fn parse_metrics(&self, data: &serde_json::Value) -> Option<XTweetMetrics> {
        let legacy = data.get("legacy")?;

        Some(XTweetMetrics {
            like_count: legacy.get("favorite_count").and_then(|v| v.as_u64()).unwrap_or(0),
            retweet_count: legacy.get("retweet_count").and_then(|v| v.as_u64()).unwrap_or(0),
            reply_count: legacy.get("reply_count").and_then(|v| v.as_u64()).unwrap_or(0),
            quote_count: legacy.get("quote_count").and_then(|v| v.as_u64()).unwrap_or(0),
            impression_count: data.get("views")
                .and_then(|v| v.get("count"))
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            bookmark_count: legacy.get("bookmark_count").and_then(|v| v.as_u64()),
        })
    }

    /// Extracts media from tweet
    pub fn extract_media(&self, data: &serde_json::Value) -> Result<Vec<GhostMedia>, GhostError> {
        let mut media_list = Vec::new();

        // Try legacy extended_entities
        if let Some(entities) = data.get("legacy").and_then(|l| l.get("extended_entities")) {
            if let Some(media) = entities.get("media").and_then(|m| m.as_array()) {
                for m in media {
                    if let Ok(gm) = self.parse_media_item(m) {
                        media_list.push(gm);
                    }
                }
            }
        }

        // Try entities.media
        if media_list.is_empty() {
            if let Some(entities) = data.get("entities") {
                if let Some(media) = entities.get("media").and_then(|m| m.as_array()) {
                    for m in media {
                        if let Ok(gm) = self.parse_media_item(m) {
                            media_list.push(gm);
                        }
                    }
                }
            }
        }

        Ok(media_list)
    }

    /// Parse a single media item
    fn parse_media_item(&self, media: &serde_json::Value) -> Result<GhostMedia, GhostError> {
        let media_type = media.get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("photo");

        let ghost_type = match media_type {
            "photo" | "animated_gif" => MediaType::Image,
            "video" => MediaType::Video,
            _ => MediaType::Unknown,
        };

        let url = media.get("media_url_https")
            .or_else(|| media.get("media_url"))
            .and_then(|u| u.as_str())
            .unwrap_or("")
            .to_string();

        // Get video URL if available
        let final_url = if ghost_type == MediaType::Video {
            media.get("video_info")
                .and_then(|vi| vi.get("variants"))
                .and_then(|v| v.as_array())
                .and_then(|vars| {
                    vars.iter()
                        .filter(|v| v.get("type").and_then(|t| t.as_str()) == Some("video/mp4"))
                        .filter_map(|v| v.get("url").and_then(|u| u.as_str()))
                        .next()
                })
                .unwrap_or(&url)
                .to_string()
        } else {
            url
        };

        Ok(GhostMedia {
            id: media.get("id_str")
                .and_then(|i| i.as_str())
                .unwrap_or("")
                .to_string(),
            media_type: ghost_type,
            url: final_url,
            preview_url: media.get("media_url_https")
                .and_then(|u| u.as_str())
                .map(|s| s.to_string()),
            width: media.get("original_info")
                .and_then(|o| o.get("width"))
                .and_then(|w| w.as_u64())
                .map(|w| w as u32),
            height: media.get("original_info")
                .and_then(|o| o.get("height"))
                .and_then(|h| h.as_u64())
                .map(|h| h as u32),
            duration_secs: media.get("video_info")
                .and_then(|vi| vi.get("duration_millis"))
                .and_then(|d| d.as_u64())
                .map(|d| d as f64 / 1000.0),
            alt_text: media.get("ext_alt_text")
                .and_then(|a| a.as_str())
                .map(|s| s.to_string()),
            content_type: None,
            size_bytes: None,
        })
    }

    /// Extracts metadata
    pub fn extract_metadata(&self, data: &serde_json::Value) -> Result<XPostMetadata, GhostError> {
        let legacy = data.get("legacy");
        
        Ok(XPostMetadata {
            source: legacy.and_then(|l| l.get("source")).and_then(|s| s.as_str()).map(|s| s.to_string()),
            lang: legacy.and_then(|l| l.get("lang")).and_then(|l| l.as_str()).map(|s| s.to_string()),
            is_quote_status: legacy.and_then(|l| l.get("is_quote_status")).and_then(|q| q.as_bool()).unwrap_or(false),
            possibly_sensitive: legacy.and_then(|l| l.get("possibly_sensitive")).and_then(|p| p.as_bool()),
            coordinates: None,
            place: None,
            hashtags: self.extract_hashtags(data),
            mentions: self.extract_mentions(data),
            urls: self.extract_urls(data),
            cashtags: vec![],
            conversation_id: legacy.and_then(|l| l.get("conversation_id_str")).and_then(|c| c.as_str()).map(|s| s.to_string()),
            bookmark_count: None,
            edit_history: vec![],
            edit_remaining: None,
            is_note_tweet: false,
            reply_settings: None,
        })
    }

    /// Extract hashtags
    fn extract_hashtags(&self, data: &serde_json::Value) -> Vec<HashtagEntity> {
        let entities = data.get("legacy").and_then(|l| l.get("entities"))
            .or_else(|| data.get("entities"));
        
        entities
            .and_then(|e| e.get("hashtags"))
            .and_then(|h| h.as_array())
            .map(|hashtags| {
                hashtags.iter()
                    .filter_map(|h| {
                        Some(HashtagEntity::new(
                            h.get("text").and_then(|t| t.as_str())?,
                            h.get("indices").and_then(|i| i.as_array())?.get(0)?.as_u64()? as usize,
                            h.get("indices").and_then(|i| i.as_array())?.get(1)?.as_u64()? as usize,
                        ))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Extract mentions
    fn extract_mentions(&self, data: &serde_json::Value) -> Vec<UserMention> {
        let entities = data.get("legacy").and_then(|l| l.get("entities"))
            .or_else(|| data.get("entities"));

        entities
            .and_then(|e| e.get("user_mentions"))
            .and_then(|m| m.as_array())
            .map(|mentions| {
                mentions.iter()
                    .filter_map(|m| {
                        let indices = m.get("indices").and_then(|i| i.as_array())?;
                        Some(UserMention::new(
                            m.get("id_str").and_then(|i| i.as_str())?,
                            m.get("screen_name").and_then(|s| s.as_str())?,
                            indices.get(0)?.as_u64()? as usize,
                            indices.get(1)?.as_u64()? as usize,
                        ))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Extract URLs
    fn extract_urls(&self, data: &serde_json::Value) -> Vec<UrlEntity> {
        let entities = data.get("legacy").and_then(|l| l.get("entities"))
            .or_else(|| data.get("entities"));

        entities
            .and_then(|e| e.get("urls"))
            .and_then(|u| u.as_array())
            .map(|urls| {
                urls.iter()
                    .filter_map(|u| {
                        let indices = u.get("indices").and_then(|i| i.as_array())?;
                        Some(UrlEntity::new(
                            u.get("url").and_then(|u| u.as_str())?,
                            u.get("expanded_url").and_then(|e| e.as_str())?,
                            indices.get(0)?.as_u64()? as usize,
                            indices.get(1)?.as_u64()? as usize,
                        ))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl Default for PostParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Parses a user profile from X
pub struct UserParser;

impl UserParser {
    /// Creates a new user parser
    pub fn new() -> Self {
        Self
    }

    /// Parses user data into GhostUser
    pub fn parse(&self, data: &serde_json::Value) -> Result<GhostUser, GhostError> {
        // Get ID - handle multiple possible formats
        let id = if let Some(id_str) = data.get("id_str").and_then(|v| v.as_str()) {
            id_str.to_string()
        } else if let Some(rest_id) = data.get("rest_id").and_then(|r| r.as_u64()) {
            rest_id.to_string()
        } else if let Some(id) = data.get("id").and_then(|i| i.as_str()) {
            id.to_string()
        } else {
            return Err(GhostError::ParseError("Missing user ID".into()));
        };

        // Get username
        let username = data.get("screen_name")
            .or_else(|| data.get("username"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| GhostError::ParseError("Missing username".into()))?
            .to_string();

        // Get legacy data if available
        let legacy = data.get("legacy");

        // Get display name
        let display_name = legacy.and_then(|l| l.get("name"))
            .or_else(|| data.get("name"))
            .and_then(|n| n.as_str())
            .map(|s| s.to_string());

        // Get description
        let bio = legacy.and_then(|l| l.get("description"))
            .or_else(|| data.get("description"))
            .and_then(|d| d.as_str())
            .map(|s| s.to_string());

        // Get avatar
        let avatar_url = legacy.and_then(|l| l.get("profile_image_url_https"))
            .or_else(|| data.get("profile_image_url_https"))
            .or_else(|| data.get("profile_image_url"))
            .and_then(|u| u.as_str())
            .map(|s| s.to_string());

        // Get banner
        let banner_url = legacy.and_then(|l| l.get("profile_banner_url"))
            .or_else(|| data.get("profile_banner_url"))
            .and_then(|u| u.as_str())
            .map(|s| s.to_string());

        // Get location
        let location = legacy.and_then(|l| l.get("location"))
            .or_else(|| data.get("location"))
            .and_then(|l| l.as_str())
            .map(|s| s.to_string());

        // Get URL
        let website = legacy.and_then(|l| l.get("url"))
            .or_else(|| data.get("url"))
            .and_then(|u| u.as_str())
            .map(|s| s.to_string());

        // Get metrics
        let followers_count = legacy.and_then(|l| l.get("followers_count"))
            .or_else(|| data.get("followers_count"))
            .or_else(|| data.get("public_metrics").and_then(|m| m.get("followers_count")))
            .and_then(|v| v.as_u64());

        let following_count = legacy.and_then(|l| l.get("friends_count"))
            .or_else(|| data.get("following_count"))
            .or_else(|| data.get("public_metrics").and_then(|m| m.get("following_count")))
            .and_then(|v| v.as_u64());

        let posts_count = legacy.and_then(|l| l.get("statuses_count"))
            .or_else(|| data.get("tweet_count"))
            .or_else(|| data.get("public_metrics").and_then(|m| m.get("tweet_count")))
            .and_then(|v| v.as_u64());

        // Get verification status
        let is_verified = legacy.and_then(|l| l.get("verified"))
            .or_else(|| data.get("verified"))
            .and_then(|v| v.as_bool());

        // Get protected status
        let is_private = legacy.and_then(|l| l.get("protected"))
            .or_else(|| data.get("protected"))
            .and_then(|p| p.as_bool());

        // Get created_at
        let created_at = legacy.and_then(|l| l.get("created_at"))
            .or_else(|| data.get("created_at"))
            .and_then(|s| s.as_str())
            .and_then(|s| parse_twitter_timestamp(s));

        Ok(GhostUser {
            id,
            platform: Platform::X,
            username,
            display_name,
            bio,
            avatar_url,
            banner_url,
            profile_url: None,
            location,
            website,
            followers_count,
            following_count,
            posts_count,
            is_verified,
            is_private,
            is_bot: None,
            created_at,
            raw_metadata: Some(data.clone()),
        })
    }

    /// Extracts metadata
    pub fn extract_metadata(&self, data: &serde_json::Value) -> Result<XUserMetadata, GhostError> {
        let legacy = data.get("legacy");

        // Check verification types
        let is_blue_verified = data.get("is_blue_verified")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let is_business_verified = data.get("is_business_verified")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let is_gov_verified = legacy.and_then(|l| l.get("verified_type"))
            .and_then(|v| v.as_str())
            .map(|t| t == "Government")
            .unwrap_or(false);

        let is_legacy_verified = legacy.and_then(|l| l.get("verified"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        Ok(XUserMetadata {
            is_blue_verified,
            is_business_verified,
            is_gov_verified,
            is_legacy_verified,
            followers_you_follow: None,
            can_dm: legacy.and_then(|l| l.get("can_dm")).and_then(|c| c.as_bool()).unwrap_or(true),
            can_media_tag: legacy.and_then(|l| l.get("can_media_tag")).and_then(|c| c.as_bool()).unwrap_or(true),
            created_at: legacy.and_then(|l| l.get("created_at")).and_then(|s| s.as_str()).and_then(|s| parse_twitter_timestamp(s)),
            location: legacy.and_then(|l| l.get("location")).and_then(|l| l.as_str()).map(|s| s.to_string()),
            pinned_tweet_id: None,
            listed_count: legacy.and_then(|l| l.get("listed_count")).and_then(|l| l.as_u64()),
            url: None,
            has_banner: legacy.and_then(|l| l.get("profile_banner_url")).is_some(),
            account_type: None,
        })
    }
}

impl Default for UserParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse Twitter's timestamp format (e.g., "Wed Oct 10 20:19:24 +0000 2018")
fn parse_twitter_timestamp(s: &str) -> Option<i64> {
    chrono::DateTime::parse_from_str(s, "%a %b %d %H:%M:%S %z %Y")
        .map(|dt| dt.timestamp())
        .ok()
        .or_else(|| chrono::DateTime::parse_from_rfc3339(s).map(|dt| dt.timestamp()).ok())
}
