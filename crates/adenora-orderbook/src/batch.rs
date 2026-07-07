use crate::book::{MarketBook, Order};
use crate::matching::{self, MatchResult};
use adenora_common::types::*;
use std::collections::VecDeque;
use tokio::sync::Mutex;
use uuid::Uuid;

/// People Mode: Batch Auction Engine
///
/// Orders are collected into a queue during a batch window (default 500ms).
/// At the end of each window, all orders in the batch are shuffled and
/// inserted into the book simultaneously, then matching occurs.
///
/// This eliminates latency advantage — a human submitting at 499ms and
/// a bot submitting at 1ms are treated identically within the same batch.
pub struct BatchEngine {
    /// Pending orders waiting for the next batch
    queue: Mutex<VecDeque<Order>>,
    /// The order books (one per outcome side)
    book: Mutex<MarketBook>,
    /// Batch interval in milliseconds
    interval_ms: u64,
}

impl BatchEngine {
    pub fn new(interval_ms: u64) -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            book: Mutex::new(MarketBook::new()),
            interval_ms,
        }
    }

    /// Submit an order into the current batch queue.
    /// Returns immediately — the order will be processed at the next batch.
    pub async fn submit(&self, order: Order) -> OrderId {
        let id = order.id;
        self.queue.lock().await.push_back(order);
        id
    }

    /// Insert a resting order directly into the book without going through a
    /// batch cycle. Used to rehydrate open orders into the engine on startup.
    pub async fn rehydrate(&self, order: Order) {
        self.book.lock().await.submit_order(order);
    }

    /// Cancel a pending or resting order.
    pub async fn cancel(&self, order_id: OrderId) -> Option<Order> {
        // Try to remove from pending queue first
        {
            let mut queue = self.queue.lock().await;
            if let Some(pos) = queue.iter().position(|o| o.id == order_id) {
                let mut order = queue.remove(pos).unwrap();
                order.status = OrderStatus::Cancelled;
                return Some(order);
            }
        }
        // Try to remove from resting book
        let mut book = self.book.lock().await;
        let mut order = book.cancel_order(order_id)?;
        order.status = OrderStatus::Cancelled;
        Some(order)
    }

    /// Execute one batch cycle: drain queue, insert all, match each side.
    /// Returns the batch ID and the match result (trades + cancelled ids).
    pub async fn execute_batch(&self) -> (BatchId, MatchResult) {
        let batch_id = Uuid::new_v4();

        // Drain all pending orders
        let orders: Vec<Order> = {
            let mut queue = self.queue.lock().await;
            queue.drain(..).collect()
        };

        if orders.is_empty() {
            return (batch_id, MatchResult::default());
        }

        let mut book = self.book.lock().await;

        // Insert all orders from this batch into their side's book.
        for order in orders {
            book.submit_order(order);
        }

        // Run a uniform-price call auction on each outcome side independently —
        // YES never crosses NO, and within a side all batch orders clear at one
        // price with no intra-batch time priority.
        let mut result = MatchResult::default();
        for side in [Side::Yes, Side::No] {
            let r = matching::match_auction(book.side_mut(side), MarketMode::People, Some(batch_id));
            result.trades.extend(r.trades);
            result.cancelled.extend(r.cancelled);
        }

        // Remove unfilled IOC/FOK orders (they must not rest) and report them so
        // their reserved funds get released. GTC orders remain resting.
        for side in [Side::Yes, Side::No] {
            let b = book.side_mut(side);
            for orders in b.bids.levels.values_mut().chain(b.asks.levels.values_mut()) {
                orders.retain(|o| {
                    let expires = matches!(o.time_in_force, TimeInForce::Ioc | TimeInForce::Fok)
                        && !o.is_fully_filled();
                    if expires {
                        result.cancelled.push(o.id);
                    }
                    !expires
                });
            }
            b.bids.levels.retain(|_, orders| !orders.is_empty());
            b.asks.levels.retain(|_, orders| !orders.is_empty());
        }

        (batch_id, result)
    }

    /// Get a snapshot of the current order book (YES side — the primary quote).
    pub async fn snapshot(&self) -> crate::book::BookSnapshot {
        self.book.lock().await.snapshot()
    }

    /// Snapshot of a specific outcome side.
    pub async fn snapshot_side(&self, side: Side) -> crate::book::BookSnapshot {
        self.book.lock().await.snapshot_side(side)
    }

    /// Get the batch interval.
    pub fn interval_ms(&self) -> u64 {
        self.interval_ms
    }
}
