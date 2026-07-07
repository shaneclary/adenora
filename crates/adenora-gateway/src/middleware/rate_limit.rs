use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

use axum::{
    extract::{ConnectInfo, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::state::AppState;

/// Axum middleware: enforce per-IP rate limit.
///
/// Note: the `X-Bot-Key` header used to bypass this limiter on mere presence,
/// but there is no stored bot key to validate it against, so any client could
/// spoof it for unlimited throughput. All requests are now rate-limited; bots
/// authenticate via JWT like any other account.
pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    let ip = client_ip(&state, &req);

    match state.rate_limiter.check(&ip).await {
        Ok(_remaining) => next.run(req).await,
        Err(()) => (
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded. Please slow down.",
        )
            .into_response(),
    }
}

/// Determine the rate-limit key for a request.
///
/// Uses the real socket peer address. `X-Forwarded-For` is only trusted when the
/// direct peer is a configured trusted proxy — otherwise a client could set the
/// header to a random value per request and get a fresh bucket each time.
fn client_ip(state: &AppState, req: &Request) -> String {
    let peer_ip = req
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ci| ci.0.ip());

    match peer_ip {
        Some(ip) if state.config.server.trusted_proxies.contains(&ip) => req
            .headers()
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.split(',').next())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| ip.to_string()),
        Some(ip) => ip.to_string(),
        // No connection info (e.g. in tests) — fall back to a single shared key
        // rather than trusting a spoofable header.
        None => "unknown".to_string(),
    }
}

/// Token bucket rate limiter.
/// People mode: generous limits (60 req/min per IP).
/// Bot mode: unlimited (no rate limit applied).
#[derive(Clone)]
pub struct RateLimiter {
    buckets: Arc<Mutex<HashMap<String, Bucket>>>,
    max_tokens: u32,
    refill_per_sec: f64,
}

struct Bucket {
    tokens: f64,
    last_refill: Instant,
}

impl RateLimiter {
    pub fn new(max_tokens: u32, refill_per_sec: f64) -> Self {
        Self {
            buckets: Arc::new(Mutex::new(HashMap::new())),
            max_tokens,
            refill_per_sec,
        }
    }

    /// Default: 60 requests per minute, refills at 1/sec.
    pub fn default_people() -> Self {
        Self::new(60, 1.0)
    }

    /// Bot mode: effectively unlimited (10000 req/min).
    pub fn bot_unlimited() -> Self {
        Self::new(10_000, 167.0)
    }

    /// Check if a request is allowed. Returns remaining tokens.
    pub async fn check(&self, key: &str) -> Result<u32, ()> {
        let mut buckets = self.buckets.lock().await;
        let now = Instant::now();

        let bucket = buckets.entry(key.to_string()).or_insert(Bucket {
            tokens: self.max_tokens as f64,
            last_refill: now,
        });

        let elapsed = now.duration_since(bucket.last_refill).as_secs_f64();
        bucket.tokens = (bucket.tokens + elapsed * self.refill_per_sec)
            .min(self.max_tokens as f64);
        bucket.last_refill = now;

        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            Ok(bucket.tokens as u32)
        } else {
            Err(())
        }
    }

    /// Periodic cleanup of stale buckets.
    pub async fn cleanup(&self) {
        let mut buckets = self.buckets.lock().await;
        let now = Instant::now();
        buckets.retain(|_, bucket| {
            now.duration_since(bucket.last_refill).as_secs() < 300
        });
    }
}
