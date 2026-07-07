use crate::batch::BatchEngine;
use crate::book::{BookSnapshot, Order, Trade};
use crate::continuous::ContinuousEngine;
use crate::matching::MatchResult;
use adenora_common::types::*;

/// Dual-mode market engine — wraps both People (batch) and Bot (continuous)
/// order books for a single market.
///
/// Same underlying event, same oracle resolution, separate order books.
pub struct DualModeEngine {
    pub market_id: MarketId,
    pub people: BatchEngine,
    pub bot: ContinuousEngine,
}

impl DualModeEngine {
    pub fn new(market_id: MarketId, batch_interval_ms: u64) -> Self {
        Self {
            market_id,
            people: BatchEngine::new(batch_interval_ms),
            bot: ContinuousEngine::new(),
        }
    }

    /// Submit an order to the appropriate engine based on market mode.
    pub async fn submit(&self, order: Order) -> SubmitResult {
        match order.mode {
            MarketMode::People => {
                let id = self.people.submit(order).await;
                SubmitResult::Queued { order_id: id }
            }
            MarketMode::Unlimited => {
                let (id, result) = self.bot.submit(order).await;
                SubmitResult::Executed {
                    order_id: id,
                    trades: result.trades,
                    cancelled: result.cancelled,
                }
            }
        }
    }

    /// Cancel an order from either engine.
    pub async fn cancel(&self, order_id: OrderId, mode: MarketMode) -> Option<Order> {
        match mode {
            MarketMode::People => self.people.cancel(order_id).await,
            MarketMode::Unlimited => self.bot.cancel(order_id).await,
        }
    }

    /// Execute a batch cycle for the people-mode engine.
    pub async fn execute_people_batch(&self) -> (BatchId, MatchResult) {
        self.people.execute_batch().await
    }

    /// Rehydrate a resting people-mode order into the book (startup recovery).
    pub async fn rehydrate_people(&self, order: Order) {
        self.people.rehydrate(order).await;
    }

    /// Rehydrate a resting order into the engine matching its mode (startup).
    pub async fn rehydrate(&self, order: Order) {
        match order.mode {
            MarketMode::People => self.people.rehydrate(order).await,
            MarketMode::Unlimited => self.bot.rehydrate(order).await,
        }
    }

    /// Get order book snapshots for both modes.
    pub async fn snapshots(&self) -> DualSnapshot {
        DualSnapshot {
            market_id: self.market_id,
            people: self.people.snapshot().await,
            bot: self.bot.snapshot().await,
        }
    }
}

#[derive(Debug)]
pub enum SubmitResult {
    /// Order queued for next batch (people mode)
    Queued { order_id: OrderId },
    /// Order matched immediately (bot mode)
    Executed {
        order_id: OrderId,
        trades: Vec<Trade>,
        /// Orders the engine removed during matching (self-trade prevention or
        /// IOC/FOK that could not fill) — their reserved funds must be released.
        cancelled: Vec<OrderId>,
    },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DualSnapshot {
    pub market_id: MarketId,
    pub people: BookSnapshot,
    pub bot: BookSnapshot,
}
