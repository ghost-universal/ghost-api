//! X HTML selectors for scraping
//!
//! CSS selectors and data-testid attributes for X.com

/// CSS selectors for X.com elements
pub struct XSelectors;

impl XSelectors {
    /// Selector for tweet container
    pub const TWEET_CONTAINER: &'static str = "[data-testid='tweet']";

    /// Selector for tweet text
    pub const TWEET_TEXT: &'static str = "[data-testid='tweetText']";

    /// Selector for user avatar
    pub const USER_AVATAR: &'static str = "[data-testid='Tweet-User-Avatar']";

    /// Selector for like button
    pub const LIKE_BUTTON: &'static str = "[data-testid='like']";

    /// Selector for retweet button
    pub const RETWEET_BUTTON: &'static str = "[data-testid='retweet']";

    /// Selector for reply button
    pub const REPLY_BUTTON: &'static str = "[data-testid='reply']";

    /// Selector for user name
    pub const USER_NAME: &'static str = "[data-testid='UserName']";

    /// Selector for user description/bio
    pub const USER_DESCRIPTION: &'static str = "[data-testid='UserDescription']";

    /// Selector for profile image
    pub const PROFILE_IMAGE: &'static str = "[data-testid='profile-image']";

    /// Selector for followers count
    pub const FOLLOWERS_COUNT: &'static str = "[data-testid='followers-count']";

    /// Selector for following count
    pub const FOLLOWING_COUNT: &'static str = "[data-testid='following-count']";

    /// Selector for verified badge
    pub const VERIFIED_BADGE: &'static str = "[data-testid='verification-badge']";

    /// Selector for media image
    pub const MEDIA_IMAGE: &'static str = "[data-testid='tweetPhoto']";

    /// Selector for media video
    pub const MEDIA_VIDEO: &'static str = "[data-testid='videoComponent']";

    /// Selector for trending section
    pub const TRENDING_SECTION: &'static str = "[data-testid='trend']";

    /// Selector for search box
    pub const SEARCH_BOX: &'static str = "[data-testid='SearchBox_Search_Input']";

    /// Selector for timeline
    pub const TIMELINE: &'static str = "[data-testid='primaryColumn']";

    /// Returns all selectors for testing
    pub fn all() -> Vec<(&'static str, &'static str)> {
        // TODO: Implement selector enumeration
        vec![
            ("tweet_container", Self::TWEET_CONTAINER),
            ("tweet_text", Self::TWEET_TEXT),
            ("user_avatar", Self::USER_AVATAR),
            ("like_button", Self::LIKE_BUTTON),
            ("retweet_button", Self::RETWEET_BUTTON),
            ("reply_button", Self::REPLY_BUTTON),
        ]
    }
}
