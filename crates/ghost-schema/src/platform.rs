//! Platform definitions and platform-specific types
//!
//! This module contains platform identifiers and platform-specific
//! configurations for anti-detection and routing.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::GhostError;

/// Supported social media platforms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    /// X (formerly Twitter)
    X,
    /// Threads (Meta)
    Threads,
    /// Unknown/unsupported platform
    #[default]
    Unknown,
}

impl Platform {
    /// Returns the display name of the platform
    pub fn display_name(&self) -> &'static str {
        // TODO: Implement platform display name mapping
        match self {
            Platform::X => "X (Twitter)",
            Platform::Threads => "Threads",
            Platform::Unknown => "Unknown",
        }
    }

    /// Returns the base URL for the platform
    pub fn base_url(&self) -> &'static str {
        // TODO: Implement platform base URL mapping
        match self {
            Platform::X => "https://x.com",
            Platform::Threads => "https://www.threads.net",
            Platform::Unknown => "",
        }
    }

    /// Returns the API base URL for the platform
    pub fn api_url(&self) -> &'static str {
        // TODO: Implement platform API URL mapping
        match self {
            Platform::X => "https://api.twitter.com",
            Platform::Threads => "https://graph.threads.net",
            Platform::Unknown => "",
        }
    }

    /// Returns whether this platform requires authentication for basic operations
    pub fn requires_auth(&self) -> bool {
        // TODO: Implement platform auth requirement determination
        match self {
            Platform::X => true,
            Platform::Threads => true,
            Platform::Unknown => false,
        }
    }

    /// Returns the adapter crate name for this platform
    pub fn adapter_name(&self) -> &'static str {
        // TODO: Implement adapter name mapping
        match self {
            Platform::X => "x-adapter",
            Platform::Threads => "threads-adapter",
            Platform::Unknown => "",
        }
    }

    /// Returns the default rate limit window (requests per 15 min)
    pub fn default_rate_limit(&self) -> u32 {
        // TODO: Implement platform rate limit defaults
        match self {
            Platform::X => 450,
            Platform::Threads => 200,
            Platform::Unknown => 1000,
        }
    }

    /// Returns the recommended jitter range in milliseconds
    pub fn jitter_range_ms(&self) -> (u64, u64) {
        // TODO: Implement platform-specific jitter recommendations
        match self {
            Platform::X => (500, 3000),
            Platform::Threads => (800, 4000),
            Platform::Unknown => (100, 500),
        }
    }

    /// Parses a platform from a string (convenience method)
    pub fn parse(s: &str) -> Self {
        // TODO: Implement platform parsing with normalization
        match s.to_lowercase().as_str() {
            "x" | "twitter" => Platform::X,
            "threads" => Platform::Threads,
            _ => Platform::Unknown,
        }
    }

    /// Returns all supported platforms
    pub fn all() -> Vec<Platform> {
        // TODO: Implement platform enumeration
        vec![Platform::X, Platform::Threads]
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl std::str::FromStr for Platform {
    type Err = GhostError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO: Implement proper platform parsing with error
        Ok(Self::parse(s))
    }
}

/// Platform-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    /// The platform this config applies to
    pub platform: Platform,
    /// JA3 TLS fingerprint profile
    pub fingerprint: String,
    /// Header entropy profile
    pub header_profile: String,
    /// Jitter range in milliseconds [min, max]
    pub jitter_range_ms: (u64, u64),
    /// Whether to respect Retry-After headers
    pub respect_retry_after: bool,
    /// Custom headers to include in all requests
    pub custom_headers: HashMap<String, String>,
    /// User agent string
    pub user_agent: Option<String>,
}

impl PlatformConfig {
    /// Creates a new platform config with defaults
    pub fn new(platform: Platform) -> Self {
        // TODO: Implement platform config construction with sensible defaults
        let (jitter_min, jitter_max) = platform.jitter_range_ms();
        Self {
            platform,
            fingerprint: "chrome_120".to_string(),
            header_profile: "desktop_windows".to_string(),
            jitter_range_ms: (jitter_min, jitter_max),
            respect_retry_after: true,
            custom_headers: HashMap::new(),
            user_agent: None,
        }
    }

    /// Validates the configuration
    pub fn validate(&self) -> Result<(), GhostError> {
        // TODO: Implement config validation
        if self.jitter_range_ms.0 > self.jitter_range_ms.1 {
            return Err(GhostError::ValidationError(
                "jitter_range_ms min must be <= max".into(),
            ));
        }
        Ok(())
    }

    /// Returns a random jitter delay within the configured range
    pub fn random_jitter(&self) -> std::time::Duration {
        // TODO: Implement random jitter generation
        std::time::Duration::from_millis(self.jitter_range_ms.0)
    }

    /// Generates platform-specific headers for a request
    pub fn generate_headers(&self) -> HashMap<String, String> {
        // TODO: Implement header generation with platform-specific entropy
        let mut headers = self.custom_headers.clone();
        headers.insert(
            "User-Agent".to_string(),
            self.user_agent.clone().unwrap_or_default(),
        );
        headers
    }
}

