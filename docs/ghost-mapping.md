# Ghost Schema Mapping Guide

A comprehensive reference for mapping X (Twitter) and Threads API data models to the unified Ghost schema.

---

## Table of Contents

1. [Overview](#overview)
2. [GhostPost Mapping](#ghostpost-mapping)
3. [GhostUser Mapping](#ghostuser-mapping)
4. [GhostMedia Mapping](#ghostmedia-mapping)
5. [Platform-Specific Metadata](#platform-specific-metadata)
6. [Transformation Functions](#transformation-functions)
7. [Field Availability Matrix](#field-availability-matrix)
8. [Edge Cases & Special Handling](#edge-cases--special-handling)
9. [Implementation Examples](#implementation-examples)
10. [Validation Rules](#validation-rules)

---

## Overview

The Ghost API uses a unified schema to normalize data from different social media platforms. This document provides detailed mapping specifications for transforming platform-specific data into Ghost's unified types.

### Core Principles

1. **Platform Agnostic**: Ghost types abstract away platform differences
2. **Optional Fields**: Most fields are optional to handle platform limitations
3. **Raw Metadata Preservation**: Original platform data is preserved for debugging
4. **Type Safety**: All fields have explicit types for Rust safety guarantees

### Architecture

```
┌─────────────────┐     ┌─────────────────┐
│   X (Twitter)   │     │    Threads      │
│     API v2      │     │      API        │
└────────┬────────┘     └────────┬────────┘
         │                       │
         ▼                       ▼
┌─────────────────┐     ┌─────────────────┐
│   X Adapter     │     │ Threads Adapter │
│  (x-adapter)    │     │(threads-adapter)│
└────────┬────────┘     └────────┬────────┘
         │                       │
         └───────────┬───────────┘
                     │
                     ▼
         ┌─────────────────────┐
         │    Ghost Schema     │
         │   (ghost-schema)    │
         └─────────────────────┘
```

---

## GhostPost Mapping

The `GhostPost` struct represents a unified post/tweet/thread across all platforms.

### Rust Definition

```rust
pub struct GhostPost {
    pub id: String,                        // Required
    pub platform: Platform,                // Required
    pub text: String,                      // Required
    pub author: GhostUser,                 // Required
    pub media: Vec<GhostMedia>,            // Required (can be empty)
    pub created_at: i64,                   // Required (Unix timestamp)
    pub like_count: Option<u64>,           // Optional
    pub repost_count: Option<u64>,         // Optional
    pub reply_count: Option<u64>,          // Optional
    pub view_count: Option<u64>,           // Optional
    pub quote_count: Option<u64>,          // Optional
    pub in_reply_to: Option<String>,       // Optional
    pub quoted_post: Option<Box<GhostPost>>, // Optional
    pub raw_metadata: Option<serde_json::Value>, // Optional
}
```

### Field Mapping Table

| Ghost Field | X (Twitter) Source | Threads Source | Transformation | Notes |
|-------------|-------------------|----------------|----------------|-------|
| `id` | `data.id` | `id` | Direct | Both use string IDs |
| `platform` | `Platform::X` | `Platform::Threads` | Constant | Set by adapter |
| `text` | `data.text` | `text` | Direct | X: 280/4000 chars, Threads: 500 chars |
| `author` | `includes.users[0]` | `owner` | Map to GhostUser | Requires expansion for X |
| `media` | `includes.media[]` | `children` | Map to Vec<GhostMedia> | Extracted from attachments |
| `created_at` | `data.created_at` | `timestamp` | Parse ISO 8601 → Unix | Different formats |
| `like_count` | `data.public_metrics.like_count` | `likes_count` | Direct | Both integers |
| `repost_count` | `data.public_metrics.retweet_count` | `reposts_count` | Direct | Different naming |
| `reply_count` | `data.public_metrics.reply_count` | `replies_count` | Direct | Same concept |
| `view_count` | `data.public_metrics.impression_count` | insights.views | Direct/Async | Threads requires separate call |
| `quote_count` | `data.public_metrics.quote_count` | `quotes_count` | Direct | Both supported |
| `in_reply_to` | `data.referenced_tweets[?type=='replied_to'].id` | N/A | Array filter | X-only feature |
| `quoted_post` | `data.referenced_tweets[?type=='quoted']` | `quoted_post` | Recursive map | Different structure |
| `raw_metadata` | Full response | Full response | `serde_json::Value` | Preserved for debugging |

### X (Twitter) Mapping Detail

#### Basic Post Extraction

```rust
fn map_tweet_to_ghost(tweet: TweetResponse) -> Result<GhostPost, GhostError> {
    let data = tweet.data;
    
    // Extract author from includes
    let author = tweet.includes
        .users
        .into_iter()
        .next()
        .map(map_x_user_to_ghost)
        .unwrap_or_default();
    
    // Extract media from includes
    let media = tweet.includes
        .media
        .unwrap_or_default()
        .into_iter()
        .map(map_x_media_to_ghost)
        .collect();
    
    // Parse timestamp
    let created_at = parse_iso8601(&data.created_at)?;
    
    // Extract metrics
    let metrics = data.public_metrics.unwrap_or_default();
    
    // Find reply reference
    let in_reply_to = data.referenced_tweets
        .unwrap_or_default()
        .iter()
        .find(|r| r.reference_type == "replied_to")
        .map(|r| r.id.clone());
    
    Ok(GhostPost {
        id: data.id,
        platform: Platform::X,
        text: data.text,
        author,
        media,
        created_at,
        like_count: Some(metrics.like_count),
        repost_count: Some(metrics.retweet_count),
        reply_count: Some(metrics.reply_count),
        view_count: Some(metrics.impression_count),
        quote_count: Some(metrics.quote_count),
        in_reply_to,
        quoted_post: None, // Requires additional fetch
        raw_metadata: Some(serde_json::to_value(tweet)?),
    })
}
```

#### Referenced Tweets Handling

X uses the `referenced_tweets` array to indicate relationships:

| Reference Type | Ghost Field | Description |
|----------------|-------------|-------------|
| `replied_to` | `in_reply_to` | Direct reply to another tweet |
| `quoted` | `quoted_post` | Quote tweet (commentary on another tweet) |
| `retweeted` | N/A | Retweet (handled differently) |

```rust
#[derive(Debug, Deserialize)]
struct ReferencedTweet {
    #[serde(rename = "type")]
    reference_type: String,
    id: String,
}

fn extract_reference_type(referenced: &[ReferencedTweet], ref_type: &str) -> Option<String> {
    referenced
        .iter()
        .find(|r| r.reference_type == ref_type)
        .map(|r| r.id.clone())
}
```

### Threads Mapping Detail

#### Basic Post Extraction

```rust
fn map_threads_to_ghost(container: ThreadsMediaContainer) -> Result<GhostPost, GhostError> {
    // Extract author from owner object
    let author = map_threads_user_to_ghost(&container.owner);
    
    // Extract media based on type
    let media = match container.media_type.as_str() {
        "IMAGE" => vec![GhostMedia {
            id: container.id.clone(),
            media_type: MediaType::Image,
            url: container.media_url.clone().unwrap_or_default(),
            preview_url: container.thumbnail_url.clone(),
            ..Default::default()
        }],
        "VIDEO" => vec![GhostMedia {
            id: container.id.clone(),
            media_type: MediaType::Video,
            url: container.media_url.clone().unwrap_or_default(),
            preview_url: container.thumbnail_url.clone(),
            ..Default::default()
        }],
        "CAROUSEL" => container.children
            .iter()
            .map(map_threads_child_to_ghost)
            .collect(),
        _ => vec![],
    };
    
    // Parse timestamp (Threads uses +0000 format)
    let created_at = parse_threads_timestamp(&container.timestamp)?;
    
    Ok(GhostPost {
        id: container.id,
        platform: Platform::Threads,
        text: container.text.unwrap_or_default(),
        author,
        media,
        created_at,
        like_count: container.likes_count,
        repost_count: container.reposts_count,
        reply_count: container.replies_count,
        view_count: None, // Requires insights endpoint
        quote_count: container.quotes_count,
        in_reply_to: if container.is_reply { Some(String::new()) } else { None },
        quoted_post: container.quoted_post.map(|qp| Box::new(map_threads_to_ghost(qp)?)),
        raw_metadata: Some(serde_json::to_value(container)?),
    })
}
```

#### Carousel Handling

Threads carousels require special handling as they contain multiple child media:

```rust
#[derive(Debug, Deserialize)]
struct ThreadsMediaContainer {
    id: String,
    media_type: String,
    text: Option<String>,
    media_url: Option<String>,
    thumbnail_url: Option<String>,
    children: Vec<ThreadsMediaChild>,
    // ... other fields
}

#[derive(Debug, Deserialize)]
struct ThreadsMediaChild {
    id: String,
    media_type: String,
    media_url: String,
    thumbnail_url: Option<String>,
}

fn map_threads_child_to_ghost(child: &ThreadsMediaChild) -> GhostMedia {
    GhostMedia {
        id: child.id.clone(),
        media_type: match child.media_type.as_str() {
            "IMAGE" => MediaType::Image,
            "VIDEO" => MediaType::Video,
            _ => MediaType::Unknown,
        },
        url: child.media_url.clone(),
        preview_url: child.thumbnail_url.clone(),
        ..Default::default()
    }
}
```

---

## GhostUser Mapping

The `GhostUser` struct represents a unified user profile across all platforms.

### Rust Definition

```rust
pub struct GhostUser {
    pub id: String,                        // Required
    pub platform: Platform,                // Required
    pub username: String,                  // Required
    pub display_name: Option<String>,      // Optional
    pub bio: Option<String>,               // Optional
    pub avatar_url: Option<String>,        // Optional
    pub banner_url: Option<String>,        // Optional
    pub profile_url: Option<String>,       // Optional
    pub location: Option<String>,          // Optional
    pub website: Option<String>,           // Optional
    pub followers_count: Option<u64>,      // Optional
    pub following_count: Option<u64>,      // Optional
    pub posts_count: Option<u64>,          // Optional
    pub is_verified: Option<bool>,         // Optional
    pub is_private: Option<bool>,          // Optional
    pub is_bot: Option<bool>,              // Optional
    pub created_at: Option<i64>,           // Optional
    pub raw_metadata: Option<serde_json::Value>, // Optional
}
```

### Field Mapping Table

| Ghost Field | X (Twitter) Source | Threads Source | Transformation | Notes |
|-------------|-------------------|----------------|----------------|-------|
| `id` | `id` | `id` | Direct | Threads uses Instagram IDs |
| `platform` | `Platform::X` | `Platform::Threads` | Constant | Set by adapter |
| `username` | `username` | `username` | Direct | Strip @ prefix if present |
| `display_name` | `name` | `name` | Direct | May contain emojis |
| `bio` | `description` | `threads_biography` | Direct | Different field name |
| `avatar_url` | `profile_image_url` | `threads_profile_picture_url` | Direct | Full URL |
| `banner_url` | N/A (v1.1 only) | N/A | Not available | Deprecated in X v2 |
| `profile_url` | Constructed | `profile_url` | Construct/Map | `x.com/{username}` |
| `location` | `location` | N/A | Direct | X-only feature |
| `website` | `url` | N/A | Direct | X-only feature |
| `followers_count` | `public_metrics.followers_count` | N/A | Direct | Threads requires insights |
| `following_count` | `public_metrics.following_count` | N/A | Direct | Threads requires insights |
| `posts_count` | `public_metrics.tweet_count` | N/A | Direct | X-only feature |
| `is_verified` | `verified` OR `verified_type.is_some()` | `is_verified` | Complex | Different verification systems |
| `is_private` | `protected` | N/A | Direct | X-only feature |
| `is_bot` | N/A | N/A | Heuristic | Not provided by platforms |
| `created_at` | `created_at` | N/A | Parse ISO 8601 | X-only feature |
| `raw_metadata` | Full response | Full response | `serde_json::Value` | Preserved for debugging |

### X (Twitter) User Mapping

```rust
#[derive(Debug, Deserialize)]
struct XUser {
    id: String,
    username: String,
    name: String,
    description: Option<String>,
    location: Option<String>,
    url: Option<String>,
    profile_image_url: Option<String>,
    protected: Option<bool>,
    verified: Option<bool>,
    verified_type: Option<String>,
    public_metrics: Option<XUserMetrics>,
    created_at: Option<String>,
    entities: Option<XUserEntities>,
}

#[derive(Debug, Deserialize)]
struct XUserMetrics {
    followers_count: u64,
    following_count: u64,
    tweet_count: u64,
    listed_count: u64,
}

fn map_x_user_to_ghost(user: XUser) -> GhostUser {
    // Handle verification - multiple types in X
    let is_verified = user.verified.unwrap_or(false) 
        || user.verified_type.is_some();
    
    GhostUser {
        id: user.id,
        platform: Platform::X,
        username: user.username,
        display_name: Some(user.name),
        bio: user.description,
        avatar_url: user.profile_image_url,
        banner_url: None, // Not in v2 API
        profile_url: Some(format!("https://x.com/{}", user.username)),
        location: user.location,
        website: user.url,
        followers_count: user.public_metrics.as_ref().map(|m| m.followers_count),
        following_count: user.public_metrics.as_ref().map(|m| m.following_count),
        posts_count: user.public_metrics.as_ref().map(|m| m.tweet_count),
        is_verified: Some(is_verified),
        is_private: user.protected,
        is_bot: None,
        created_at: user.created_at.as_ref().and_then(|t| parse_iso8601(t).ok()),
        raw_metadata: Some(serde_json::to_value(user).unwrap_or(serde_json::Value::Null)),
    }
}
```

### Verification Type Handling (X Only)

X has multiple verification types that need special handling:

| `verified_type` | Description | Ghost Mapping |
|-----------------|-------------|---------------|
| `blue` | X Premium/Blue subscriber | `is_verified = true` |
| `business` | Verified business | `is_verified = true` |
| `government` | Government official | `is_verified = true` |
| `null` (but `verified=true`) | Legacy verified | `is_verified = true` |

For detailed verification tracking, use `XUserMetadata`:

```rust
pub struct XUserMetadata {
    pub is_blue_verified: bool,
    pub is_business_verified: bool,
    pub is_gov_verified: bool,
    pub is_legacy_verified: bool,
    // ... other fields
}

impl XUserMetadata {
    fn from_x_user(user: &XUser) -> Self {
        Self {
            is_blue_verified: user.verified_type.as_deref() == Some("blue"),
            is_business_verified: user.verified_type.as_deref() == Some("business"),
            is_gov_verified: user.verified_type.as_deref() == Some("government"),
            is_legacy_verified: user.verified.unwrap_or(false) && user.verified_type.is_none(),
        }
    }
}
```

### Threads User Mapping

```rust
#[derive(Debug, Deserialize)]
struct ThreadsUser {
    id: String,
    username: String,
    name: String,
    threads_profile_picture_url: Option<String>,
    threads_biography: Option<String>,
    profile_url: Option<String>,
    is_verified: Option<bool>,
}

fn map_threads_user_to_ghost(user: &ThreadsUser) -> GhostUser {
    GhostUser {
        id: user.id.clone(),
        platform: Platform::Threads,
        username: user.username.clone(),
        display_name: Some(user.name.clone()),
        bio: user.threads_biography.clone(),
        avatar_url: user.threads_profile_picture_url.clone(),
        banner_url: None,
        profile_url: user.profile_url.clone(),
        location: None,
        website: None,
        followers_count: None, // Requires insights endpoint
        following_count: None,
        posts_count: None,
        is_verified: user.is_verified,
        is_private: None,
        is_bot: None,
        created_at: None,
        raw_metadata: Some(serde_json::to_value(user).unwrap_or(serde_json::Value::Null)),
    }
}
```

---

## GhostMedia Mapping

The `GhostMedia` struct represents unified media attachments.

### Rust Definition

```rust
pub struct GhostMedia {
    pub id: String,                        // Required
    pub media_type: MediaType,             // Required
    pub url: String,                       // Required
    pub preview_url: Option<String>,       // Optional
    pub width: Option<u32>,                // Optional
    pub height: Option<u32>,               // Optional
    pub duration_secs: Option<f64>,        // Optional
    pub alt_text: Option<String>,          // Optional
    pub content_type: Option<String>,      // Optional
    pub size_bytes: Option<u64>,           // Optional
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaType {
    Image,
    Video,
    Gif,
    Audio,
    Unknown,
}
```

### Field Mapping Table

| Ghost Field | X (Twitter) Source | Threads Source | Transformation | Notes |
|-------------|-------------------|----------------|----------------|-------|
| `id` | `media_key` | `id` | Direct | Different ID formats |
| `media_type` | `type` | `media_type` | Enum map | See type mapping below |
| `url` | `url` (photo) or `preview_image_url` (video) | `media_url` | Conditional | Depends on type |
| `preview_url` | `preview_image_url` | `thumbnail_url` | Direct | For videos |
| `width` | `width` | N/A | Direct | X-only |
| `height` | `height` | N/A | Direct | X-only |
| `duration_secs` | `duration_ms / 1000` | N/A | Divide by 1000 | X-only |
| `alt_text` | `alt_text` | N/A | Direct | X-only (accessibility) |
| `content_type` | Inferred | Inferred | From URL/type | MIME type |
| `size_bytes` | N/A | N/A | Not provided | Neither platform provides |

### Media Type Mapping

| X (Twitter) `type` | Threads `media_type` | Ghost `MediaType` |
|-------------------|---------------------|-------------------|
| `photo` | `IMAGE` | `MediaType::Image` |
| `video` | `VIDEO` | `MediaType::Video` |
| `animated_gif` | N/A | `MediaType::Gif` |
| N/A | `TEXT` | N/A (not media) |
| N/A | `CAROUSEL` | Multiple items |
| N/A | N/A | `MediaType::Unknown` |

```rust
impl MediaType {
    fn from_x_type(x_type: &str) -> Self {
        match x_type {
            "photo" => MediaType::Image,
            "video" => MediaType::Video,
            "animated_gif" => MediaType::Gif,
            _ => MediaType::Unknown,
        }
    }
    
    fn from_threads_type(threads_type: &str) -> Self {
        match threads_type {
            "IMAGE" => MediaType::Image,
            "VIDEO" => MediaType::Video,
            _ => MediaType::Unknown,
        }
    }
}
```

### X (Twitter) Media Mapping

```rust
#[derive(Debug, Deserialize)]
struct XMedia {
    media_key: String,
    #[serde(rename = "type")]
    media_type: String,
    url: Option<String>,
    preview_image_url: Option<String>,
    alt_text: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    duration_ms: Option<u64>,
    public_metrics: Option<XMediaMetrics>,
    variants: Option<Vec<XVideoVariant>>,
}

#[derive(Debug, Deserialize)]
struct XMediaMetrics {
    view_count: u64,
}

#[derive(Debug, Deserialize)]
struct XVideoVariant {
    bit_rate: Option<u32>,
    content_type: String,
    url: String,
}

fn map_x_media_to_ghost(media: XMedia) -> GhostMedia {
    let media_type = MediaType::from_x_type(&media.media_type);
    
    // URL selection depends on type
    let url = match media_type {
        MediaType::Image => media.url.unwrap_or_default(),
        MediaType::Video | MediaType::Gif => media.preview_image_url.unwrap_or_default(),
        _ => String::new(),
    };
    
    GhostMedia {
        id: media.media_key,
        media_type,
        url,
        preview_url: media.preview_image_url,
        width: media.width,
        height: media.height,
        duration_secs: media.duration_ms.map(|ms| ms as f64 / 1000.0),
        alt_text: media.alt_text,
        content_type: infer_content_type(&url, &media.media_type),
        size_bytes: None,
    }
}

fn infer_content_type(url: &str, media_type: &str) -> Option<String> {
    if url.contains(".mp4") || media_type == "video" {
        Some("video/mp4".to_string())
    } else if url.contains(".png") {
        Some("image/png".to_string())
    } else if url.contains(".gif") || media_type == "animated_gif" {
        Some("image/gif".to_string())
    } else {
        Some("image/jpeg".to_string()) // Default
    }
}
```

### Threads Media Mapping

```rust
fn map_threads_media_to_ghost(container: &ThreadsMediaContainer) -> Vec<GhostMedia> {
    match container.media_type.as_str() {
        "TEXT" => vec![], // No media for text posts
        
        "IMAGE" => vec![GhostMedia {
            id: container.id.clone(),
            media_type: MediaType::Image,
            url: container.media_url.clone().unwrap_or_default(),
            preview_url: container.thumbnail_url.clone(),
            content_type: Some("image/jpeg".to_string()),
            ..Default::default()
        }],
        
        "VIDEO" => vec![GhostMedia {
            id: container.id.clone(),
            media_type: MediaType::Video,
            url: container.media_url.clone().unwrap_or_default(),
            preview_url: container.thumbnail_url.clone(),
            content_type: Some("video/mp4".to_string()),
            ..Default::default()
        }],
        
        "CAROUSEL" => container.children
            .iter()
            .map(|child| GhostMedia {
                id: child.id.clone(),
                media_type: MediaType::from_threads_type(&child.media_type),
                url: child.media_url.clone(),
                preview_url: child.thumbnail_url.clone(),
                ..Default::default()
            })
            .collect(),
        
        _ => vec![],
    }
}
```

---

## Platform-Specific Metadata

Beyond the unified Ghost types, each adapter stores platform-specific metadata for features not covered by the unified schema.

### X (Twitter) Metadata

```rust
/// X-specific post metadata stored in raw_metadata
pub struct XPostMetadata {
    /// Source client (e.g., "Twitter Web App")
    pub source: Option<String>,
    /// Language code (BCP 47)
    pub lang: Option<String>,
    /// Is quote status
    pub is_quote_status: bool,
    /// Sensitive content flag
    pub possibly_sensitive: Option<bool>,
    /// Geographic coordinates
    pub coordinates: Option<Coordinates>,
    /// Place information
    pub place: Option<Place>,
    /// Extracted hashtags
    pub hashtags: Vec<String>,
    /// User mentions with positions
    pub mentions: Vec<UserMention>,
    /// URL entities with expansions
    pub urls: Vec<UrlEntity>,
    /// Conversation thread ID
    pub conversation_id: Option<String>,
    /// Bookmark count
    pub bookmark_count: Option<u64>,
    /// Edit history tweet IDs
    pub edit_history: Vec<String>,
}

/// Geographic coordinates
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
    pub coord_type: String,
}

/// Place information
pub struct Place {
    pub id: String,
    pub name: String,
    pub full_name: String,
    pub country: String,
    pub country_code: String,
    pub place_type: String,
}

/// User mention entity
pub struct UserMention {
    pub id: String,
    pub username: String,
    pub name: Option<String>,
    pub indices: (usize, usize),
}

/// URL entity with expanded info
pub struct UrlEntity {
    pub url: String,           // t.co URL
    pub expanded_url: String,  // Full URL
    pub display_url: String,   // Display text
    pub indices: (usize, usize),
}

/// X-specific user metadata
pub struct XUserMetadata {
    /// Verification types
    pub is_blue_verified: bool,
    pub is_business_verified: bool,
    pub is_gov_verified: bool,
    pub is_legacy_verified: bool,
    /// Followers you also follow
    pub followers_you_follow: Option<u64>,
    /// DM capability
    pub can_dm: bool,
    /// Media tagging permission
    pub can_media_tag: bool,
    /// Pinned tweet ID
    pub pinned_tweet_id: Option<String>,
    /// Listed count
    pub listed_count: Option<u64>,
}
```

### Threads Metadata

```rust
/// Threads-specific post metadata
pub struct ThreadsPostMetadata {
    /// Post type classification
    pub post_type: PostType,
    /// Video has audio track
    pub has_audio: bool,
    /// Is a reel video
    pub is_reel: bool,
    /// Language detection
    pub lang: Option<String>,
    /// Reply audience restriction
    pub reply_audience: Option<ReplyAudience>,
    /// Hide status
    pub hide_status: Option<HideStatus>,
    /// Short URL identifier
    pub shortcode: Option<String>,
    /// Product type (THREADS, THREADS_REEL)
    pub media_product_type: Option<String>,
    /// Instagram crosspost flag
    pub is_shared_to_feed: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostType {
    Text,
    Image,
    Video,
    Carousel,
    Reel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplyAudience {
    Everyone,
    Mentions,
    Followers,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HideStatus {
    Hidden,
    Shown,
}

/// Threads-specific user metadata
pub struct ThreadsUserMetadata {
    /// Meta Verified subscription
    pub is_verified: bool,
    /// Business account type
    pub is_business_account: bool,
    /// Creator account type
    pub is_creator_account: bool,
    /// Linked Instagram account
    pub has_linked_instagram: bool,
    /// Deep link URL
    pub profile_deep_link: Option<String>,
    /// Bio links
    pub bio_links: Vec<BioLink>,
}

/// Link in user bio
pub struct BioLink {
    pub url: String,
    pub text: Option<String>,
}
```

---

## Transformation Functions

### Timestamp Parsing

Different timestamp formats require different parsing:

```rust
use chrono::{DateTime, Utc, TimeZone};

/// Parse X (Twitter) ISO 8601 timestamp
/// Format: "2024-01-15T10:30:00.000Z"
pub fn parse_x_timestamp(ts: &str) -> Result<i64, GhostError> {
    let dt = DateTime::parse_from_rfc3339(ts)
        .map_err(|e| GhostError::ParseError(format!("Invalid X timestamp: {}", e)))?;
    Ok(dt.timestamp())
}

/// Parse Threads timestamp
/// Format: "2024-01-15T10:30:00+0000"
pub fn parse_threads_timestamp(ts: &str) -> Result<i64, GhostError> {
    // Threads uses a non-standard format
    let dt = DateTime::parse_from_str(ts, "%Y-%m-%dT%H:%M:%S%z")
        .or_else(|_| DateTime::parse_from_rfc3339(ts))
        .map_err(|e| GhostError::ParseError(format!("Invalid Threads timestamp: {}", e)))?;
    Ok(dt.timestamp())
}

/// Generic ISO 8601 parser
pub fn parse_iso8601(ts: &str) -> Result<i64, GhostError> {
    // Try multiple formats
    let formats = [
        "%Y-%m-%dT%H:%M:%S%.3fZ",
        "%Y-%m-%dT%H:%M:%SZ",
        "%Y-%m-%dT%H:%M:%S%z",
        "%Y-%m-%dT%H:%M:%S%.3f%z",
    ];
    
    for fmt in &formats {
        if let Ok(dt) = DateTime::parse_from_str(ts, fmt) {
            return Ok(dt.timestamp());
        }
    }
    
    // Fallback to RFC 3339
    DateTime::parse_from_rfc3339(ts)
        .map(|dt| dt.timestamp())
        .map_err(|e| GhostError::ParseError(format!("Invalid timestamp: {}", e)))
}
```

### Username Normalization

```rust
/// Normalize username (remove @ prefix, lowercase)
pub fn normalize_username(username: &str) -> String {
    username
        .trim_start_matches('@')
        .to_lowercase()
}

/// Construct profile URL from username
pub fn build_profile_url(platform: Platform, username: &str) -> String {
    let normalized = normalize_username(username);
    match platform {
        Platform::X => format!("https://x.com/{}", normalized),
        Platform::Threads => format!("https://www.threads.net/@{}", normalized),
    }
}
```

### Metric Aggregation

```rust
/// Aggregated metrics across platforms
pub struct AggregatedMetrics {
    pub likes: u64,
    pub shares: u64,
    pub comments: u64,
    pub views: Option<u64>,
    pub engagement_rate: Option<f64>,
}

impl From<&GhostPost> for AggregatedMetrics {
    fn from(post: &GhostPost) -> Self {
        let likes = post.like_count.unwrap_or(0);
        let shares = post.repost_count.unwrap_or(0);
        let comments = post.reply_count.unwrap_or(0);
        let views = post.view_count;
        
        let engagement_rate = views.map(|v| {
            if v > 0 {
                (likes + shares + comments) as f64 / v as f64 * 100.0
            } else {
                0.0
            }
        });
        
        Self {
            likes,
            shares,
            comments,
            views,
            engagement_rate,
        }
    }
}
```

---

## Field Availability Matrix

### Post Fields by Platform

| Field | X (Twitter) | Threads | Source Type |
|-------|-------------|---------|-------------|
| `id` | ✅ | ✅ | Default |
| `text` | ✅ | ✅ | Default |
| `created_at` | ✅ | ✅ | Requested |
| `author` | ✅ | ✅ | Expanded |
| `media` | ✅ | ✅ | Expanded |
| `like_count` | ✅ | ✅ | Default |
| `repost_count` | ✅ | ✅ | Default |
| `reply_count` | ✅ | ✅ | Default |
| `view_count` | ✅ | ⚠️ | Default/Insights |
| `quote_count` | ✅ | ✅ | Default |
| `bookmark_count` | ✅ | ❌ | Default |
| `in_reply_to` | ✅ | ⚠️ | Default |
| `quoted_post` | ✅ | ✅ | Default |
| `lang` | ✅ | ❌ | Default |
| `source` | ✅ | ❌ | Default |
| `geo/place` | ✅ | ❌ | Default |
| `possibly_sensitive` | ✅ | ❌ | Default |
| `conversation_id` | ✅ | ❌ | Default |
| `edit_history` | ✅ | ❌ | Default |
| `reply_audience` | ❌ | ✅ | Default |
| `carousel` | ❌ | ✅ | Default |

Legend:
- ✅ Fully supported
- ⚠️ Partially supported or requires additional API call
- ❌ Not supported

### User Fields by Platform

| Field | X (Twitter) | Threads | Source Type |
|-------|-------------|---------|-------------|
| `id` | ✅ | ✅ | Default |
| `username` | ✅ | ✅ | Default |
| `display_name` | ✅ | ✅ | Default |
| `bio` | ✅ | ✅ | Requested |
| `avatar_url` | ✅ | ✅ | Requested |
| `banner_url` | ❌ | ❌ | N/A |
| `profile_url` | ✅ | ✅ | Constructed |
| `location` | ✅ | ❌ | Requested |
| `website` | ✅ | ❌ | Requested |
| `followers_count` | ✅ | ⚠️ | Requested/Insights |
| `following_count` | ✅ | ❌ | Requested |
| `posts_count` | ✅ | ❌ | Requested |
| `is_verified` | ✅ | ✅ | Requested |
| `is_private` | ✅ | ❌ | Requested |
| `created_at` | ✅ | ❌ | Requested |
| `pinned_post_id` | ✅ | ❌ | Requested |

---

## Edge Cases & Special Handling

### 1. Missing Author Data (X)

When using X API without proper expansions, author data may not be included:

```rust
fn extract_author_with_fallback(tweet: &TweetResponse) -> GhostUser {
    tweet.includes
        .users
        .as_ref()
        .and_then(|users| users.first())
        .map(|u| map_x_user_to_ghost(u.clone()))
        .unwrap_or_else(|| {
            // Fallback: create minimal user from author_id
            GhostUser {
                id: tweet.data.author_id.clone().unwrap_or_default(),
                platform: Platform::X,
                username: String::new(),
                display_name: None,
                ..Default::default()
            }
        })
}
```

### 2. Threads Views Not Available

Threads doesn't provide view counts on basic posts:

```rust
async fn get_threads_views(&self, post_id: &str) -> Result<Option<u64>, GhostError> {
    // Views require insights endpoint with special permissions
    match self.fetch_insights(post_id).await {
        Ok(insights) => {
            let views = insights.data
                .iter()
                .find(|m| m.name == "views")
                .and_then(|m| m.values.first())
                .map(|v| v.value);
            Ok(views)
        }
        Err(_) => Ok(None), // Gracefully handle missing permissions
    }
}
```

### 3. Carousel Posts (Threads)

Threads carousels can have mixed media types:

```rust
fn handle_carousel(container: &ThreadsMediaContainer) -> Vec<GhostMedia> {
    container.children
        .iter()
        .enumerate()
        .map(|(idx, child)| {
            let mut media = map_threads_child_to_ghost(child);
            // Add position for ordering
            media.id = format!("{}-{}", container.id, idx);
            media
        })
        .collect()
}
```

### 4. Deleted/Unavailable Posts

```rust
fn handle_deleted_post(result: Result<TweetResponse, XError>) -> Option<GhostPost> {
    match result {
        Ok(response) if response.errors.is_some() => {
            // Post exists but has errors (e.g., suspended user)
            response.data.map(|d| GhostPost {
                id: d.id,
                platform: Platform::X,
                text: "[Content unavailable]".to_string(),
                ..Default::default()
            })
        }
        Ok(response) => {
            Some(map_tweet_to_ghost(response))
        }
        Err(XError::NotFound { .. }) => None,
        Err(e) => {
            tracing::warn!("Error fetching post: {:?}", e);
            None
        }
    }
}
```

### 5. Rate Limit Handling

```rust
async fn fetch_with_retry<T, F, Fut>(
    fetch_fn: F,
    max_retries: u32,
) -> Result<T, GhostError>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, GhostError>>,
{
    let mut retries = 0;
    let mut delay = Duration::from_secs(1);
    
    loop {
        match fetch_fn().await {
            Ok(result) => return Ok(result),
            Err(GhostError::RateLimited { retry_after }) => {
                if retries >= max_retries {
                    return Err(GhostError::RateLimited { retry_after });
                }
                let wait = retry_after.map(Duration::from_secs).unwrap_or(delay);
                tokio::time::sleep(wait).await;
                delay = std::cmp::min(delay * 2, Duration::from_secs(60));
                retries += 1;
            }
            Err(e) => return Err(e),
        }
    }
}
```

---

## Implementation Examples

### Complete Post Mapping Pipeline

```rust
use ghost_schema::{GhostPost, GhostUser, GhostMedia, Platform, MediaType};
use serde::Deserialize;

// ============================================================================
// X (Twitter) Implementation
// ============================================================================

pub struct XAdapter {
    client: reqwest::Client,
    bearer_token: String,
}

impl XAdapter {
    /// Fetch and map a tweet to GhostPost
    pub async fn get_post(&self, id: &str) -> Result<GhostPost, GhostError> {
        let url = format!(
            "https://api.x.com/2/tweets/{}?tweet.fields=created_at,public_metrics,entities,referenced_tweets,attachments&expansions=author_id,attachments.media_keys&user.fields=name,username,profile_image_url,verified,public_metrics&media.fields=url,preview_image_url,type,duration_ms,alt_text",
            id
        );
        
        let response = self.client
            .get(&url)
            .bearer_auth(&self.bearer_token)
            .send()
            .await?
            .json::<XTweetResponse>()
            .await?;
        
        self.map_to_ghost(response)
    }
    
    fn map_to_ghost(&self, response: XTweetResponse) -> Result<GhostPost, GhostError> {
        let data = response.data.ok_or(GhostError::NotFound)?;
        
        // Extract author from includes
        let author = response.includes
            .users
            .into_iter()
            .find(|u| u.id == data.author_id)
            .map(|u| self.map_user_to_ghost(u))
            .unwrap_or_default();
        
        // Extract media
        let media = response.includes
            .media
            .unwrap_or_default()
            .into_iter()
            .map(|m| self.map_media_to_ghost(m))
            .collect();
        
        // Parse timestamp
        let created_at = data.created_at
            .as_ref()
            .map(|t| parse_iso8601(t))
            .transpose()?
            .unwrap_or(0);
        
        // Extract metrics
        let metrics = data.public_metrics.unwrap_or_default();
        
        // Find reply reference
        let in_reply_to = data.referenced_tweets
            .unwrap_or_default()
            .iter()
            .find(|r| r.reference_type == "replied_to")
            .map(|r| r.id.clone());
        
        Ok(GhostPost {
            id: data.id,
            platform: Platform::X,
            text: data.text,
            author,
            media,
            created_at,
            like_count: Some(metrics.like_count),
            repost_count: Some(metrics.retweet_count),
            reply_count: Some(metrics.reply_count),
            view_count: Some(metrics.impression_count),
            quote_count: Some(metrics.quote_count),
            in_reply_to,
            quoted_post: None,
            raw_metadata: Some(serde_json::to_value(response)?),
        })
    }
    
    fn map_user_to_ghost(&self, user: XUser) -> GhostUser {
        let is_verified = user.verified.unwrap_or(false) 
            || user.verified_type.is_some();
        
        GhostUser {
            id: user.id,
            platform: Platform::X,
            username: user.username,
            display_name: Some(user.name),
            bio: user.description,
            avatar_url: user.profile_image_url,
            followers_count: user.public_metrics.map(|m| m.followers_count),
            is_verified: Some(is_verified),
            is_private: user.protected,
            created_at: user.created_at.and_then(|t| parse_iso8601(&t).ok()),
            ..Default::default()
        }
    }
    
    fn map_media_to_ghost(&self, media: XMedia) -> GhostMedia {
        let media_type = match media.media_type.as_str() {
            "photo" => MediaType::Image,
            "video" => MediaType::Video,
            "animated_gif" => MediaType::Gif,
            _ => MediaType::Unknown,
        };
        
        GhostMedia {
            id: media.media_key,
            media_type,
            url: media.url.unwrap_or_default(),
            preview_url: media.preview_image_url,
            duration_secs: media.duration_ms.map(|ms| ms as f64 / 1000.0),
            alt_text: media.alt_text,
            ..Default::default()
        }
    }
}

// ============================================================================
// Threads Implementation
// ============================================================================

pub struct ThreadsAdapter {
    client: reqwest::Client,
    access_token: String,
}

impl ThreadsAdapter {
    /// Fetch and map a thread to GhostPost
    pub async fn get_post(&self, id: &str) -> Result<GhostPost, GhostError> {
        let url = format!(
            "https://graph.threads.net/v1.0/{}?fields=id,text,media_type,media_url,thumbnail_url,timestamp,owner,likes_count,reposts_count,replies_count,quotes_count,children{{id,media_type,media_url,thumbnail_url}}",
            id
        );
        
        let response = self.client
            .get(&url)
            .bearer_auth(&self.access_token)
            .send()
            .await?
            .json::<ThreadsMediaContainer>()
            .await?;
        
        self.map_to_ghost(response)
    }
    
    fn map_to_ghost(&self, container: ThreadsMediaContainer) -> Result<GhostPost, GhostError> {
        let author = self.map_user_to_ghost(&container.owner);
        
        let media = match container.media_type.as_str() {
            "IMAGE" => vec![GhostMedia {
                id: container.id.clone(),
                media_type: MediaType::Image,
                url: container.media_url.clone().unwrap_or_default(),
                preview_url: container.thumbnail_url.clone(),
                ..Default::default()
            }],
            "VIDEO" => vec![GhostMedia {
                id: container.id.clone(),
                media_type: MediaType::Video,
                url: container.media_url.clone().unwrap_or_default(),
                preview_url: container.thumbnail_url.clone(),
                ..Default::default()
            }],
            "CAROUSEL" => container.children
                .iter()
                .map(|c| GhostMedia {
                    id: c.id.clone(),
                    media_type: match c.media_type.as_str() {
                        "IMAGE" => MediaType::Image,
                        "VIDEO" => MediaType::Video,
                        _ => MediaType::Unknown,
                    },
                    url: c.media_url.clone(),
                    preview_url: c.thumbnail_url.clone(),
                    ..Default::default()
                })
                .collect(),
            _ => vec![],
        };
        
        let created_at = container.timestamp
            .as_ref()
            .map(|t| parse_threads_timestamp(t))
            .transpose()?
            .unwrap_or(0);
        
        Ok(GhostPost {
            id: container.id,
            platform: Platform::Threads,
            text: container.text.unwrap_or_default(),
            author,
            media,
            created_at,
            like_count: container.likes_count,
            repost_count: container.reposts_count,
            reply_count: container.replies_count,
            view_count: None, // Requires insights
            quote_count: container.quotes_count,
            in_reply_to: None,
            quoted_post: None,
            raw_metadata: Some(serde_json::to_value(container)?),
        })
    }
    
    fn map_user_to_ghost(&self, owner: &ThreadsOwner) -> GhostUser {
        GhostUser {
            id: owner.id.clone(),
            platform: Platform::Threads,
            username: owner.username.clone(),
            display_name: None,
            profile_url: Some(format!("https://www.threads.net/@{}", owner.username)),
            ..Default::default()
        }
    }
}
```

---

## Validation Rules

### Post Validation

```rust
impl GhostPost {
    pub fn validate(&self) -> Result<(), GhostError> {
        // Required fields
        if self.id.is_empty() {
            return Err(GhostError::ValidationError("Post ID is required".into()));
        }
        if self.text.is_empty() && self.media.is_empty() {
            return Err(GhostError::ValidationError(
                "Post must have text or media".into()
            ));
        }
        if self.created_at == 0 {
            return Err(GhostError::ValidationError("Created timestamp is required".into()));
        }
        
        // Reasonable bounds
        if self.created_at > chrono::Utc::now().timestamp() {
            return Err(GhostError::ValidationError(
                "Created timestamp cannot be in the future".into()
            ));
        }
        
        // Platform-specific validation
        match self.platform {
            Platform::X => {
                if self.text.len() > 4000 {
                    tracing::warn!("X post exceeds 4000 characters (Note Tweet)");
                }
            }
            Platform::Threads => {
                if self.text.len() > 500 {
                    tracing::warn!("Threads post exceeds 500 characters");
                }
            }
        }
        
        Ok(())
    }
}
```

### User Validation

```rust
impl GhostUser {
    pub fn validate(&self) -> Result<(), GhostError> {
        if self.id.is_empty() {
            return Err(GhostError::ValidationError("User ID is required".into()));
        }
        if self.username.is_empty() {
            return Err(GhostError::ValidationError("Username is required".into()));
        }
        
        // Username format validation
        let valid_username = self.username
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_');
        if !valid_username {
            return Err(GhostError::ValidationError(
                format!("Invalid username format: {}", self.username)
            ));
        }
        
        Ok(())
    }
}
```

### Media Validation

```rust
impl GhostMedia {
    pub fn validate(&self) -> Result<(), GhostError> {
        if self.url.is_empty() {
            return Err(GhostError::ValidationError("Media URL is required".into()));
        }
        
        // URL format validation
        if !self.url.starts_with("http") {
            return Err(GhostError::ValidationError(
                format!("Invalid media URL: {}", self.url)
            ));
        }
        
        // Duration validation for videos
        if matches!(self.media_type, MediaType::Video | MediaType::Gif) {
            if let Some(duration) = self.duration_secs {
                if duration <= 0.0 {
                    return Err(GhostError::ValidationError(
                        "Video duration must be positive".into()
                    ));
                }
            }
        }
        
        Ok(())
    }
}
```

---

## Appendix: Type Definitions

### X API Response Types

```rust
#[derive(Debug, Deserialize)]
struct XTweetResponse {
    data: Option<XTweetData>,
    includes: XIncludes,
    errors: Option<Vec<XError>>,
}

#[derive(Debug, Deserialize)]
struct XTweetData {
    id: String,
    text: String,
    author_id: String,
    created_at: Option<String>,
    public_metrics: Option<XTweetMetrics>,
    referenced_tweets: Option<Vec<XReferencedTweet>>,
    attachments: Option<XAttachments>,
    entities: Option<XEntities>,
    lang: Option<String>,
    source: Option<String>,
    conversation_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct XTweetMetrics {
    like_count: u64,
    retweet_count: u64,
    reply_count: u64,
    quote_count: u64,
    impression_count: u64,
    bookmark_count: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct XIncludes {
    users: Vec<XUser>,
    media: Option<Vec<XMedia>>,
    tweets: Option<Vec<XTweetData>>,
}

#[derive(Debug, Deserialize)]
struct XReferencedTweet {
    #[serde(rename = "type")]
    reference_type: String,
    id: String,
}

#[derive(Debug, Deserialize)]
struct XAttachments {
    media_keys: Option<Vec<String>>,
    poll_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct XEntities {
    hashtags: Option<Vec<XHashtag>>,
    mentions: Option<Vec<XMention>>,
    urls: Option<Vec<XUrl>>,
    cashtags: Option<Vec<XCashtag>>,
}
```

### Threads API Response Types

```rust
#[derive(Debug, Deserialize)]
struct ThreadsMediaContainer {
    id: String,
    media_type: String,
    media_product_type: Option<String>,
    text: Option<String>,
    media_url: Option<String>,
    thumbnail_url: Option<String>,
    timestamp: Option<String>,
    owner: ThreadsOwner,
    children: Vec<ThreadsMediaChild>,
    likes_count: Option<u64>,
    reposts_count: Option<u64>,
    replies_count: Option<u64>,
    quotes_count: Option<u64>,
    is_reply: Option<bool>,
    is_quote_post: Option<bool>,
    quoted_post: Option<Box<ThreadsMediaContainer>>,
    reply_audience: Option<String>,
    permalink: Option<String>,
    shortcode: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ThreadsOwner {
    id: String,
    username: String,
}

#[derive(Debug, Deserialize)]
struct ThreadsMediaChild {
    id: String,
    media_type: String,
    media_url: String,
    thumbnail_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ThreadsInsightsResponse {
    data: Vec<ThreadsInsight>,
}

#[derive(Debug, Deserialize)]
struct ThreadsInsight {
    name: String,
    period: String,
    values: Vec<ThreadsInsightValue>,
}

#[derive(Debug, Deserialize)]
struct ThreadsInsightValue {
    value: u64,
}
```

---

## Change Log

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2024-01-15 | Initial comprehensive mapping guide |
