use adenora_common::config::AdenoraConfig;
use adenora_orderbook::engine::DualModeEngine;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::middleware::rate_limit::RateLimiter;

pub type MarketEngines = Arc<RwLock<HashMap<Uuid, Arc<DualModeEngine>>>>;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AdenoraConfig>,
    pub db: sqlx::PgPool,
    pub engines: MarketEngines,
    pub rate_limiter: Arc<RateLimiter>,
}

impl AppState {
    pub async fn new(config: AdenoraConfig) -> Result<Self, sqlx::Error> {
        let db = sqlx::postgres::PgPoolOptions::new()
            .max_connections(config.database.max_connections)
            .connect(&config.database.url)
            .await?;

        Ok(Self {
            config: Arc::new(config),
            db,
            engines: Arc::new(RwLock::new(HashMap::new())),
            rate_limiter: Arc::new(RateLimiter::default_people()),
        })
    }

    /// Get or create a dual-mode engine for a market.
    pub async fn get_engine(&self, market_id: Uuid) -> Arc<DualModeEngine> {
        {
            let engines = self.engines.read().await;
            if let Some(engine) = engines.get(&market_id) {
                return engine.clone();
            }
        }

        let mut engines = self.engines.write().await;
        engines
            .entry(market_id)
            .or_insert_with(|| {
                Arc::new(DualModeEngine::new(
                    market_id,
                    self.config.trading.batch_interval_ms,
                ))
            })
            .clone()
    }
}