impl Default for PlatformConfig {
    fn default() -> Self {
        Self::new(Platform::Unknown)
    }
}

/// Platform shield configuration for anti-detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformShield {
    /// The platform this shield applies to
    pub platform: Platform,
    /// JA3/H2 fingerprint configuration
    pub tls_fingerprint: TlsFingerprint,
    /// Header entropy configuration
    pub header_entropy: HeaderEntropy,
    /// Rate limit behavior
    pub rate_limit_behavior: RateLimitBehavior,
    /// Challenge/WAF handling
    pub challenge_handling: ChallengeHandling,
}

impl PlatformShield {
    /// Creates a new platform shield with defaults
    pub fn new(platform: Platform) -> Self {
        // TODO: Implement platform shield construction
        Self {
            platform,
            tls_fingerprint: TlsFingerprint::default(),
            header_entropy: HeaderEntropy::default(),
            rate_limit_behavior: RateLimitBehavior::default(),
            challenge_handling: ChallengeHandling::default(),
        }
    }

    /// Validates the shield configuration
    pub fn validate(&self) -> Result<(), GhostError> {
        // TODO: Implement shield validation
        Ok(())
    }

    /// Detects if a response indicates a WAF challenge
    pub fn detect_challenge(&self, _response: &crate::PayloadBlob) -> Option<ChallengeType> {
        // TODO: Implement challenge detection logic
        None
    }

    /// Generates countermeasures for detected challenges
    pub fn generate_countermeasures(&self, _challenge: ChallengeType) -> Vec<Countermeasure> {
        // TODO: Implement countermeasure generation
        Vec::new()
    }
}

impl Default for PlatformShield {
    fn default() -> Self {
        Self::new(Platform::Unknown)
    }
}

/// TLS fingerprint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsFingerprint {
    /// Browser profile to mimic
    pub browser_profile: BrowserProfile,
    /// Whether to randomize cipher order
    pub randomize_ciphers: bool,
    /// Custom JA3 string (if provided, overrides profile)
    pub custom_ja3: Option<String>,
}

impl Default for TlsFingerprint {
    fn default() -> Self {
        Self {
            browser_profile: BrowserProfile::Chrome120,
            randomize_ciphers: false,
            custom_ja3: None,
        }
    }
}

/// Browser profiles for fingerprinting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BrowserProfile {
    Chrome120,
    Chrome119,
    Firefox121,
    Safari17,
    Edge120,
    Custom,
}

impl BrowserProfile {
    /// Returns the JA3 string for this profile
    pub fn ja3_string(&self) -> Option<&'static str> {
        // TODO: Implement JA3 string lookup
        None
    }
}

/// Header entropy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderEntropy {
    /// Header profile to use
    pub profile: HeaderProfile,
    /// Whether to randomize header order
    pub randomize_order: bool,
    /// Whether to add decoy headers
    pub add_decoys: bool,
}

impl Default for HeaderEntropy {
    fn default() -> Self {
        Self {
            profile: HeaderProfile::DesktopWindows,
            randomize_order: true,
            add_decoys: false,
        }
    }
}

/// Header profiles for different platforms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeaderProfile {
    DesktopWindows,
    DesktopMacos,
    DesktopLinux,
    MobileIos,
    MobileAndroid,
    Custom,
}

/// Rate limit behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitBehavior {
    /// Whether to automatically retry on rate limit
    pub auto_retry: bool,
    /// Maximum retry attempts
    pub max_retries: u32,
    /// Base delay for exponential backoff (ms)
    pub base_delay_ms: u64,
    /// Maximum delay cap (ms)
    pub max_delay_ms: u64,
    /// Whether to honor Retry-After headers
    pub honor_retry_after: bool,
}

impl Default for RateLimitBehavior {
    fn default() -> Self {
        Self {
            auto_retry: true,
            max_retries: 3,
            base_delay_ms: 1000,
            max_delay_ms: 60000,
            honor_retry_after: true,
        }
    }
}

/// Challenge/WAF handling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeHandling {
    /// Whether to auto-pivot to browser-based on challenge
    pub auto_pivot: bool,
    /// Timeout for solving challenges (ms)
    pub solve_timeout_ms: u64,
    /// Maximum challenge attempts before giving up
    pub max_attempts: u32,
}

impl Default for ChallengeHandling {
    fn default() -> Self {
        Self {
            auto_pivot: true,
            solve_timeout_ms: 30000,
            max_attempts: 3,
        }
    }
}

/// Types of challenges that can be encountered
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChallengeType {
    Cloudflare,
    HCaptcha,
    Recaptcha,
    Turnstile,
    JsChallenge,
    LoginRequired,
    Unknown,
}

/// Countermeasures for challenges
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Countermeasure {
    PivotToBrowser,
    RotateProxy,
    RotateSession,
    InjectCookies {
        cookies: String,
    },
    SolveChallenge {
        challenge_type: String,
    },
    WaitAndRetry {
        delay_ms: u64,
    },
}
