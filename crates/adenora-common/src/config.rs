use std::env;
use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct AdenoraConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub trading: TradingConfig,
    pub lottery: LotteryConfig,
    pub bot_arena: BotArenaConfig,
    pub kyc: KycConfig,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub bind_addr: SocketAddr,
    pub cors_origins: Vec<String>,
    /// IPs of trusted reverse proxies (e.g. the front-end load balancer).
    /// `X-Forwarded-For` is only honored when the direct peer is one of these;
    /// otherwise the client socket address is used for rate limiting so a client
    /// cannot spoof the header to escape limits.
    pub trusted_proxies: Vec<std::net::IpAddr>,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expiry_hours: u64,
    pub refresh_expiry_days: u64,
    pub min_age: u8,
}

#[derive(Debug, Clone)]
pub struct TradingConfig {
    pub batch_interval_ms: u64,
    pub max_contracts_per_order: u32,
    pub min_price_cents: u32,
    pub max_price_cents: u32,
    pub dispute_window_hours: u64,
    pub oracle_min_sources: u32,
}

#[derive(Debug, Clone)]
pub struct LotteryConfig {
    pub default_winner_pct: u32,
    pub max_ticket_price_cents: u32,
    pub free_entry_after_losses: u32,
}

#[derive(Debug, Clone)]
pub struct BotArenaConfig {
    pub enabled: bool,
    pub max_bots_per_user: u32,
    pub bot_name_max_length: usize,
}

#[derive(Debug, Clone)]
pub struct KycConfig {
    pub webhook_secret: String,
    pub provider: String,
}

impl AdenoraConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        Self {
            server: ServerConfig {
                bind_addr: env::var("ADENORA_BIND")
                    .unwrap_or_else(|_| "0.0.0.0:8080".into())
                    .parse()
                    .expect("invalid ADENORA_BIND"),
                cors_origins: env::var("ADENORA_CORS_ORIGINS")
                    .unwrap_or_else(|_| "*".into())
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect(),
                trusted_proxies: env::var("ADENORA_TRUSTED_PROXIES")
                    .unwrap_or_default()
                    .split(',')
                    .filter_map(|s| s.trim().parse().ok())
                    .collect(),
            },
            database: DatabaseConfig {
                url: env::var("DATABASE_URL")
                    .unwrap_or_else(|_| "postgres://adenora:adenora@localhost/adenora".into()),
                max_connections: env::var("DATABASE_MAX_CONN")
                    .unwrap_or_else(|_| "10".into())
                    .parse()
                    .unwrap_or(10),
            },
            auth: AuthConfig {
                jwt_secret: env::var("JWT_SECRET")
                    .unwrap_or_else(|_| "CHANGE_ME_IN_PRODUCTION".into()),
                jwt_expiry_hours: env::var("JWT_EXPIRY_HOURS")
                    .unwrap_or_else(|_| "24".into())
                    .parse()
                    .unwrap_or(24),
                refresh_expiry_days: env::var("REFRESH_EXPIRY_DAYS")
                    .unwrap_or_else(|_| "30".into())
                    .parse()
                    .unwrap_or(30),
                min_age: 21,
            },
            trading: TradingConfig {
                batch_interval_ms: env::var("BATCH_INTERVAL_MS")
                    .unwrap_or_else(|_| "500".into())
                    .parse()
                    .unwrap_or(500),
                max_contracts_per_order: env::var("MAX_CONTRACTS")
                    .unwrap_or_else(|_| "1000".into())
                    .parse()
                    .unwrap_or(1000),
                min_price_cents: 1,
                max_price_cents: 99,
                dispute_window_hours: env::var("DISPUTE_WINDOW_HOURS")
                    .unwrap_or_else(|_| "48".into())
                    .parse()
                    .unwrap_or(48),
                oracle_min_sources: env::var("ORACLE_MIN_SOURCES")
                    .unwrap_or_else(|_| "3".into())
                    .parse()
                    .unwrap_or(3),
            },
            lottery: LotteryConfig {
                default_winner_pct: 50,
                max_ticket_price_cents: 10000, // $100
                free_entry_after_losses: 5,
            },
            kyc: KycConfig {
                webhook_secret: env::var("KYC_WEBHOOK_SECRET")
                    .unwrap_or_else(|_| "CHANGE_ME_IN_PRODUCTION".into()),
                provider: env::var("KYC_PROVIDER")
                    .unwrap_or_else(|_| "veriff".into()),
            },
            bot_arena: BotArenaConfig {
                enabled: env::var("BOT_ARENA_ENABLED")
                    .unwrap_or_else(|_| "true".into())
                    .parse()
                    .unwrap_or(true),
                max_bots_per_user: 10,
                bot_name_max_length: 32,
            },
        }
    }

    /// The literal used as the fallback for any secret that is not configured.
    /// It is public knowledge (it ships in `.env.example`), so booting with it
    /// would let anyone forge JWTs or KYC webhooks.
    pub const INSECURE_DEFAULT_SECRET: &'static str = "CHANGE_ME_IN_PRODUCTION";

    /// Reject an insecure configuration before the server starts serving.
    ///
    /// Refuses to boot if any secret is still the well-known default, unless
    /// `ADENORA_ALLOW_INSECURE_DEFAULTS=1` is set (for local development/tests).
    pub fn validate(&self) -> Result<(), String> {
        if env::var("ADENORA_ALLOW_INSECURE_DEFAULTS").as_deref() == Ok("1") {
            return Ok(());
        }

        let mut problems = Vec::new();
        if self.auth.jwt_secret == Self::INSECURE_DEFAULT_SECRET {
            problems.push("JWT_SECRET is unset or the well-known default");
        }
        if self.kyc.webhook_secret == Self::INSECURE_DEFAULT_SECRET {
            problems.push("KYC_WEBHOOK_SECRET is unset or the well-known default");
        }

        if problems.is_empty() {
            Ok(())
        } else {
            Err(format!(
                "insecure configuration: {}. Set real secrets, or export \
                 ADENORA_ALLOW_INSECURE_DEFAULTS=1 for local development only.",
                problems.join("; ")
            ))
        }
    }
}
