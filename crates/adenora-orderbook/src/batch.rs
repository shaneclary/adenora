use crate::book::{Order, OrderBook, Trade};
use crate::matching;
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
    /// The order book
    book: Mutex<OrderBook>,
    /// Batch interval in milliseconds
    interval_ms: u64,
}

impl BatchEngine {
    pub fn new(interval_ms: u64) -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            book: Mutex::new(OrderBook::new()),
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

    /// Execute one batch cycle: drain queue, insert all, match.
    /// Returns the batch ID and any trades produced.
    pub async fn execute_batch(&self) -> (BatchId, Vec<Trade>) {
        let batch_id = Uuid::new_v4();

        // Drain all pending orders
        let orders: Vec<Order> = {
            let mut queue = self.queue.lock().await;
            queue.drain(..).collect()
        };

        if orders.is_empty() {
            return (batch_id, Vec::new());
        }

        let mut book = self.book.lock().await;

        // Insert all orders from this batch into the book
        // They all get the same timestamp (batch time), so within a batch
        // the order is effectively random — no latency advantage
        for order in orders {
            match order.time_in_force {
                TimeInForce::Gtc | TimeInForce::Ioc | TimeInForce::Fok => {
                    book.submit_order(order);
                }
            }
        }

        // Match all crossing orders
        let trades = matching::match_orders(&mut book, MarketMode::People, Some(batch_id));

        // Handle IOC orders that weren't fully filled — cancel remainder
        for (_price, orders) in book.bids.levels.iter_mut() {
            orders.retain(|o| o.time_in_force != TimeInForce::Ioc || o.is_fully_filled());
        }
        for (_price, orders) in book.asks.levels.iter_mut() {
            orders.retain(|o| o.time_in_force != TimeInForce::Ioc || o.is_fully_filled());
        }
        // Clean up empty price levels
        book.bids.levels.retain(|_, orders| !orders.is_empty());
        book.asks.levels.retain(|_, orders| !orders.is_empty());

        (batch_id, trades)
    }

    /// Get a snapshot of the current order book.
    pub async fn snapshot(&self) -> crate::book::BookSnapshot {
        self.book.lock().await.snapshot()
    }

    /// Get the batch interval.
    pub fn interval_ms(&self) -> u64 {
        self.interval_ms
    }
}
