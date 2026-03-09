# `ghost-api`

[![Build Status](https://img.shields.io/github/actions/workflow/status/your-org/ghost-api/ci.yml?style=flat-square)](https://github.com/your-org/ghost-api/actions)
[![Crates.io](https://img.shields.io/crates/v/ghost-api.svg?style=flat-square)](https://crates.io/crates/ghost-api)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](https://opensource.org/licenses/MIT)
[![Memory Safety: 100%](https://img.shields.io/badge/memory--safety-100%25-success?style=flat-square)]()

**The unified programmatic bridge for X (Twitter) & Threads.** 

Official APIs are extortionate; open-source scrapers are brittle. `ghost-api` is a type-safe Rust orchestration engine that wraps polyglot scrapers (Python, TS, Go) into a single interface with health-based routing and per-request multi-tenant injection.

---

## Table of Contents

### Getting Started
- [1. The One-Line Pitch](#1-the-one-line-pitch)
- [2. Core Value Proposition](#2-core-value-proposition)
- [3. Core Concepts: Scraper vs Adapter](#3-core-concepts-scraper-vs-adapter)
- [4. Project Structure](#4-project-structure)
- [5. Architecture & Request Lifecycle](#5-architecture--request-lifecycle)
- [6. Key Concepts & Terminology](#6-key-concepts--terminology)
- [7. System Requirements](#7-system-requirements)
- [8. Installation](#8-installation)
- [9. Quick Start (Programmatic API)](#9-quick-start-programmatic-api)
- [10. Multi-Tenancy Configuration](#10-multi-tenancy-configuration)

### Integration & Architecture
- [11. Web Console & Debugging](#11-web-console--debugging)
- [12. GhostWorker Integration Protocol](#12-ghostworker-integration-protocol)
- [13. Context-Aware Multi-Tenancy (BYO-Injection)](#13-context-aware-multi-tenancy-byo-injection)
- [14. Health Scoring & Circuit Breaking](#14-health-scoring--circuit-breaking)
- [15. Fallback Hierarchy (The Tiered Cascade)](#15-fallback-hierarchy-the-tiered-cascade)
- [16. Observability & Performance Tuning](#16-observability--performance-tuning)
- [17. Proxy Orchestration (Protocol-Agnostic Tunneling)](#17-proxy-orchestration-protocol-agnostic-tunneling)
- [18. The Rust SDK: Direct Type-Safe Access](#18-the-rust-sdk-direct-type-safe-access)

### Platform & Security
- [19. Unified Platform AST (Abstract Syntax Tree)](#19-unified-platform-ast-abstract-syntax-tree)
- [20. The "Platform Shield" (Handling Hostility)](#20-the-platform-shield-handling-hostility)
- [21. Sandboxing & Runtime Isolation](#21-sandboxing--runtime-isolation)
- [22. Production Topologies & Deployment](#22-production-topologies--deployment)

### Operations & Scaling
- [23. Scraper Pool Auto-Scaling](#23-scraper-pool-auto-scaling)
- [24. Cost Attribution & Budget Controls](#24-cost-attribution--budget-controls)
- [25. Session Health Monitoring](#25-session-health-monitoring)
- [26. Credential Vault Integration](#26-credential-vault-integration)
- [27. Scaling the Scraper Pool (Developer SDK)](#27-scaling-the-scraper-pool-developer-sdk)

### Testing & Troubleshooting
- [28. Simulation & Mocking (The "Shadow Graph")](#28-simulation--mocking-the-shadow-graph)
- [29. High-Throughput Batching & Backpressure](#29-high-throughput-batching--backpressure)
- [30. Troubleshooting & "Ban-Hammer" Diagnosis](#30-troubleshooting--ban-hammer-diagnosis)
- [31. Ethical Scraping & Rate-Limit Etiquette](#31-ethical-scraping--rate-limit-etiquette)

### Community & License
- [32. Roadmap: The Multi-Platform Future](#32-roadmap-the-multi-platform-future)
- [33. Community & Support](#33-community--support)
- [34. License & Contributor Agreement](#34-license--contributor-agreement)

---

## 1. The One-Line Pitch

A capability-driven routing mesh that treats scrapers as "unreliable workers," orchestrating them with health-aware fallbacks and granular credential/proxy injection.

---

## 2. Core Value Proposition

*   **Programmatic-First:** Built as a Rust crate for direct integration. The HTTP/Swagger layer is an optional sidecar.
*   **Multi-Tenant Injection:** No global config for accounts. Pass `Session`, `Cookies`, and `Proxy` objects directly into the method call. 
*   **Polyglot Runtime:** Executes TS (via NAPI), Python (via PyO3), and Go (via gRPC) scrapers as if they were native Rust functions.
*   **Health-Based Auto-Route:** Real-time scoring of scraper pools. If a TS scraper returns a 429, the engine transparently re-routes the next attempt to a healthy Python worker or the Official API.
*   **Zero-Config Swagger:** Auto-generated via `utoipa`. Provides a "debug console" to manually test injection parameters without writing code.

---

## 3. Core Concepts: Scraper vs Adapter

Before diving into architecture, understand the key separation of concerns:

| Layer | Responsibility | Example |
|-------|----------------|---------|
| **Scraper** | The "How" — fetches raw data from platforms | Python Playwright, Node Puppeteer, Go HTTP client |
| **Adapter** | The "What" — translates platform-specific data to unified schema | `x-adapter`, `threads-adapter` |
| **Core** | The "Brain" — routing, health scoring, fallback logic | `ghost-core` |

**The Flow:**
```
Scraper (fetches raw HTML/JSON) 
    → Adapter (parses & normalizes) 
    → Core (returns unified GhostPost)
```

**Why this matters:** If X changes their CSS selectors, you update the `x-adapter`—not every scraper implementation.

---

## 4. Project Structure

`ghost-api` uses a workspace-first architecture, decoupling the Scraper from the Adapter.

```text
ghost-api/
├── Cargo.toml                  # Workspace manifest
├── crates/
│   ├── ghost-core/             # Routing & Health logic
│   ├── ghost-adapters/         # Platform-specific de-obfuscation
│   │   ├── x-adapter/          # Maps X's 'data-testid' and GraphQL to Ghost-AST
│   │   └── threads-adapter/    # Maps Threads' internal relay-style JSON to Ghost-AST
│   ├── ghost-bridge/           # Polyglot FFI (PyO3, NAPI)
│   ├── ghost-schema/           # Unified Social AST (Abstract Syntax Tree)
│   ├── ghost-server/           # Optional: Axum + Utoipa (Swagger UI) binary
│   └── ghost-vault/            # Context/Multi-tenancy: Proxy & Credential injection logic
├── scrapers/                   # The "Workers"
│   ├── node-agent/             # TS-based Puppeteer/Stealth scrapers
│   ├── py-stealth/             # Python-based Playwright/Request scrapers
│   └── go-client/              # Go-based RE clients
├── scripts/                    # Scraper dependency installers (npm/pip)
└── config/                     # Default capability manifests & health thresholds
```

---

## 5. Architecture & Request Lifecycle

Since X and Threads are closed/hostile platforms, the lifecycle includes a "De-obfuscation" phase. Everything is handled via the `GhostContext`.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         REQUEST LIFECYCLE                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│  1. CALL          api.x().get_user(id, ctx)                                  │
│         ↓                                                                    │
│  2. DISPATCH      ghost-core selects best Scraper (via health scoring)       │
│         ↓                                                                    │
│  3. EXTRACTION    Scraper returns PayloadBlob (raw HTML/JSON)                │
│         ↓                                                                    │
│  4. ADAPTATION    Adapter runs Heuristic Matcher to extract data             │
│         ↓                                                                    │
│  5. NORMALIZATION Platform quirks mapped to unified schema                   │
│         ↓                                                                    │
│  6. FALLBACK      (if failure) Retry with alternative scraper/tier           │
│         ↓                                                                    │
│  7. RETURN        Unified GhostPost struct                                   │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 6. Key Concepts & Terminology

| Term | Description |
|------|-------------|
| **GhostWorker** | The trait that scrapers implement to integrate with the engine. |
| **PayloadBlob** | Raw, unprocessed data returned by scrapers (HTML, JSON, etc.). |
| **GhostPost** | The unified struct representing a post across all platforms. |
| **Capability** | Defines what a worker can do (e.g., `X_SEARCH`, `THREADS_READ`). |
| **Injection** | Providing ephemeral state (Proxy/Cookies) at call-time. |
| **Circuit Breaker** | Automatic cooling-off periods for scrapers hitting rate limits. |
| **Health Score** | A weighted score (0.0 - 1.0) combining success rate and latency. |
| **Scraper Pivot** | Switching to a different scraper *within the same tier* on WAF detection. |
| **Tier Fallback** | Escalating to the *next tier* when current tier is exhausted. |

---

## 7. System Requirements

*   **Rust:** 1.78+
*   **Runtimes:** (Only if using respective scrapers)
    *   Node.js 20+ (for `ghost-bridge-ts`)
    *   Python 3.11+ (for `ghost-bridge-py`)
*   **Network:** Redis/Valkey (Optional: for distributed health state).

---

## 8. Installation

### Library (Rust Workspace)

```toml
[dependencies]
ghost-api = { version = "0.1", features = ["ts", "python", "x-adapter", "threads-adapter"] }
```

**Available Features:**

| Feature | Description |
|---------|-------------|
| `ts` | Enable Node.js/NAPI scraper bridge |
| `python` | Enable Python/PyO3 scraper bridge |
| `go` | Enable Go gRPC scraper bridge |
| `x-adapter` | X (Twitter) platform adapter |
| `threads-adapter` | Threads platform adapter |
| `server` | Include Axum HTTP server |

### Docker (All-in-one)

```bash
docker pull ghcr.io/your-org/ghost-api:latest
docker run -p 3000:3000 -v ./config:/etc/ghost-api ghost-api
```

---

## 9. Quick Start (Programmatic API)

The primary way to use `ghost-api` is through the Rust API.

```rust
use ghost_api::{Ghost, GhostContext, Strategy, GhostPost};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ghost = Ghost::init().await?;

    // Multi-tenant injection: Define per-request credentials and proxy
    let ctx = GhostContext::builder()
        .tenant_id("user_789")
        .proxy("socks5://user:pass@1.2.3.4:1080")
        .session("ct0=...; auth_token=...")
        .build();

    // Execute with health-aware routing
    // Returns unified GhostPost regardless of platform
    let post: GhostPost = ghost.x()
        .get_post("123456789", &ctx, Strategy::HealthFirst)
        .await?;

    println!("Fetched: {}", post.text);
    println!("Platform: {:?}", post.platform); // X | Threads
    Ok(())
}
```

---

## 10. Multi-Tenancy Configuration

`ghost-api` doesn't "store" users. It manages **Contexts**.

### Basic Configuration

```toml
# Default Config (ghost.toml)
[engine]
default_strategy = "health_first"
retry_count = 3

[scrapers.ts_agent]
enabled = true
capabilities = ["x_read", "threads_search"]
health_threshold = 0.7
max_concurrent = 5

[scrapers.py_stealth]
enabled = true
capabilities = ["x_read", "x_search", "threads_read"]
health_threshold = 0.6
max_concurrent = 3

[scrapers.official]
enabled = true
api_key = "${OFFICIAL_X_KEY}"
priority = 99  # Only used if everything else fails

# Platform Shield Configuration
[shield.x]
fingerprint = "chrome_120"           # JA3 fingerprint profile
header_profile = "desktop_windows"   # Header entropy profile
jitter_range_ms = [500, 3000]        # Random delay range

[shield.threads]
fingerprint = "chrome_120"
header_profile = "desktop_macos"
jitter_range_ms = [800, 4000]
```

### Parallel Multi-Tenant Usage

```rust
use futures::future::join_all;

// Run queries for multiple tenants in parallel
let tenants = vec![
    GhostContext::builder().tenant_id("user_1").session("...").build(),
    GhostContext::builder().tenant_id("user_2").session("...").build(),
    GhostContext::builder().tenant_id("user_3").session("...").build(),
];

let results: Vec<Result<GhostPost, _>> = join_all(
    tenants.iter().map(|ctx| ghost.x().get_post("123456789", ctx))
).await;

// Handle per-tenant results
for (i, result) in results.iter().enumerate() {
    match result {
        Ok(post) => println!("Tenant {}: {}", i, post.text),
        Err(e) => eprintln!("Tenant {} failed: {}", i, e),
    }
}
```

---

## 11. Web Console & Debugging

Running `ghost-api serve` spins up an auto-generated Swagger UI at `/swagger-ui`.

### Features

*   **Interactive Injection:** Paste a cookie string and immediately test health across different scrapers.
*   **Endpoint Controls:** Every endpoint includes optional headers for `X-Ghost-Proxy` and `X-Ghost-Session`.
*   **Health Dashboard:** The `/health` route visualizes which scrapers are alive with their latency/capability matrix.
*   **Live Metrics:** The `/metrics/viz` endpoint (optional) provides real-time worker activity and success/fail ratios.
*   **Manual Override:** Force a specific scraper (e.g., "Always use Python for this call") to debug scraper-specific issues.
*   **Curl Generator:** Test complex multi-tenant requests in the UI and "Copy as Curl" for programmatic use.
*   **Trace Tab:** See exactly which scraper handled the request and raw response headers for debugging "Shadowbans."

---

## 12. GhostWorker Integration Protocol

`ghost-api` treats external scrapers as untrusted black boxes. Scrapers return **PayloadBlobs** to Adapters for processing.

### The GhostWorker Contract

To add a new worker (Python/TS/Go), implement the `GhostWorker` trait:

```rust
pub trait GhostWorker: Send + Sync {
    /// Unique identifier for this worker
    fn id(&self) -> &str;
    
    /// Capabilities this worker supports
    fn capabilities(&self) -> Vec<Capability>;
    
    /// Execute a request and return a PayloadBlob
    async fn execute(&self, ctx: &RawContext) -> Result<PayloadBlob, GhostError>;
}
```

### Integration Points

*   **Capability Manifest:** Every worker exports a `manifest.json` defining its "Skills" (e.g., `X_SEARCH`, `THREADS_READ`). The engine ignores workers that don't explicitly claim a capability.
*   **The Bridge:**
    *   **TS/Node:** Shared memory via `napi-rs`. Zero-copy buffers for high-volume media scraping.
    *   **Python:** In-process execution via `PyO3`. Perfect for `Playwright-python` integration.
    *   **Go/Other:** Lightweight Unix Domain Sockets (UDS) with Protobuf encoding.
*   **Virtual DOM Mapping:** Adapters use CSS-selector-versioning. If X changes a class name, you update the Adapter's map, not the Scraper code.
*   **Shadow-DOM Support:** Built-in logic for handling "Closed Platform" UI tricks (e.g., Threads' nested divs or X's dynamic GraphQL query IDs).
*   **WAF Bypass Hooks:** Adapters can request a "Scraper Pivot"—telling the engine: *"This payload is a bot-challenge; re-run with a High-Compute Scraper (Puppeteer) instead of a Raw-Request Scraper."*

### Error Handling Hierarchy

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      ERROR HANDLING FLOW                                     │
├─────────────────────────────────────────────────────────────────────────────┤
│  LAYER          │ RESPONSIBILITY                                             │
├─────────────────┼───────────────────────────────────────────────────────────┤
│  Scraper        │ Detect HTTP errors (429, 403, 500)                        │
│                 │ Return GhostError with category                            │
├─────────────────┼───────────────────────────────────────────────────────────┤
│  Adapter        │ Detect parsing failures (DOM changed, selector mismatch)  │
│                 │ Request Scraper Pivot if WAF detected                      │
├─────────────────┼───────────────────────────────────────────────────────────┤
│  Core           │ Classify error type: Proxy, Account, or Scraper issue     │
│                 │ Trigger fallback or circuit breaker                        │
│                 │ Run automated diagnostics on 100% failure                  │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 13. Context-Aware Multi-Tenancy (BYO-Injection)

Statelessness is a feature, not a bug. `ghost-api` doesn't "manage" accounts; it orchestrates the credentials you provide per-call.

*   **The `GhostContext` Object:** Every programmatic call requires a context.
    ```rust
    let ctx = GhostContext::builder()
        .tenant_id("user_789")        // For internal rate-limiting
        .proxy(Proxy::from_env())     // Auto-rotation or fixed
        .session(Session::Cookies("auth_token=...; ct0=..."))
        .build();
    ```
*   **Zero Global State:** Query X using 100 different accounts across 100 different proxies in parallel from the same `Ghost` instance.
*   **Automatic Refresh:** If a scraper supports session refreshing, the engine emits a `GhostEvent::SessionUpdated` which you can hook into to save the new state back to your DB.

---

## 14. Health Scoring & Circuit Breaking

The engine maintains a real-time "Health Matrix" to prevent sending requests into a black hole.

### Scoring Algorithm

$$Health = (S_{rate} \times 0.6) + (L_{norm} \times 0.4)$$

| Variable | Description | Range |
|----------|-------------|-------|
| $S_{rate}$ | Success rate over last N requests | 0.0 - 1.0 |
| $L_{norm}$ | Normalized latency (fast = 1.0, slow = 0.0) | 0.0 - 1.0 |

### Calculation Example

```
Worker: py-playwright
├── Success rate: 85% (S_rate = 0.85)
├── Avg latency: 500ms, normalized against 2000ms max (L_norm = 0.75)
└── Health = (0.85 × 0.6) + (0.75 × 0.4) = 0.51 + 0.30 = 0.81

Result: Healthy (threshold = 0.7) ✓
```

### Threshold Classification

| Health Score | Status | Behavior |
|--------------|--------|----------|
| > 0.8 | **Healthy** | Preferred for routing |
| 0.5 - 0.8 | **Degraded** | Used when healthy workers unavailable |
| < 0.5 | **Unhealthy** | Only used as last resort |
| 0.0 | **Dead** | Circuit breaker engaged, cooling down |

### Circuit Breaker Behavior

*   **The Greylist:** Scrapers returning 429s (Rate Limited) are moved to a `CoolDown` state. The engine exponentially backs off before probing them with a "canary" request.
*   **Sticky Health:** Health is tracked per-scraper **per-endpoint**. A scraper might be "Healthy" for fetching posts but "Dead" for searching users.

---

## 15. Fallback Hierarchy (The Tiered Cascade)

Routing is determined by the `Strategy` enum passed at call-time.

### Decision Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      ROUTING DECISION TREE                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Request Received                                                            │
│       │                                                                      │
│       ▼                                                                      │
│  ┌─────────────────┐                                                        │
│  │ TIER 1: Fast    │  Health > 0.8, lightweight scrapers                    │
│  │ Scrapers        │  (HTTP clients, minimal browser)                       │
│  └────────┬────────┘                                                        │
│           │ Failed?                                                          │
│           ▼                                                                  │
│  ┌─────────────────┐                                                        │
│  │ PIVOT or        │  Same tier, different scraper?                         │
│  │ ESCALATE?       │  WAF detected → Pivot to browser-based                 │
│  └────────┬────────┘                                                        │
│           │ Escalate                                                         │
│           ▼                                                                  │
│  ┌─────────────────┐                                                        │
│  │ TIER 2: Heavy   │  Headless browsers (Playwright, Puppeteer)             │
│  │ Scrapers        │  Higher latency/cost, better stealth                   │
│  └────────┬────────┘                                                        │
│           │ Failed?                                                          │
│           ▼                                                                  │
│  ┌─────────────────┐                                                        │
│  │ TIER 3: Official│  Only if Strategy::OfficialFallback enabled            │
│  │ API             │  Uses paid API keys                                    │
│  └────────┬────────┘                                                        │
│           │ Failed?                                                          │
│           ▼                                                                  │
│  ┌─────────────────┐                                                        │
│  │ MeshExhausted   │  Returns detailed trace of all failures                │
│  └─────────────────┘                                                        │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Pivot vs Fallback

| Action | When | Effect |
|--------|------|--------|
| **Pivot** | WAF/challenge detected within tier | Switch to different scraper in same tier |
| **Fallback** | All scrapers in tier failed | Escalate to next tier |

---

## 16. Observability & Performance Tuning

`ghost-api` is built on `tokio` for high-concurrency and `tracing` for deep visibility.

*   **Structured Tracing:** Every request gets a `trace_id`. Follow a call as it moves from the Rust core into a Python subprocess and back.
*   **Metrics Export:** Native Prometheus integration:
    *   `ghost_scraper_success_total{worker="py-playwright", endpoint="x_get_user"}`
    *   `ghost_fallback_trigger_total{reason="rate_limit"}`
*   **Memory Safety:** 100% `#![forbid(unsafe_code)]` in `ghost-core`. Memory leaks in a Python/TS scraper are isolated to their respective runtimes and won't crash the Rust engine.

---

## 17. Proxy Orchestration (Protocol-Agnostic Tunneling)

`ghost-api` doesn't just "pass a string" to the scraper. It manages the proxy lifecycle to avoid fingerprinting and IP-burn.

*   **Proxy-Per-Call:** Inject `SOCKS5`, `HTTP`, or `Residential` gateways directly into the `GhostContext`.
*   **Sticky Sessions:** Optional `session_id` mapping to ensure a specific account always exits through the same IP/Gateway to minimize "suspicious login" flags.
*   **Auto-Blacklisting:** If a proxy returns a 403 or a "Robot detected" page consistently, the Health Engine marks that specific Proxy+Scraper pair as "Unhealthy" without killing the scraper itself.
*   **Internal Tunneling:** The Rust core can act as a MITM proxy for child scrapers, handling TLS termination and header injection centrally to ensure uniformity across Python/TS workers.

---

## 18. The Rust SDK: Direct Type-Safe Access

While the Swagger UI exists for debugging, the "True North" of `ghost-api` is its programmatic Rust interface. No JSON-parsing overhead—just types.

*   **Zero-Cost Abstractions:** Scraper outputs are deserialized directly into `ghost-schema` structs.
*   **Async-First:** Built on `tokio`. Use `select!` to race multiple scrapers or `join_all` for massive data ingestion.
*   **The `Ghost` Trait:**
    ```rust
    // Direct programmatic power
    let x_client = ghost.x(); 
    let results = x_client.search("Hacker News", &ctx).await?; 
    ```
*   **Middleware Hooks:** Attach `on_request` or `on_response` listeners to the engine to log raw HTML for local debugging or to implement custom caching layers.

---

## 19. Unified Platform AST (Abstract Syntax Tree)

We treat "Closed Platforms" as data sources for a single, unified Social AST.

### Type Mapping

| Feature | X (Closed) | Threads (Closed) | Ghost-API (Unified) |
| :--- | :--- | :--- | :--- |
| **Object** | `Tweet` | `Post/Thread` | `GhostPost` |
| **Identity** | `RestID` | `PK / ID` | `UID` |
| **Auth** | `Guest Token / CT0` | `LSD / Token` | `GhostSession` |
| **Adapter** | `x-adapter` | `threads-adapter` | **Pure Rust Struct** |

### Capabilities

| Capability | X | Threads | Description |
|------------|---|---------|-------------|
| `X_READ` | ✓ | — | Read posts/tweets |
| `X_SEARCH` | ✓ | — | Search content |
| `THREADS_READ` | — | ✓ | Read threads |
| `THREADS_SEARCH` | — | ✓ | Search threads |

### Features

*   **Platform Leakage Prevention:** The Adapter strips "tracking pixels" and platform-specific telemetry from the data before it ever hits your programmatic code.
*   **AST Versioning:** If Threads pushes a breaking UI change, the Adapter layer catches it. Your Rust code continues to receive a stable `GhostPost` struct.
*   **Feature Detection:** Query the engine at runtime: `ghost.capabilities_for("threads")`. It returns a list of active scrapers and the specific endpoints they currently support based on their latest health checks.

---

## 20. The "Platform Shield" (Handling Hostility)

X and Threads use advanced fingerprinting to detect automated access. `ghost-api` Adapters include sophisticated countermeasures.

### Anti-Detection Features

*   **JA3/H2 Fingerprint Injection:** Ensures the Scraper's TLS handshake matches the expected platform browser, preventing detection via TLS fingerprinting.
*   **Header Entropy:** Adapters generate platform-correct headers (`x-fb-lsd`, `x-twitter-active-user`) dynamically per request based on the injected `GhostContext`, avoiding static header patterns that trigger bot detection.
*   **Rate-Limit Jitter:** The Adapter calculates "Platform-Specific Fatigue"—forcing the engine to wait between calls based on known (but undocumented) X/Threads limits to avoid triggering rate-limiting heuristics.
*   **Challenge Detection:** When a WAF challenge (e.g., Cloudflare, hCaptcha) is detected, the Adapter can automatically request a "Scraper Pivot" to a browser-based worker capable of solving the challenge.

### Configuration

```toml
[shield.x]
fingerprint = "chrome_120"           # Options: chrome_120, firefox_121, safari_17
header_profile = "desktop_windows"   # Options: desktop_windows, desktop_macos, mobile_ios
jitter_range_ms = [500, 3000]        # Min/max random delay between requests
respect_retry_after = true           # Parse and honor Retry-After headers

[shield.threads]
fingerprint = "chrome_120"
header_profile = "desktop_macos"
jitter_range_ms = [800, 4000]
```

---

## 21. Sandboxing & Runtime Isolation

Running Python and Node.js code inside a Rust process is risky. `ghost-api` focuses on "Crash-Only" resilience.

*   **FFI Safety:** All calls to `PyO3` or `NAPI` are wrapped in `catch_unwind` and specialized error boundaries.
*   **Resource Limits:** Cap the memory and CPU usage of the "heavy" scrapers (like Playwright-based ones) to prevent them from starving the Rust async executor.
*   **Process Restart:** If a scraper's underlying bridge (e.g., the Python interpreter) hits a fatal state, the `ghost-bridge` automatically re-initializes it while the Health Engine routes traffic to other workers.

---

## 22. Production Topologies & Deployment

`ghost-api` is designed to scale from a single binary to a distributed mesh.

| Mode | Description | Use Case |
|------|-------------|----------|
| **Embedded** | Everything in one process | CLI tools or low-to-medium volume bots |
| **Sidecar** | Rust core as sidecar, UDS communication | Zero network latency, main app integration |
| **Distributed** | Scrapers on separate worker nodes via gRPC | Spot instances, horizontal scaling |
| **Crate-Only** | Import `ghost-core` and `ghost-bridge` directly | Maximum performance, minimum footprint |

---

## 23. Scraper Pool Auto-Scaling

Manual capacity planning for scraper workers is error-prone. `ghost-api` supports automatic scaling based on queue depth and health metrics.

### Configuration

```toml
[autoscaling]
enabled = true
min_workers = 2
max_workers = 20

# Scale up when queue depth > 50 for 30 seconds
scale_up_threshold = { queue_depth = 50, duration_secs = 30 }

# Scale down when utilization < 30% for 5 minutes
scale_down_threshold = { utilization = 0.3, duration_secs = 300 }

# Prefer spot instances for worker nodes (cost optimization)
prefer_spot = true
spot_fallback_on_demand = true
```

### Metrics Tracked

| Metric | Description | Use |
|--------|-------------|-----|
| Queue Depth | Pending requests per scraper type | Scale up trigger |
| Average Latency | Request processing time | Health indicator |
| Health Score Trend | Worker health over time | Predictive scaling |
| Cost per Success | Estimated cost per successful request | Cost optimization |

### Programmatic Control

```rust
// Monitor autoscaling events
while let Some(event) = ghost.autoscaling_events().recv().await {
    match event {
        AutoscaleEvent::ScalingUp { from, to, reason } => {
            println!("Scaling: {} -> {} workers ({})", from, to, reason);
        }
        AutoscaleEvent::ScalingDown { from, to, reason } => {
            println!("Scaling down: {} -> {} ({})", from, to, reason);
        }
        AutoscaleEvent::SpotInterrupted { worker_id } => {
            // Gracefully migrate work before spot termination
            ghost.migrate_worker(worker_id).await?;
        }
    }
}
```

---

## 24. Cost Attribution & Budget Controls

Multi-tenant systems make it hard to track who's burning resources. `ghost-api` provides per-tenant cost tracking with budget limits.

### Per-Tenant Budgets

```rust
let ctx = GhostContext::builder()
    .tenant_id("client_acme")
    .budget(BudgetLimits {
        max_requests_per_hour: 1000,
        max_cost_per_day: 50.0,  // USD
        alert_at_percent: 80,    // Alert at 80% usage
    })
    .build();
```

### Cost Factors

| Factor | Description | Calculation |
|--------|-------------|-------------|
| Official API | Direct platform API calls | Per-request pricing |
| Proxy Bandwidth | Data transfer through proxies | Estimated per GB |
| Compute Time | Scraper execution time | Weighted by scraper type |

### Cost Reporting

```bash
$ ghost-api costs report --tenant client_acme --period today

Tenant: client_acme
Period: 2024-01-15

Requests: 4,521 (Budget: 10,000)
Estimated Cost: $12.34 (Budget: $50.00)

Breakdown:
├── Official API: $8.00 (12 calls, fallback tier)
├── Proxy Bandwidth: $3.12 (2.1 GB)
└── Compute: $1.22 (1,240 scraper-minutes)

Alerts: None
```

### Budget Event Handling

```rust
// Subscribe to budget events
while let Some(event) = ghost.budget_events().recv().await {
    match event {
        BudgetEvent::ApproachingLimit { tenant_id, usage_percent } => {
            notify_tenant(tenant_id, format!("Budget at {}%", usage_percent));
        }
        BudgetEvent::LimitExceeded { tenant_id, limit_type } => {
            // Requests will be rejected until budget resets
            log::warn!("Tenant {} exceeded {} limit", tenant_id, limit_type);
        }
        BudgetEvent::LimitReset { tenant_id } => {
            notify_tenant(tenant_id, "Budget reset".to_string());
        }
    }
}
```

---

## 25. Session Health Monitoring

Accounts get banned silently—you often only discover it on the next failed request. `ghost-api` provides background health checks for injected sessions.

### Configuration

```rust
let ctx = GhostContext::builder()
    .session("auth_token=...")
    .session_health_check(HealthCheckConfig {
        interval: Duration::from_secs(300),  // Check every 5 min
        endpoint: "get_own_profile",         // Lightweight check
        on_failure: SessionAction::Emit,     // Emit event on failure
    })
    .build();
```

### Health Check Events

```rust
// Subscribe to session health events
while let Some(event) = ghost.session_events().recv().await {
    match event {
        SessionEvent::Unhealthy { session_id, reason } => {
            // reason: "suspended", "rate_limited", "cookie_expired", "locked"
            match reason {
                SessionUnhealthyReason::Suspended => {
                    // Remove from rotation permanently
                    remove_session_from_pool(session_id);
                }
                SessionUnhealthyReason::RateLimited { retry_after } => {
                    // Park until cooldown expires
                    park_session(session_id, retry_after);
                }
                SessionUnhealthyReason::CookieExpired => {
                    // Request fresh credentials from tenant
                    request_session_refresh(session_id);
                }
                _ => {}
            }
        }
        SessionEvent::Recovered { session_id } => {
            // Session back to healthy after cooldown
            restore_session_to_pool(session_id);
        }
        SessionEvent::Warning { session_id, message } => {
            // Early warning signs (e.g., elevated challenge rate)
            log::warn!("Session {}: {}", session_id, message);
        }
    }
}
```

### Health Status

| Status | Description | Action |
|--------|-------------|--------|
| `Healthy` | Passing all checks | Normal operation |
| `Degraded` | Intermittent failures | Reduced priority |
| `RateLimited` | Temporarily blocked | Park until `retry_after` |
| `Suspended` | Account suspended | Remove from pool |
| `Unknown` | Not yet checked | Pending first check |

---

## 26. Credential Vault Integration

Storing credentials in environment variables or config files is insecure. `ghost-api` provides native integration with secret managers.

### Supported Providers

| Provider | Feature Flag | Description |
|----------|--------------|-------------|
| AWS Secrets Manager | `vault-aws` | AWS native secrets |
| HashiCorp Vault | `vault-hashicorp` | Enterprise secret management |
| GCP Secret Manager | `vault-gcp` | Google Cloud secrets |
| Azure Key Vault | `vault-azure` | Microsoft Azure secrets |

### Configuration

```toml
[vault]
provider = "aws_secrets_manager"
region = "us-east-1"

# Cache settings
cache_ttl_secs = 300          # Cache secrets for 5 min max
cache_encrypted = true        # Encrypt cached secrets

# Audit logging
audit_enabled = true
audit_log = "/var/log/ghost-api/vault-audit.log"

# Reference secrets by path
[scrapers.official]
api_key = "vault:aws:social-apis/twitter/api-key"

[proxy_pools.residential]
credentials = "vault:aws:proxy-pools/residential-auth"

[tenants.client_acme]
session = "vault:aws:tenants/client-acme/x-session"
```

### Runtime Behavior

```rust
// Secrets are fetched on-demand, never cached longer than necessary
let ctx = GhostContext::builder()
    .tenant_id("client_acme")
    .session_from_vault("vault:aws:tenants/client-acme/x-session")
    .build();

// Automatic rotation detection
ghost.vault().on_rotation(|secret_id, new_value| {
    // Secret was rotated in the vault
    // Update any active sessions using this secret
    update_active_sessions(secret_id, new_value);
});
```

### Audit Trail

All credential access is logged:

```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "event": "secret_accessed",
  "secret_id": "vault:aws:tenants/client-acme/x-session",
  "tenant_id": "client_acme",
  "request_id": "req_abc123",
  "source_ip": "10.0.1.50",
  "user_agent": "ghost-api/0.1.0"
}
```

---

## 27. Scaling the Scraper Pool (Developer SDK)

Adding a new worker to `ghost-api` shouldn't require a PR to the core. We use a plugin-style registry.

*   **The Scraper Template:** Use provided boilerplate for Python (PyO3) or Node.js (NAPI).
*   **The GhostWorker Trait:** Your worker accepts a `RawContext` (JSON/MsgPack) and returns a `PayloadBlob`.
*   **Discovery:** `ghost-api` auto-scans the `/scrapers` directory. If it finds a binary or a library matching the naming convention, it registers its capabilities at startup.

---

## 28. Simulation & Mocking (The "Shadow Graph")

How do you test your code without burning your X/Threads accounts?

*   **Mock Scrapers:** `ghost-api` includes a `mock-worker` that returns static or randomized "Twitter-like" data.
*   **VCR Mode:** Record real platform responses and replay them during CI/CD. Note: VCR captures snapshots—live testing is still required for adapter validation.
*   **Chaos Testing:** Inject "429 Too Many Requests" or "Account Suspended" responses into the stream to verify your Rust error-handling and fallback logic.

---

## 29. High-Throughput Batching & Backpressure

Scraping is slow; Rust is fast. We bridge the gap with `tokio` mpsc channels.

*   **Concurrency Limits:** Set `max_concurrent_requests` per scraper. If a Python worker is pegged at 5 browsers, the engine automatically queues or re-routes to a Go-based RE worker.
*   **Request Coalescing:** If 10 calls ask for the same Tweet ID simultaneously, `ghost-api` deduplicates them into a single scraper call to save proxy bandwidth and account health.
*   **Zero-Copy Deserialization:** We use `serde` with `Cow` (Clone-on-Write) types to minimize memory overhead when moving large profile JSONs from the Scraper Bridge to the HTTP output.

---

## 30. Troubleshooting & "Ban-Hammer" Diagnosis

When a scraper fails, you need to know *why* (IP ban, Account ban, or DOM change).

*   **The Trace Header:** Every response includes `X-Ghost-Provider`, `X-Ghost-Health-Delta`, and `X-Ghost-Scraper-Type`.
*   **Automated Diagnostics:** If a worker hits 100% failure, `ghost-api` runs a "Sanity Check" (e.g., fetching a public profile with no proxy) to determine if the issue is the Proxy, the Account, or the Scraper's code.
*   **Log Streaming:** View scraper `stderr` directly in your terminal/Loki logs—prefixed with the worker name for easy debugging.

---

## 31. Ethical Scraping & Rate-Limit Etiquette

Don't be the reason we can't have nice things.

*   **Jitter & Human Simulation:** The `GhostContext` can inject random delays between requests to mimic human browsing patterns.
*   **Respecting `Retry-After`:** The engine parses platform-specific headers and automatically parks the associated `GhostSession` until the lockout expires.
*   **Data Minimization:** Use the `fields` parameter to tell the **Adapter** to only extract what you need, reducing CPU load and scraper execution time.

---

## 32. Roadmap: The Multi-Platform Future

`ghost-api` is designed for anything with a "hostile" UI.

| Version | Milestone |
|---------|-----------|
| **v0.1** | Current release: X and Threads support with GhostWorker protocol, health scoring, and platform adapters. |
| **v0.2** | Support for Bluesky (ATProto) and Mastodon (as native tiers, no scraping needed). |
| **v0.3** | LinkedIn and Instagram adapters (High-friction targets). |
| **v0.4** | Distributed Health Sync via Redis (for multi-instance deployments). |
| **v0.5** | WASM-based scrapers for even tighter isolation. |

---

## 33. Community & Support

*   **GitHub Issues:** [github.com/your-org/ghost-api/issues](https://github.com/your-org/ghost-api/issues)
*   **Discussions:** [github.com/your-org/ghost-api/discussions](https://github.com/your-org/ghost-api/discussions)
*   **Contributing:** See `CONTRIBUTING.md` for coding standards (no `unwrap()`, all the types).

---

## 34. License & Contributor Agreement

`ghost-api` is MIT Licensed.

*   **Attribution:** If you use this in a commercial product, keep the headers.
*   **Warranties:** None. Scraping is a cat-and-mouse game. If X or Threads changes their entire architecture tomorrow, we "move fast and fix the adapters."
*   **Contributions:** PRs for new **Adapters** or **Scrapers** are highly encouraged. Check `CONTRIBUTING.md` for the coding standard.
