# X (Twitter) vs Threads API Data Model Comparison

A comprehensive analysis of data models, schemas, and field mappings between X (formerly Twitter) API v2 and Threads API by Meta.

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [API Architecture Overview](#api-architecture-overview)
3. [Core Object Models](#core-object-models)
   - [Post/Tweet Object](#posttweet-object)
   - [User Object](#user-object)
   - [Media Object](#media-object)
   - [Metrics & Engagement](#metrics--engagement)
4. [Extended Object Models](#extended-object-models)
   - [Poll Object](#poll-object)
   - [Place/Location Object](#placelocation-object)
   - [Conversation/Thread Object](#conversationthread-object)
5. [Entity Objects](#entity-objects)
   - [Hashtags & Mentions](#hashtags--mentions)
   - [URLs & Links](#urls--links)
6. [Authentication & Session Models](#authentication--session-models)
7. [Error Handling Models](#error-handling-models)
8. [Field Mapping Reference](#field-mapping-reference)
9. [Unified Ghost Schema Mapping](#unified-ghost-schema-mapping)
10. [Best Practices & Recommendations](#best-practices--recommendations)

---

## Executive Summary

This document provides an in-depth comparison of the data models used by X (Twitter) API v2 and Threads API by Meta. Both platforms offer social media APIs with similar core concepts (posts, users, media, engagement metrics) but differ significantly in their implementation details, field naming conventions, and feature sets.

### Key Findings

| Aspect | X (Twitter) API v2 | Threads API |
|--------|-------------------|-------------|
| **API Style** | REST with JSON responses | GraphQL-based (Graph API) |
| **Post Character Limit** | 280 characters (basic), 4,000 (Blue) | 500 characters |
| **Media per Post** | Up to 4 images, 1 video | Up to 10 images/videos (carousel) |
| **Verification Types** | Blue, Business, Government, Legacy | Meta Verified |
| **Rate Limits** | Tweet-level (varies by endpoint) | App-level + User-level quotas |
| **Metrics Granularity** | Public + Private metrics | Public metrics only |

---

## API Architecture Overview

### X (Twitter) API v2 Architecture

X API v2 follows a RESTful architecture with field expansion capabilities. The API returns minimal data by default and requires explicit field requests via the `fields` parameter.

#### Base Structure
```
https://api.x.com/2/
├── tweets/
│   ├── {id}
│   ├── search/
│   └── recent
├── users/
│   ├── {id}
│   └── by/username/{username}
└── spaces/
```

#### Response Format
```json
{
  "data": { /* Primary object */ },
  "includes": { /* Expanded objects */ },
  "errors": [/* Error objects */],
  "meta": { /* Pagination/metadata */ }
}
```

#### Key Characteristics

1. **Field Expansion**: Use `tweet.fields`, `user.fields`, `media.fields` to request additional data
2. **Expansions**: Use `expansions` parameter to include related objects (e.g., `author_id` expansion)
3. **Pagination**: Uses `next_token` and `previous_token` for cursor-based pagination
4. **Rate Limiting**: Per-endpoint limits with `x-rate-limit-*` headers

### Threads API Architecture

Threads API is built on Meta's Graph API infrastructure, using a GraphQL-like query structure with nodes and edges.

#### Base Structure
```
https://graph.threads.net/v1.0/
├── {user-id}/
│   ├── threads
│   ├── replies
│   └── insights
├── {media-id}/
│   └── replies
└── me/
    ├── threads
    └── insights
```

#### Response Format
```json
{
  "data": [/* Array of objects */],
  "paging": {
    "cursors": { "before": "...", "after": "..." },
    "next": "https://..."
  }
}
```

#### Key Characteristics

1. **Media-Centric**: Posts are called "media containers" and can be text, image, video, or carousel
2. **Nested Fields**: Request specific fields using the `fields` parameter with nested field syntax
3. **Rate Limiting**: Application-level rate limits based on CPU time
4. **Insights Integration**: Built-in analytics for business/creator accounts

---

## Core Object Models

### Post/Tweet Object

The post object represents the primary content unit on both platforms. While both serve similar purposes, the structure and available fields differ significantly.

#### X (Twitter) Tweet Object

The Tweet object in X API v2 contains extensive fields for text content, engagement metrics, entities, and platform-specific features.

| Field | Type | Description | Request Parameter |
|-------|------|-------------|-------------------|
| `id` | string | Unique identifier for the Tweet | Default |
| `text` | string | The actual text content of the Tweet | Default |
| `created_at` | string (ISO 8601) | Creation timestamp | `tweet.fields=created_at` |
| `author_id` | string | ID of the authoring user | `tweet.fields=author_id` |
| `in_reply_to_user_id` | string | ID of user being replied to | `tweet.fields=in_reply_to_user_id` |
| `referenced_tweets` | array | Quoted/retweeted/replied-to tweets | `tweet.fields=referenced_tweets` |
| `attachments` | object | Media keys and poll IDs | `tweet.fields=attachments` |
| `withheld` | object | Content withheld info | `tweet.fields=withheld` |
| `geo` | object | Geographic data | `tweet.fields=geo` |
| `context_annotations` | array | Topic annotations | `tweet.fields=context_annotations` |
| `conversation_id` | string | Thread conversation ID | `tweet.fields=conversation_id` |
| `lang` | string | Language code (BCP 47) | `tweet.fields=lang` |
| `source` | string | Client app name | `tweet.fields=source` |
| `possibly_sensitive` | boolean | Sensitive content flag | `tweet.fields=possibly_sensitive` |
| `edit_history_tweet_ids` | array | Edit history IDs | Default |
| `edit_controls` | object | Edit window info | `tweet.fields=edit_controls` |
| `note_tweet` | object | Long-form content | `tweet.fields=note_tweet` |

#### Public Metrics (X)
| Field | Type | Description |
|-------|------|-------------|
| `public_metrics.like_count` | integer | Number of likes |
| `public_metrics.retweet_count` | integer | Number of retweets |
| `public_metrics.reply_count` | integer | Number of replies |
| `public_metrics.quote_count` | integer | Number of quotes |
| `public_metrics.impression_count` | integer | Number of views |
| `public_metrics.bookmark_count` | integer | Number of bookmarks |

#### Private Metrics (X) - Owner Only
| Field | Type | Description |
|-------|------|-------------|
| `non_public_metrics.impression_count` | integer | Total impressions |
| `non_public_metrics.url_link_clicks` | integer | Link clicks |
| `non_public_metrics.user_profile_clicks` | integer | Profile clicks |
| `organic_metrics.*` | object | Organic engagement |
| `promoted_metrics.*` | object | Paid promotion metrics |

#### Example Tweet Object (X API v2)
```json
{
  "data": {
    "id": "1234567890123456789",
    "text": "Hello world! This is my first post. #introduction @twitter",
    "created_at": "2024-01-15T10:30:00.000Z",
    "author_id": "987654321",
    "conversation_id": "1234567890123456789",
    "in_reply_to_user_id": null,
    "referenced_tweets": [],
    "attachments": {
      "media_keys": ["3_1234567890"]
    },
    "public_metrics": {
      "like_count": 42,
      "retweet_count": 12,
      "reply_count": 5,
      "quote_count": 3,
      "impression_count": 1500,
      "bookmark_count": 8
    },
    "entities": {
      "hashtags": [{"tag": "introduction", "start": 44, "end": 57}],
      "mentions": [{"username": "twitter", "id": "783214", "start": 58, "end": 66}]
    },
    "lang": "en",
    "source": "Twitter Web App",
    "possibly_sensitive": false,
    "edit_history_tweet_ids": ["1234567890123456789"]
  },
  "includes": {
    "users": [{
      "id": "987654321",
      "username": "example_user",
      "name": "Example User"
    }],
    "media": [{
      "media_key": "3_1234567890",
      "type": "photo",
      "url": "https://pbs.twimg.com/media/..."
    }]
  }
}
```

#### Threads Media Container Object

In Threads API, posts are referred to as "media containers" and support various content types including text, images, videos, and carousels.

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Unique identifier for the media container |
| `media_product_type` | string | Product type: `THREADS` or `THREADS_REEL` |
| `media_type` | string | Content type: `TEXT`, `IMAGE`, `VIDEO`, `CAROUSEL` |
| `text` | string | Text content (up to 500 characters) |
| `shortcode` | string | Short URL identifier (e.g., `CuVz9vqrKqN`) |
| `timestamp` | string (ISO 8601) | Creation timestamp |
| `permalink` | string | Permanent URL to the thread |
| `owner` | object | User object of the author |
| `children` | object | Carousel items (for CAROUSEL type) |
| `is_reply` | boolean | Whether this is a reply |
| `is_quote_post` | boolean | Whether this quotes another post |
| `quoted_post` | object | The quoted post details |
| `hide_status` | string | Visibility status: `HIDDEN`, `SHOWN` |
| `reply_audience` | string | Who can reply: `everyone`, `mentions`, `followers` |
| `has_audio` | boolean | Whether video has audio (for VIDEO) |

#### Threads Metrics
| Field | Type | Description |
|-------|------|-------------|
| `likes_count` | integer | Number of likes |
| `quotes_count` | integer | Number of quotes |
| `reposts_count` | integer | Number of reposts |
| `replies_count` | integer | Number of replies |

#### Threads Insights (Business/Creator Accounts)
| Field | Type | Description |
|-------|------|-------------|
| `views` | integer | Total views/impressions |
| `likes` | integer | Likes count |
| `replies` | integer | Replies count |
| `reposts` | integer | Reposts count |
| `quotes` | integer | Quotes count |
| `engagement` | integer | Total engagement count |
| `follower_count` | integer | Creator's follower count at post time |

#### Example Threads Media Container
```json
{
  "id": "17944322110154832",
  "media_product_type": "THREADS",
  "media_type": "IMAGE",
  "text": "Hello world! This is my first thread. #introduction",
  "shortcode": "CuVz9vqrKqN",
  "timestamp": "2024-01-15T10:30:00+0000",
  "permalink": "https://www.threads.net/@example_user/post/CuVz9vqrKqN",
  "owner": {
    "id": "17841400008460056",
    "username": "example_user"
  },
  "is_reply": false,
  "is_quote_post": false,
  "reply_audience": "everyone",
  "media_url": "https://scontent.cdninstagram.com/...",
  "thumbnail_url": "https://scontent.cdninstagram.com/...",
  "likes_count": 42,
  "quotes_count": 3,
  "reposts_count": 12,
  "replies_count": 5
}
```

### Post Object Comparison Matrix

| Feature | X (Twitter) | Threads | Notes |
|---------|-------------|---------|-------|
| **Unique ID** | `id` (string) | `id` (string) | Both use strings |
| **Text Content** | `text` | `text` | X: 280/4000 chars, Threads: 500 chars |
| **Timestamp** | `created_at` | `timestamp` | Both ISO 8601 |
| **Author** | `author_id` (expansion) | `owner` (nested object) | Different structure |
| **Likes** | `public_metrics.like_count` | `likes_count` | Direct field vs nested |
| **Reposts** | `public_metrics.retweet_count` | `reposts_count` | Different naming |
| **Replies** | `public_metrics.reply_count` | `replies_count` | Same naming |
| **Quotes** | `public_metrics.quote_count` | `quotes_count` | Both supported |
| **Views** | `public_metrics.impression_count` | `views` (insights) | Threads requires insights |
| **Bookmarks** | `public_metrics.bookmark_count` | Not available | X-only feature |
| **Language** | `lang` | Not available | X-only feature |
| **Source** | `source` | Not available | X-only feature |
| **Location** | `geo` | Not available | X-only feature |
| **Edit History** | `edit_history_tweet_ids` | Not available | X-only feature |
| **Reply Audience** | Not available | `reply_audience` | Threads-only feature |
| **Visibility** | `withheld` | `hide_status` | Different implementation |

---

### User Object

User objects represent account profiles on both platforms. They share common fields but differ in verification and profile features.

#### X (Twitter) User Object

| Field | Type | Description | Request Parameter |
|-------|------|-------------|-------------------|
| `id` | string | Unique user identifier | Default |
| `username` | string | Handle (e.g., @twitter) | Default |
| `name` | string | Display name | Default |
| `created_at` | string (ISO 8601) | Account creation date | `user.fields=created_at` |
| `description` | string | Bio/description | `user.fields=description` |
| `location` | string | User-provided location | `user.fields=location` |
| `url` | string | Profile URL | `user.fields=url` |
| `profile_image_url` | string | Avatar URL | `user.fields=profile_image_url` |
| `profile_banner_url` | string | Banner URL | Not in v2 API |
| `pinned_tweet_id` | string | ID of pinned tweet | `user.fields=pinned_tweet_id` |
| `protected` | boolean | Is account private | `user.fields=protected` |
| `verified` | boolean | Legacy verified | `user.fields=verified` |
| `verified_type` | string | Verification type | `user.fields=verified_type` |
| `public_metrics.followers_count` | integer | Followers count | `user.fields=public_metrics` |
| `public_metrics.following_count` | integer | Following count | `user.fields=public_metrics` |
| `public_metrics.tweet_count` | integer | Tweets count | `user.fields=public_metrics` |
| `public_metrics.listed_count` | integer | Listed count | `user.fields=public_metrics` |
| `entities` | object | Parsed bio entities | `user.fields=entities` |
| `withheld` | object | Withheld countries | `user.fields=withheld` |
| `most_recent_tweet_id` | string | Latest tweet ID | `user.fields=most_recent_tweet_id` |

#### Verification Types (X)
| Type | Description |
|------|-------------|
| `blue` | Twitter Blue/X Premium subscriber |
| `business` | Verified business account |
| `government` | Government official account |
| `null` | Legacy verified (via `verified` boolean) |

#### Threads User Object

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Threads user ID (same as Instagram) |
| `username` | string | Handle without @ |
| `name` | string | Display name |
| `threads_profile_picture_url` | string | Avatar URL |
| `threads_biography` | string | Bio text |
| `profile_url` | string | Profile link |
| `is_verified` | boolean | Meta Verified status |

#### User Object Comparison Matrix

| Feature | X (Twitter) | Threads | Notes |
|---------|-------------|---------|-------|
| **Unique ID** | `id` | `id` | Threads uses Instagram IDs |
| **Username** | `username` | `username` | Same naming |
| **Display Name** | `name` | `name` | Same naming |
| **Bio** | `description` | `threads_biography` | Different naming |
| **Avatar** | `profile_image_url` | `threads_profile_picture_url` | Different naming |
| **Banner** | Not in v2 | Not available | X v1.1 had this |
| **Location** | `location` | Not available | X-only feature |
| **Website** | `url` | Not available | X-only feature |
| **Followers** | `public_metrics.followers_count` | Not in basic profile | Requires insights |
| **Following** | `public_metrics.following_count` | Not in basic profile | Requires insights |
| **Post Count** | `public_metrics.tweet_count` | Not available | X-only feature |
| **Protected/Private** | `protected` | Not available | X-only feature |
| **Verified** | `verified`, `verified_type` | `is_verified` | Different systems |
| **Created At** | `created_at` | Not available | X-only feature |
| **Pinned Post** | `pinned_tweet_id` | Not available | X-only feature |

---

### Media Object

Media attachments are handled differently between platforms, with X using a separate `includes` section and Threads embedding media directly.

#### X (Twitter) Media Object

| Field | Type | Description | Request Parameter |
|-------|------|-------------|-------------------|
| `media_key` | string | Unique media identifier | Default |
| `type` | string | `photo`, `video`, `animated_gif` | Default |
| `url` | string | Media URL (photos only) | `media.fields=url` |
| `preview_image_url` | string | Thumbnail URL (videos) | `media.fields=preview_image_url` |
| `alt_text` | string | Accessibility description | `media.fields=alt_text` |
| `width` | integer | Width in pixels | `media.fields=width,height` |
| `height` | integer | Height in pixels | `media.fields=width,height` |
| `duration_ms` | integer | Video duration | `media.fields=duration_ms` |
| `public_metrics.view_count` | integer | Video views | `media.fields=public_metrics` |
| `variants` | array | Video format variants | `media.fields=variants` |

#### Media Constraints (X)
| Media Type | Max Count | Max Size | Formats |
|------------|-----------|----------|---------|
| Photo | 4 per tweet | 5MB (photo), 15MB (animated GIF) | PNG, JPG, GIF |
| Video | 1 per tweet | 512MB | MP4, MOV |
| Animated GIF | 1 per tweet | 15MB | GIF |

#### Threads Media Container Types

| Type | `media_type` | Description | Constraints |
|------|--------------|-------------|-------------|
| Text | `TEXT` | Text-only post | 500 characters max |
| Image | `IMAGE` | Single image | 8MB, JPG/PNG |
| Video | `VIDEO` | Single video | 60 seconds, MP4/MOV |
| Carousel | `CAROUSEL` | Multiple media | Up to 10 items, mixed images/videos |
| Reel | `THREADS_REEL` | Short video | Up to 90 seconds |

#### Threads Media Fields
| Field | Type | Description |
|-------|------|-------------|
| `media_url` | string | Direct media URL |
| `thumbnail_url` | string | Video thumbnail URL |
| `media_type` | string | Content type |
| `children` | object | Carousel items (nested) |
| `has_audio` | boolean | Video audio flag |
| `is_shared_to_feed` | boolean | Instagram crosspost flag |

#### Media Object Comparison Matrix

| Feature | X (Twitter) | Threads | Notes |
|---------|-------------|---------|-------|
| **Images per Post** | 4 max | 10 max (carousel) | Threads allows more |
| **Video per Post** | 1 | 1 (or in carousel) | Similar |
| **Max Video Duration** | 140s (2m20s) | 90s (reel: 90s) | X allows longer |
| **Alt Text** | `alt_text` | Limited support | X more accessible |
| **GIF Support** | `animated_gif` | Not native | X has GIF type |
| **Video Views** | `public_metrics.view_count` | In insights | Different access |
| **Dimensions** | `width`, `height` | Not provided | X more detailed |
| **Video Variants** | `variants` | Not provided | X offers multiple formats |

---

### Metrics & Engagement

Engagement metrics are tracked differently between platforms, with X offering both public and private metrics while Threads focuses on public metrics.

#### X (Twitter) Metrics Structure

```json
{
  "public_metrics": {
    "like_count": 100,
    "retweet_count": 50,
    "reply_count": 25,
    "quote_count": 10,
    "impression_count": 5000,
    "bookmark_count": 15
  },
  "non_public_metrics": {
    "impression_count": 5000,
    "url_link_clicks": 100,
    "user_profile_clicks": 50
  },
  "organic_metrics": {
    "impression_count": 4500,
    "like_count": 90,
    "reply_count": 20,
    "retweet_count": 45
  },
  "promoted_metrics": {
    "impression_count": 500,
    "like_count": 10,
    "reply_count": 5,
    "retweet_count": 5
  }
}
```

#### Threads Metrics Structure

```json
{
  "likes_count": 100,
  "reposts_count": 50,
  "quotes_count": 10,
  "replies_count": 25
}
```

#### Threads Insights (via `/insights` endpoint)

```json
{
  "data": [
    {
      "name": "views",
      "period": "lifetime",
      "values": [{"value": 5000}]
    },
    {
      "name": "likes",
      "period": "lifetime",
      "values": [{"value": 100}]
    },
    {
      "name": "replies",
      "period": "lifetime",
      "values": [{"value": 25}]
    },
    {
      "name": "reposts",
      "period": "lifetime",
      "values": [{"value": 50}]
    },
    {
      "name": "quotes",
      "period": "lifetime",
      "values": [{"value": 10}]
    },
    {
      "name": "engagement",
      "period": "lifetime",
      "values": [{"value": 185}]
    }
  ]
}
```

#### Metrics Comparison Matrix

| Metric | X (Twitter) | Threads | Access Level |
|--------|-------------|---------|--------------|
| **Likes** | `like_count` | `likes_count` | Public |
| **Reposts** | `retweet_count` | `reposts_count` | Public |
| **Replies** | `reply_count` | `replies_count` | Public |
| **Quotes** | `quote_count` | `quotes_count` | Public |
| **Views** | `impression_count` | `views` (insights) | X: Public / Threads: Insights |
| **Bookmarks** | `bookmark_count` | Not available | X-only |
| **Link Clicks** | `url_link_clicks` | Not available | X Private |
| **Profile Clicks** | `user_profile_clicks` | Not available | X Private |
| **Engagement** | Not calculated | `engagement` | Threads insights |
| **Organic vs Paid** | Split metrics | Not available | X-only |

---

## Extended Object Models

### Poll Object

X (Twitter) supports native polls, while Threads does not currently have a polling feature.

#### X (Twitter) Poll Object

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Poll ID |
| `options` | array | Poll options |
| `options[].position` | integer | Option position |
| `options[].label` | string | Option text |
| `options[].votes` | integer | Vote count |
| `voting_status` | string | `open` or `closed` |
| `end_datetime` | string (ISO 8601) | Poll end time |
| `duration_minutes` | integer | Poll duration |

#### Example Poll Object (X)
```json
{
  "poll": {
    "id": "1234567890",
    "options": [
      {"position": 1, "label": "Option A", "votes": 100},
      {"position": 2, "label": "Option B", "votes": 50}
    ],
    "voting_status": "closed",
    "end_datetime": "2024-01-16T10:30:00.000Z",
    "duration_minutes": 1440
  }
}
```

### Place/Location Object

X (Twitter) supports location tagging with detailed place information. Threads does not have native location support.

#### X (Twitter) Place Object

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Place ID |
| `name` | string | Place name |
| `full_name` | string | Full place name |
| `country` | string | Country name |
| `country_code` | string | ISO country code |
| `place_type` | string | `poi`, `neighborhood`, `city`, `admin` |
| `geo` | object | Bounding box coordinates |
| `contained_within` | array | Parent places |

#### Example Place Object (X)
```json
{
  "geo": {
    "place_id": "01a9a39529b27f36",
    "name": "Manhattan",
    "full_name": "Manhattan, NY",
    "country": "United States",
    "country_code": "US",
    "place_type": "city",
    "coordinates": {
      "type": "Polygon",
      "coordinates": [[[-74.026, 40.684], [-73.947, 40.684], [-73.947, 40.877], [-74.026, 40.877]]]
    }
  }
}
```

### Conversation/Thread Object

Both platforms support conversation threads but implement them differently.

#### X (Twitter) Conversation Model

- Uses `conversation_id` to group tweets
- `referenced_tweets` array for reply chain
- `in_reply_to_user_id` for direct replies

```json
{
  "conversation_id": "1234567890123456789",
  "in_reply_to_user_id": "987654321",
  "referenced_tweets": [
    {"type": "replied_to", "id": "1234567890123456780"}
  ]
}
```

#### Threads Conversation Model

- Replies have `is_reply` flag
- `quoted_post` for quote posts
- Direct nesting of replies

```json
{
  "is_reply": true,
  "is_quote_post": false,
  "quoted_post": {
    "id": "17944322110154830",
    "text": "Original post text"
  }
}
```

---

## Entity Objects

### Hashtags & Mentions

Both platforms parse and return hashtag and mention entities from post text.

#### X (Twitter) Entity Objects

```json
{
  "entities": {
    "hashtags": [
      {
        "tag": "introduction",
        "start": 44,
        "end": 57
      }
    ],
    "mentions": [
      {
        "username": "twitter",
        "id": "783214",
        "start": 58,
        "end": 66
      }
    ],
    "cashtags": [
      {
        "tag": "TWTR",
        "start": 67,
        "end": 72
      }
    ],
    "urls": [
      {
        "url": "https://t.co/abc123",
        "expanded_url": "https://example.com",
        "display_url": "example.com",
        "start": 73,
        "end": 96
      }
    ]
  }
}
```

#### Threads Entity Objects

Threads parses entities but doesn't provide detailed entity objects in the API response. Hashtags and mentions are extracted from text.

```json
{
  "text": "Hello world! #introduction @example_user",
  // No separate entity object provided
}
```

### URLs & Links

#### X (Twitter) URL Entities

| Field | Type | Description |
|-------|------|-------------|
| `url` | string | t.co shortened URL |
| `expanded_url` | string | Full original URL |
| `display_url` | string | Display-friendly URL |
| `unwound_url` | string | Final resolved URL |
| `images` | array | Open Graph images |
| `title` | string | Link title |
| `description` | string | Link description |
| `start`, `end` | integer | Character positions |

#### Threads URL Handling

Threads provides:
- Automatic link detection in text
- No separate URL entity objects
- No link previews in API responses

---

## Authentication & Session Models

### X (Twitter) Authentication

#### OAuth 2.0 Flow
| Grant Type | Use Case |
|------------|----------|
| Authorization Code | User context access |
| Client Credentials | App-only access |
| PKCE | Mobile/SPA apps |

#### Token Types
| Token Type | Scope | Expiration |
|------------|-------|------------|
| Access Token | User context | Varies |
| Bearer Token | App-only | No expiration |
| Refresh Token | Token refresh | 6 months |

### Threads Authentication

#### OAuth 2.0 Flow
| Grant Type | Use Case |
|------------|----------|
| Authorization Code | User context access |

#### Required Permissions
| Permission | Description |
|------------|-------------|
| `threads_basic` | Read profile and posts |
| `threads_content_publish` | Create posts |
| `threads_manage_replies` | Manage replies |
| `threads_manage_insights` | Access insights |
| `threads_read_insights` | Read insights |

---

## Error Handling Models

### X (Twitter) Error Model

```json
{
  "errors": [
    {
      "value": "1234567890",
      "detail": "Could not find tweet with ids: [1234567890].",
      "title": "Not Found Error",
      "resource_type": "tweet",
      "parameter": "id",
      "resource_id": "1234567890",
      "type": "https://api.twitter.com/2/problems/resource-not-found"
    }
  ],
  "title": "Invalid Request",
  "detail": "One or more errors occurred.",
  "type": "https://api.twitter.com/2/problems/invalid-request"
}
```

#### X Error Types
| Type | HTTP Status | Description |
|------|-------------|-------------|
| `resource-not-found` | 404 | Resource doesn't exist |
| `usage-capped` | 429 | Rate limit exceeded |
| `forbidden` | 403 | Access denied |
| `invalid-request` | 400 | Bad request |
| `client-forbidden` | 403 | Client not authorized |

### Threads Error Model

```json
{
  "error": {
    "message": "Invalid OAuth access token",
    "type": "OAuthException",
    "code": 190,
    "error_user_msg": "The access token is invalid or has expired."
  }
}
```

#### Threads Error Codes
| Code | Type | Description |
|------|------|-------------|
| 190 | OAuthException | Invalid token |
| 4 | OAuthException | Invalid request |
| 17 | ApiException | Rate limit |
| 32 | ApiException | Page permission |
| 100 | ApiException | Invalid parameter |
| 200 | ApiException | Permission denied |

---

## Field Mapping Reference

### Post/Tweet Field Mapping

| Unified Field | X (Twitter) Field | Threads Field | Transformation |
|---------------|-------------------|---------------|----------------|
| `id` | `id` | `id` | Direct |
| `text` | `text` | `text` | Direct |
| `created_at` | `created_at` | `timestamp` | ISO 8601 |
| `author_id` | `author_id` | `owner.id` | Nested extraction |
| `author_username` | includes.user.username | `owner.username` | Nested extraction |
| `like_count` | `public_metrics.like_count` | `likes_count` | Direct |
| `repost_count` | `public_metrics.retweet_count` | `reposts_count` | Direct |
| `reply_count` | `public_metrics.reply_count` | `replies_count` | Direct |
| `quote_count` | `public_metrics.quote_count` | `quotes_count` | Direct |
| `view_count` | `public_metrics.impression_count` | insights.views | Requires insights |
| `in_reply_to_id` | `referenced_tweets[?type=='replied_to'].id` | N/A | Array filter |
| `quoted_post_id` | `referenced_tweets[?type=='quoted'].id` | `quoted_post.id` | Array filter |
| `language` | `lang` | N/A | X-only |
| `source` | `source` | N/A | X-only |
| `possibly_sensitive` | `possibly_sensitive` | N/A | X-only |
| `conversation_id` | `conversation_id` | N/A | X-only |

### User Field Mapping

| Unified Field | X (Twitter) Field | Threads Field | Transformation |
|---------------|-------------------|---------------|----------------|
| `id` | `id` | `id` | Direct |
| `username` | `username` | `username` | Direct (strip @) |
| `display_name` | `name` | `name` | Direct |
| `bio` | `description` | `threads_biography` | Direct |
| `avatar_url` | `profile_image_url` | `threads_profile_picture_url` | Direct |
| `banner_url` | N/A (v1.1 only) | N/A | Not available |
| `followers_count` | `public_metrics.followers_count` | N/A | X-only basic |
| `following_count` | `public_metrics.following_count` | N/A | X-only basic |
| `posts_count` | `public_metrics.tweet_count` | N/A | X-only |
| `is_verified` | `verified` or `verified_type` | `is_verified` | Boolean check |
| `is_private` | `protected` | N/A | X-only |
| `created_at` | `created_at` | N/A | X-only |
| `location` | `location` | N/A | X-only |
| `website` | `url` | N/A | X-only |

### Media Field Mapping

| Unified Field | X (Twitter) Field | Threads Field | Transformation |
|---------------|-------------------|---------------|----------------|
| `id` | `media_key` | `id` | Direct |
| `type` | `type` | `media_type` | Map values |
| `url` | `url` (photo) or `preview_image_url` (video) | `media_url` | Conditional |
| `thumbnail_url` | `preview_image_url` | `thumbnail_url` | Direct |
| `width` | `width` | N/A | X-only |
| `height` | `height` | N/A | X-only |
| `duration_secs` | `duration_ms / 1000` | N/A | Convert ms to seconds |
| `alt_text` | `alt_text` | N/A | X-only |
| `view_count` | `public_metrics.view_count` | insights.views | Different access |

---

## Unified Ghost Schema Mapping

The Ghost API uses a unified schema to normalize data from different platforms. Below is the mapping between platform-specific fields and the unified Ghost schema.

### GhostPost Mapping

```rust
pub struct GhostPost {
    pub id: String,              // X: id, Threads: id
    pub platform: Platform,      // X | Threads
    pub text: String,            // X: text, Threads: text
    pub author: GhostUser,       // Mapped from includes or owner
    pub media: Vec<GhostMedia>,  // X: includes.media, Threads: children
    pub created_at: i64,         // X: created_at, Threads: timestamp (Unix)
    pub like_count: Option<u64>, // X: public_metrics.like_count, Threads: likes_count
    pub repost_count: Option<u64>, // X: public_metrics.retweet_count, Threads: reposts_count
    pub reply_count: Option<u64>,  // X: public_metrics.reply_count, Threads: replies_count
    pub view_count: Option<u64>,   // X: public_metrics.impression_count, Threads: insights.views
    pub quote_count: Option<u64>,  // X: public_metrics.quote_count, Threads: quotes_count
    pub in_reply_to: Option<String>, // X: referenced_tweets, Threads: is_reply
    pub quoted_post: Option<Box<GhostPost>>, // X: referenced_tweets, Threads: quoted_post
    pub raw_metadata: Option<serde_json::Value>, // Original platform data
}
```

### GhostUser Mapping

```rust
pub struct GhostUser {
    pub id: String,              // X: id, Threads: id
    pub platform: Platform,      // X | Threads
    pub username: String,        // X: username, Threads: username
    pub display_name: Option<String>, // X: name, Threads: name
    pub bio: Option<String>,     // X: description, Threads: threads_biography
    pub avatar_url: Option<String>, // X: profile_image_url, Threads: threads_profile_picture_url
    pub banner_url: Option<String>, // X: Not in v2, Threads: N/A
    pub profile_url: Option<String>, // Constructed from username
    pub location: Option<String>,   // X: location, Threads: N/A
    pub website: Option<String>,    // X: url, Threads: N/A
    pub followers_count: Option<u64>, // X: public_metrics.followers_count, Threads: insights
    pub following_count: Option<u64>, // X: public_metrics.following_count, Threads: N/A
    pub posts_count: Option<u64>,    // X: public_metrics.tweet_count, Threads: N/A
    pub is_verified: Option<bool>,   // X: verified || verified_type, Threads: is_verified
    pub is_private: Option<bool>,    // X: protected, Threads: N/A
    pub is_bot: Option<bool>,        // X: N/A, Threads: N/A
    pub created_at: Option<i64>,     // X: created_at, Threads: N/A
    pub raw_metadata: Option<serde_json::Value>, // Original platform data
}
```

### GhostMedia Mapping

```rust
pub struct GhostMedia {
    pub id: String,              // X: media_key, Threads: id
    pub media_type: MediaType,   // X: type -> enum, Threads: media_type -> enum
    pub url: String,             // X: url/preview_image_url, Threads: media_url
    pub preview_url: Option<String>, // X: preview_image_url, Threads: thumbnail_url
    pub width: Option<u32>,      // X: width, Threads: N/A
    pub height: Option<u32>,     // X: height, Threads: N/A
    pub duration_secs: Option<f64>, // X: duration_ms/1000, Threads: N/A
    pub alt_text: Option<String>,   // X: alt_text, Threads: N/A
    pub content_type: Option<String>, // Inferred from URL/type
    pub size_bytes: Option<u64>,    // N/A for both
}

pub enum MediaType {
    Image,  // X: photo, Threads: IMAGE
    Video,  // X: video, Threads: VIDEO
    Gif,    // X: animated_gif, Threads: N/A
    Audio,  // N/A for both
    Unknown,
}
```

---

## Best Practices & Recommendations

### API Selection Strategy

1. **Use X API v2 when you need:**
   - Advanced search capabilities
   - Location-based queries
   - Poll creation/analysis
   - Detailed engagement metrics (public + private)
   - Long-form content (Notes)
   - Edit history tracking

2. **Use Threads API when you need:**
   - Carousel posts with multiple media
   - Simpler integration with Meta ecosystem
   - Instagram cross-posting
   - Creator/business insights integration
   - More media per post (up to 10)

### Rate Limiting Strategy

| Platform | Rate Limit Type | Strategy |
|----------|----------------|----------|
| X | Per-endpoint | Implement exponential backoff with `retry-after` |
| Threads | CPU time quota | Batch requests, cache responses |

### Data Normalization Recommendations

1. **Always store raw metadata**: Keep original platform data for debugging and future field extraction
2. **Handle missing fields gracefully**: Both platforms have optional fields that may not be present
3. **Normalize timestamps**: Convert all timestamps to Unix milliseconds for consistency
4. **Implement field expansion**: Only request fields you need to reduce payload size
5. **Cache user data**: User profiles change less frequently than posts

### Error Handling Best Practices

1. **X API v2:**
   - Check `errors` array in response
   - Handle `429 Too Many Requests` with proper backoff
   - Respect `x-rate-limit-reset` header

2. **Threads API:**
   - Check `error.code` for specific error types
   - Handle OAuth token expiration (code 190)
   - Implement proper permission request flows

### Migration Considerations

When building cross-platform applications:

1. **Abstract platform differences**: Use adapter pattern with unified schema
2. **Handle platform-specific features**: Some features won't have equivalents
3. **Document limitations**: Be clear about what's supported per platform
4. **Version your API clients**: Both platforms evolve their APIs over time

---

## Appendix: Platform-Specific Features

### X-Only Features
- Polls
- Location/Geo tagging
- Bookmarks
- Lists
- Spaces (audio rooms)
- Community notes
- Long-form content (Notes)
- Edit history
- Private metrics
- Promoted/tweet metrics split
- Cashtags ($TICKER)
- Thread/conversation ID grouping

### Threads-Only Features
- Carousel posts (up to 10 items)
- Reply audience restrictions
- Reel integration
- Instagram cross-posting
- Meta Verified integration
- Simplified reply/quote structure

---

## Pagination Models

Pagination is a critical aspect of both APIs when dealing with large result sets. Each platform implements different pagination strategies with varying capabilities and constraints.

### X (Twitter) Pagination

X API v2 uses cursor-based pagination with tokens for navigating through result sets. Different endpoints use slightly different pagination mechanisms.

#### Pagination Types by Endpoint

| Endpoint Type | Token Name | Direction | Max Results |
|---------------|------------|-----------|-------------|
| Search Tweets | `next_token` | Forward only | 100 per page |
| User Timeline | `pagination_token` | Bi-directional | 100 per page |
| Followers/Following | `pagination_token` | Bi-directional | 1000 per page |
| Likes | `pagination_token` | Bi-directional | 100 per page |
| Bookmarks | `pagination_token` | Bi-directional | 100 per page |

#### Pagination Response Structure

```json
{
  "data": [/* Array of tweets */],
  "meta": {
    "result_count": 100,
    "next_token": "7140dibdnowq9du7gx4480r5k5blf8x4pqr069h9t",
    "previous_token": "46527ibdnowq9du7gx4480r5k5blf8x4pqr069h9t",
    "newest_id": "1234567890123456789",
    "oldest_id": "9876543210987654321"
  }
}
```

#### Pagination Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `max_results` | integer | Number of results per page (10-100 for tweets) |
| `next_token` | string | Token for next page (forward pagination) |
| `pagination_token` | string | Token for bidirectional pagination |
| `previous_token` | string | Token for previous page (when available) |

#### Pagination Best Practices (X)

1. **Always check for `next_token`**: Its presence indicates more results available
2. **Store pagination tokens**: For resuming interrupted pagination sessions
3. **Use `max_results` wisely**: Larger pages reduce API calls but increase latency
4. **Handle rate limits**: Pagination resets on rate limit, track progress

### Threads Pagination

Threads API uses cursor-based pagination following Meta's Graph API conventions with `before` and `after` cursors.

#### Pagination Response Structure

```json
{
  "data": [/* Array of media containers */],
  "paging": {
    "cursors": {
      "before": "QVFIUlA3ODN...",
      "after": "QVFIUlA3ODN..."
    },
    "previous": "https://graph.threads.net/v1.0/me/threads?access_token=...&before=QVFIUlA3ODN...",
    "next": "https://graph.threads.net/v1.0/me/threads?access_token=...&after=QVFIUlA3ODN..."
  }
}
```

#### Pagination Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `after` | string | Cursor for results after this point |
| `before` | string | Cursor for results before this point |
| `limit` | integer | Number of results per page |

#### Pagination Comparison Matrix

| Feature | X (Twitter) | Threads |
|---------|-------------|---------|
| **Pagination Style** | Token-based | Cursor-based |
| **Bidirectional** | Yes (some endpoints) | Yes |
| **Default Page Size** | 100 | 25 |
| **Max Page Size** | 100-1000 | 100 |
| **Token Persistence** | Stateless | Stateless |
| **Time-based** | No | No (cursor-based) |

---

## Streaming APIs

Real-time data streaming is available on X API v2 but not currently on Threads API.

### X (Twitter) Streaming Endpoints

X API v2 offers two types of streaming endpoints for real-time tweet delivery:

#### 1. Filtered Stream

Real-time streaming of tweets matching specified rules.

| Feature | Description |
|---------|-------------|
| **Endpoint** | `GET /2/tweets/search/stream` |
| **Access Level** | Basic, Pro, Enterprise |
| **Rules Limit** | 5 (Basic), 1000 (Pro), unlimited (Enterprise) |
| **Reconnect** | Auto-reconnect with backfill available |

##### Stream Rule Object

```json
{
  "id": "1234567890",
  "value": "python OR #programming -is:retweet",
  "tag": "Programming tweets"
}
```

##### Rule Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `keyword` | Match keyword | `python` |
| `#hashtag` | Match hashtag | `#programming` |
| `@username` | Mention user | `@twitter` |
| `from:user` | Tweets from user | `from:twitter` |
| `to:user` | Replies to user | `to:twitter` |
| `-keyword` | Exclude keyword | `-spam` |
| `is:retweet` | Retweets only | `is:retweet` |
| `is:reply` | Replies only | `is:reply` |
| `is:quote` | Quote tweets only | `is:quote` |
| `has:media` | Has media | `has:media` |
| `has:images` | Has images | `has:images` |
| `has:videos` | Has videos | `has:videos` |
| `has:links` | Has links | `has:links` |
| `lang:en` | Language filter | `lang:en` |

##### Stream Connection Flow

```
1. Connect to stream endpoint (long-lived HTTP connection)
2. Receive tweets in real-time as JSON
3. Handle reconnects with exponential backoff
4. Use backfill minutes on reconnect (Pro/Enterprise)
```

#### 2. Sample Stream

Random sample of approximately 1% of all tweets.

| Feature | Description |
|---------|-------------|
| **Endpoint** | `GET /2/tweets/sample/stream` |
| **Access Level** | Basic, Pro, Enterprise |
| **Sample Rate** | ~1% of all tweets |
| **Filtering** | None (random sample) |

#### Streaming Response Format

```json
{
  "data": {
    "id": "1234567890123456789",
    "text": "This is a streaming tweet example",
    "created_at": "2024-01-15T10:30:00.000Z"
  },
  "includes": {
    "users": [{
      "id": "987654321",
      "username": "example_user"
    }]
  },
  "matching_rules": [{
    "id": "1234567890",
    "tag": "Programming tweets"
  }]
}
```

### Threads Streaming

Threads API does not currently offer streaming endpoints. Real-time updates are only available through webhooks.

#### Streaming Comparison Matrix

| Feature | X (Twitter) | Threads |
|---------|-------------|---------|
| **Real-time Streaming** | Yes | No |
| **Filtered Stream** | Yes | No |
| **Sample Stream** | Yes | No |
| **Firehose Access** | Enterprise only | N/A |
| **Reconnect Handling** | Built-in | N/A |
| **Backfill Support** | Pro/Enterprise | N/A |

---

## Direct Messages (X Only)

X API v2 provides comprehensive Direct Message (DM) functionality. Threads API does not support direct messaging.

### DM Object Model

Direct Messages are organized into conversations with events.

#### Conversation Object

| Field | Type | Description |
|-------|------|-------------|
| `conversation_id` | string | Unique conversation identifier |
| `type` | string | `one_to_one` or `group` |
| `participants` | array | User objects in conversation |
| `dm_events` | array | Message events in conversation |

#### DM Event Types

| Event Type | Description |
|------------|-------------|
| `MessageCreate` | New message sent |
| `ParticipantsJoin` | User joined group |
| `ParticipantsLeave` | User left group |

#### DM Event Object

```json
{
  "event_type": "MessageCreate",
  "id": "1234567890",
  "text": "Hello, this is a direct message",
  "sender_id": "987654321",
  "recipient_id": "123456789",
  "created_at": "2024-01-15T10:30:00.000Z",
  "attachments": {
    "media_keys": ["3_1234567890"]
  },
  "referenced_messages": [{
    "event_type": "MessageCreate",
    "id": "9876543210"
  }]
}
```

#### DM Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/2/dm_conversations` | GET | List conversations |
| `/2/dm_conversations/{id}` | GET | Get conversation |
| `/2/dm_conversations/{id}/dm_events` | GET | Get events |
| `/2/dm_conversations/with/{participant_id}` | POST | Create 1-1 DM |
| `/2/dm_conversations` | POST | Create group DM |
| `/2/dm_conversations/{id}/messages` | POST | Send message |

#### DM Media Attachments

| Media Type | Support | Max Size |
|------------|---------|----------|
| Image | Yes | 5MB |
| Video | Yes | 15MB |
| GIF | Yes | 15MB |

---

## Media Upload Models

Both platforms support media uploads but with different workflows and constraints.

### X (Twitter) Media Upload

X uses a chunked upload process for media files, especially for videos.

#### Upload Process

```
1. INIT    - Initialize upload, get media_id
2. APPEND  - Upload chunks (for videos/large files)
3. FINALIZE - Complete upload
4. STATUS  - Check processing status (videos)
```

#### Upload Endpoints

| Endpoint | Phase | Description |
|----------|-------|-------------|
| `/1.1/media/upload.json` | INIT | Start upload |
| `/1.1/media/upload.json` | APPEND | Add chunks |
| `/1.1/media/upload.json` | FINALIZE | Complete upload |
| `/1.1/media/upload.json` | STATUS | Check status |

#### INIT Request

```json
{
  "command": "INIT",
  "media_type": "video/mp4",
  "total_bytes": 10485760,
  "media_category": "tweet_video",
  "additional_owners": ["123456789"]
}
```

#### INIT Response

```json
{
  "media_id": "1234567890123456789",
  "media_id_string": "1234567890123456789",
  "expires_after_secs": 86400
}
```

#### Media Categories

| Category | Description |
|----------|-------------|
| `tweet_image` | Image for tweet |
| `tweet_video` | Video for tweet |
| `tweet_gif` | Animated GIF |
| `dm_image` | Image for DM |
| `dm_video` | Video for DM |
| `subtitles` | Video subtitles |

### Threads Media Upload

Threads uses a container-based approach with separate upload and publish steps.

#### Upload Workflow

```
1. Create media container with image_url or video_url
2. Container processes asynchronously
3. Check container status
4. Publish container as thread
```

#### Create Media Container

```http
POST /{user-id}/threads?media_type=IMAGE&image_url=https://...
POST /{user-id}/threads?media_type=VIDEO&video_url=https://...
```

#### Container Response

```json
{
  "id": "17944322110154832"
}
```

#### Check Container Status

```http
GET /{container-id}?fields=status,status_code
```

#### Status Response

```json
{
  "id": "17944322110154832",
  "status": "FINISHED",
  "status_code": "PUBLISHED"
}
```

#### Container Status Codes

| Status Code | Description |
|-------------|-------------|
| `IN_PROGRESS` | Processing |
| `FINISHED` | Ready to publish |
| `PUBLISHED` | Successfully published |
| `ERROR` | Processing failed |
| `EXPIRED` | Container expired |
| `ARCHIVED` | Archived |

#### Carousel Upload

For carousel posts, create individual containers then combine:

```http
POST /{user-id}/threads?media_type=CAROUSEL&children=<container1>,<container2>
```

#### Media Upload Comparison

| Feature | X (Twitter) | Threads |
|---------|-------------|---------|
| **Upload Method** | Chunked upload | URL-based container |
| **Direct Upload** | Yes | No (URL required) |
| **Max Image Size** | 5MB | 8MB |
| **Max Video Size** | 512MB | N/A |
| **Max Video Duration** | 140s | 90s |
| **Alt Text** | Yes | Limited |
| **Subtitles** | Yes | No |
| **Processing Status** | Polling required | Polling required |

---

## Search and Filtering

Search capabilities differ significantly between platforms, with X offering more advanced query options.

### X (Twitter) Search

X API v2 provides comprehensive search functionality with query operators.

#### Search Endpoints

| Endpoint | Access | Description |
|----------|--------|-------------|
| `/2/tweets/search/recent` | Basic+ | Last 7 days |
| `/2/tweets/search/all` | Enterprise | Full archive |

#### Search Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `query` | string | Search query |
| `max_results` | integer | Results per page (10-100) |
| `next_token` | string | Pagination token |
| `start_time` | string | ISO 8601 start time |
| `end_time` | string | ISO 8601 end time |
| `sort_order` | string | `relevancy` or `recency` |

#### Query Operators

| Operator | Syntax | Example |
|----------|--------|---------|
| Keyword | `word` | `python` |
| Exact phrase | `"phrase"` | `"data science"` |
| Hashtag | `#tag` | `#AI` |
| Mention | `@user` | `@openai` |
| From user | `from:user` | `from:elonmusk` |
| To user | `to:user` | `to:twitter` |
| Exclude | `-keyword` | `-spam` |
| OR | `OR` | `python OR javascript` |
| AND | (space) | `python javascript` |
| Has media | `has:media` | `has:images` |
| Has links | `has:links` | `has:links` |
| Is retweet | `is:retweet` | `is:retweet` |
| Is reply | `is:reply` | `is:reply` |
| Is verified | `is:verified` | `is:verified` |
| Language | `lang:en` | `lang:en` |
| Place | `place:country:us` | `place:country:us` |

#### Search Query Examples

```
# Recent AI tweets from verified users
AI is:verified -is:retweet

# Python tutorials excluding retweets
python tutorial -is:retweet has:links

# Tweets from specific user mentioning AI
from:sama AI OR "artificial intelligence"

# News about a topic
"breaking news" (#tech OR #science)
```

### Threads Search

Threads API does not currently provide public search endpoints. Search is only available through the native app.

#### Search Comparison Matrix

| Feature | X (Twitter) | Threads |
|---------|-------------|---------|
| **Search API** | Yes | No |
| **Full-text Search** | Yes | N/A |
| **Hashtag Search** | Yes | N/A |
| **User Search** | Yes | No |
| **Date Range** | Yes | N/A |
| **Geographic** | Yes | N/A |
| **Advanced Operators** | Yes | N/A |
| **Sort Options** | Yes | N/A |

---

## Rate Limiting Details

Understanding rate limits is crucial for building robust applications on both platforms.

### X (Twitter) Rate Limits

X API v2 uses tiered rate limiting based on account level and endpoint.

#### Rate Limit Headers

| Header | Description |
|--------|-------------|
| `x-rate-limit-limit` | Maximum requests per window |
| `x-rate-limit-remaining` | Requests remaining in window |
| `x-rate-limit-reset` | Unix timestamp when limit resets |

#### Rate Limits by Tier

| Endpoint | Basic | Pro | Enterprise |
|----------|-------|-----|------------|
| **Tweet Lookup** | 300/15min | 900/15min | Unlimited |
| **Recent Search** | 60/15min | 300/15min | Unlimited |
| **User Timeline** | 900/15min | 1500/15min | Unlimited |
| **Post Tweet** | 50/24hr | 300/3hr | Custom |
| **User Lookup** | 300/15min | 900/15min | Unlimited |
| **Followers** | 15/15min | 60/15min | Custom |
| **Filtered Stream** | 50 rules | 1000 rules | Unlimited |

#### Tweet Creation Limits

| Limit Type | Basic | Pro |
|------------|-------|-----|
| Posts per 24 hours | 50 | 300 |
| Posts per 3 hours | - | 300 |
| Replies per 24 hours | 50 | 300 |
| Retweets per 24 hours | 50 | 300 |

### Threads Rate Limits

Threads API uses CPU time-based rate limiting, measured in compute units.

#### Rate Limit Structure

| Metric | Limit |
|--------|-------|
| **CPU Time** | App-level quota |
| **Request Rate** | Varies by endpoint |
| **User-level** | Per-user limits apply |

#### Rate Limit Response Headers

| Header | Description |
|--------|-------------|
| `X-App-Usage` | App-level usage info |
| `X-Ad-Account-Usage` | Ad account usage |

#### Usage Object

```json
{
  "call_count": 85,
  "total_cputime": 25,
  "total_time": 25,
  "estimated_total": 100
}
```

#### Rate Limit Comparison

| Feature | X (Twitter) | Threads |
|---------|-------------|---------|
| **Limit Type** | Request count | CPU time |
| **Reset Window** | 15 minutes | Rolling |
| **Headers** | Detailed | Summary |
| **Enterprise** | Unlimited | Custom |
| **Backoff** | Required | Recommended |

---

## User Relationships

X API provides extensive user relationship endpoints. Threads API has limited relationship functionality.

### X (Twitter) Relationships

#### Relationship Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/2/users/{id}/followers` | GET | Get followers |
| `/2/users/{id}/following` | GET | Get following |
| `/2/users/{source_id}/following/{target_id}` | GET | Check follow status |
| `/2/users/{id}/following` | POST | Follow user |
| `/2/users/{source_id}/following/{target_id}` | DELETE | Unfollow user |
| `/2/users/{id}/blocking` | GET | Get blocked users |
| `/2/users/{id}/blocking` | POST | Block user |
| `/2/users/{source_id}/blocking/{target_id}` | DELETE | Unblock user |
| `/2/users/{id}/muting` | GET | Get muted users |
| `/2/users/{id}/muting` | POST | Mute user |
| `/2/users/{source_id}/muting/{target_id}` | DELETE | Unmute user |

#### Follow Response

```json
{
  "data": {
    "following": true,
    "pending_follow": false
  }
}
```

#### Relationship Object

```json
{
  "relationship": {
    "source": {
      "id": "123456789",
      "id_str": "123456789",
      "screen_name": "user1",
      "following": true,
      "followed_by": true,
      "blocking": false,
      "muting": false,
      "can_dm": true
    },
    "target": {
      "id": "987654321",
      "id_str": "987654321",
      "screen_name": "user2",
      "following": true,
      "followed_by": true
    }
  }
}
```

### Threads Relationships

Threads API provides minimal relationship functionality.

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/me` | GET | Get own profile |
| `/{user-id}` | GET | Get user profile |

#### Relationship Comparison

| Feature | X (Twitter) | Threads |
|---------|-------------|---------|
| **Follow/Unfollow** | Yes | No |
| **Block/Unblock** | Yes | No |
| **Mute/Unmute** | Yes | No |
| **Get Followers** | Yes | No |
| **Get Following** | Yes | No |
| **Check Relationship** | Yes | No |

---

## Webhooks and Real-time Events

Webhooks enable real-time notifications of platform events.

### X (Twitter) Webhooks (Account Activity API)

X provides the Account Activity API for real-time event notifications.

#### Webhook Event Types

| Event Type | Description |
|------------|-------------|
| `tweet_create_events` | New tweets |
| `favorite_events` | Likes |
| `follow_events` | Follows |
| `block_events` | Blocks |
| `mute_events` | Mutes |
| `direct_message_events` | DMs |
| `direct_message_indicate_typing_events` | Typing indicators |
| `direct_message_mark_read_events` | Read receipts |
| `user_event` | User updates |

#### Webhook Registration

```http
POST /1.1/account_activity/all/{env_name}/webhooks.json?url=https://...
```

#### Webhook Challenge-Response

```
GET /webhook?crc_token=xxx
Response: {"response_token": "sha256=xxx"}
```

#### Webhook Payload

```json
{
  "tweet_create_events": [{
    "id": "1234567890123456789",
    "text": "Hello world",
    "user": {
      "id": "987654321",
      "screen_name": "example_user"
    }
  }],
  "user_has_blocked": false
}
```

### Threads Webhooks

Threads supports webhooks for real-time notifications through Meta's platform.

#### Supported Webhook Fields

| Field | Description |
|-------|-------------|
| `threads_id` | User ID |
| `threads_username` | Username |
| `threads_profile_picture_url` | Profile picture |
| `threads_biography` | Bio text |

#### Webhook Verification

```http
GET /webhook?hub.mode=subscribe&hub.challenge=xxx&hub.verify_token=xxx
Response: hub.challenge value
```

#### Webhook Comparison

| Feature | X (Twitter) | Threads |
|---------|-------------|---------|
| **Webhook Support** | Yes | Yes |
| **Real-time Events** | Yes | Limited |
| **Tweet Events** | Yes | N/A |
| **DM Events** | Yes | No |
| **User Events** | Yes | Limited |
| **Challenge-Response** | Yes | Yes |

---

## Content Publishing Workflows

Both platforms offer content creation APIs with different workflows.

### X (Twitter) Publishing

#### Single Tweet

```http
POST /2/tweets
Content-Type: application/json

{
  "text": "Hello world!",
  "media": {
    "media_ids": ["1234567890"]
  },
  "reply_settings": "mentionedUsers"
}
```

#### Reply Tweet

```json
{
  "text": "Great point!",
  "reply": {
    "in_reply_to_tweet_id": "9876543210987654321"
  }
}
```

#### Quote Tweet

```json
{
  "text": "My commentary",
  "quote_tweet_id": "9876543210987654321"
}
```

#### Thread Creation

```json
[
  {"text": "Thread part 1"},
  {"text": "Thread part 2"},
  {"text": "Thread part 3"}
]
```

#### Tweet Response

```json
{
  "data": {
    "id": "1234567890123456789",
    "text": "Hello world!",
    "edit_history_tweet_ids": ["1234567890123456789"]
  }
}
```

### Threads Publishing

#### Publishing Workflow

```
1. Create media container
2. Wait for processing
3. Publish container
```

#### Text Post

```http
POST /{user-id}/threads
  ?media_type=TEXT
  &text=Hello%20world
```

#### Image Post

```http
POST /{user-id}/threads
  ?media_type=IMAGE
  &image_url=https://example.com/image.jpg
  &text=Check%20this%20out
```

#### Video Post

```http
POST /{user-id}/threads
  ?media_type=VIDEO
  &video_url=https://example.com/video.mp4
  &text=My%20video
```

#### Carousel Post

```http
POST /{user-id}/threads
  ?media_type=CAROUSEL
  &children=17944322110154830,17944322110154831
  &text=Carousel%20post
```

#### Reply Post

```http
POST /{user-id}/threads
  ?media_type=TEXT
  &text=Reply
  &reply_to_id=17944322110154830
```

#### Publish Container

```http
POST /{user-id}/threads_publish
  ?creation_id=17944322110154832
```

#### Publishing Comparison

| Feature | X (Twitter) | Threads |
|---------|-------------|---------|
| **Direct Publish** | Yes | No (2-step) |
| **Text Limit** | 280/4000 | 500 |
| **Image Limit** | 4 | 10 |
| **Video Limit** | 1 | 1 |
| **Carousel** | No | Yes |
| **Reply Settings** | Yes | Yes |
| **Quote Post** | Yes | Yes |
| **Edit Support** | Yes | No |

---

## Batch Operations

X API supports batch operations for efficient data retrieval and actions.

### X (Twitter) Batch Operations

#### Batch Tweet Lookup

```http
GET /2/tweets?ids=123,456,789&tweet.fields=created_at,author_id
```

#### Batch User Lookup

```http
GET /2/users?ids=123,456,789&user.fields=created_at,public_metrics
```

#### Batch User Lookup by Username

```http
GET /2/users/by?usernames=user1,user2,user3
```

#### Batch Limits

| Operation | Max IDs | Max Usernames |
|-----------|---------|---------------|
| Tweet Lookup | 100 | N/A |
| User Lookup | 100 | 100 |
| User by Username | N/A | 100 |

### Threads Batch Operations

Threads API does not support batch operations. Each request must be made individually.

#### Batch Comparison

| Feature | X (Twitter) | Threads |
|---------|-------------|---------|
| **Batch Tweet Lookup** | Yes (100) | No |
| **Batch User Lookup** | Yes (100) | No |
| **Batch Post** | No | No |
| **Batch Delete** | No | No |

---

## Additional X (Twitter) Features

### Lists

X supports user-created lists for organizing accounts and tweets.

#### List Object

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | List ID |
| `name` | string | List name |
| `description` | string | List description |
| `member_count` | integer | Members count |
| `follower_count` | integer | Followers count |
| `private` | boolean | Is private |
| `owner_id` | string | Owner user ID |

#### List Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/2/lists` | POST | Create list |
| `/2/lists/{id}` | DELETE | Delete list |
| `/2/lists/{id}` | PUT | Update list |
| `/2/users/{id}/owned_lists` | GET | Get owned lists |
| `/2/lists/{id}/tweets` | GET | Get list tweets |
| `/2/lists/{id}/members` | GET | Get list members |
| `/2/lists/{id}/members` | POST | Add member |
| `/2/lists/{id}/members/{user_id}` | DELETE | Remove member |

### Spaces

X Spaces are live audio conversations.

#### Space Object

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Space ID |
| `title` | string | Space title |
| `state` | string | `live`, `scheduled`, `ended` |
| `creator_id` | string | Creator user ID |
| `host_ids` | array | Host user IDs |
| `speaker_ids` | array | Speaker user IDs |
| `subscriber_count` | integer | Subscriber count |
| `participant_count` | integer | Participant count |
| `is_ticketed` | boolean | Is ticketed |
| `start_date` | string | Start time |
| `scheduled_start` | string | Scheduled start |

#### Space Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/2/spaces/{id}` | GET | Get space |
| `/2/spaces` | GET | Lookup spaces |
| `/2/spaces/search` | GET | Search spaces |
| `/2/spaces/{id}/buyers` | GET | Get ticket buyers |
| `/2/spaces/{id}/tweets` | GET | Get related tweets |

### Community Notes (Birdwatch)

Community Notes enable crowd-sourced fact-checking.

#### Note Object

| Field | Type | Description |
|-------|------|-------------|
| `note_id` | string | Note ID |
| `tweet_id` | string | Associated tweet |
| `text` | string | Note text |
| `classification` | string | Note classification |
| `created_at` | string | Creation time |

### Bookmarks

Users can bookmark tweets for later reference.

#### Bookmark Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/2/users/{id}/bookmarks` | GET | Get bookmarks |
| `/2/users/{id}/bookmarks` | POST | Add bookmark |
| `/2/users/{id}/bookmarks/{tweet_id}` | DELETE | Remove bookmark |

---

## Error Handling Deep Dive

### X (Twitter) Error Types

#### HTTP Status Codes

| Code | Type | Description |
|------|------|-------------|
| 400 | Bad Request | Invalid parameters |
| 401 | Unauthorized | Missing/invalid auth |
| 403 | Forbidden | Access denied |
| 404 | Not Found | Resource missing |
| 429 | Too Many Requests | Rate limited |
| 500 | Server Error | Internal error |
| 503 | Service Unavailable | Overloaded |

#### Error Problem Types

| Type | Description |
|------|-------------|
| `invalid-request` | Malformed request |
| `client-forbidden` | Client lacks permission |
| `resource-not-found` | Resource doesn't exist |
| `usage-capped` | Rate limit or cap |
| `unsupported-authentication` | Auth method not allowed |
| `unprocessable-entity` | Validation failed |

#### Error Response Structure

```json
{
  "errors": [{
    "parameters": {
      "ids": ["invalid_id"]
    },
    "message": "The `ids` query parameter value [invalid_id] is not valid"
  }],
  "title": "Invalid Request",
  "detail": "One or more parameters to your request was invalid.",
  "type": "https://api.twitter.com/2/problems/invalid-request"
}
```

### Threads Error Types

#### Error Response Structure

```json
{
  "error": {
    "message": "Error message",
    "type": "OAuthException",
    "code": 190,
    "error_subcode": 463,
    "error_user_msg": "User-friendly message",
    "fbtrace_id": "ABC123"
  }
}
```

#### Common Error Codes

| Code | Subcode | Description |
|------|---------|-------------|
| 1 | - | Unknown error |
| 2 | - | API temporarily unavailable |
| 4 | - | API call limit exceeded |
| 17 | - | API call limit exceeded |
| 32 | - | Page permission error |
| 100 | - | Invalid parameter |
| 190 | - | Access token invalid |
| 190 | 459 | Session expired |
| 190 | 460 | Session revoked |
| 190 | 463 | Token expired |
| 200 | - | Missing permissions |
| 368 | - | Temporarily blocked |
| 506 | - | Duplicate post |

---

## References

- [X API v2 Documentation](https://docs.x.com/x-api/introduction)
- [X API Data Dictionary](https://docs.x.com/x-api/fundamentals/data-dictionary)
- [X API Pagination](https://docs.x.com/x-api/fundamentals/pagination)
- [X API Streaming](https://docs.x.com/x-api/streams/introduction)
- [X Direct Messages](https://docs.x.com/x-api/direct-messages/manage/introduction)
- [Threads API Documentation](https://developers.facebook.com/docs/threads)
- [Threads API Overview](https://developers.facebook.com/docs/threads/overview)
- [Threads API Posts](https://developers.facebook.com/docs/threads/posts)
- [Threads API Insights](https://developers.facebook.com/docs/threads/insights)
- [Threads Webhooks](https://developers.facebook.com/docs/threads/threads-webhooks)
- [Meta Graph API Reference](https://developers.facebook.com/docs/graph-api/)

---

*Document generated: January 2025*
*Last updated: Based on official API documentation as of research date*
