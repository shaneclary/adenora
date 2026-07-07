use crate::book::{MarketBook, Order};
use crate::matching::{self, MatchResult};
use adenora_common::types::*;
use tokio::sync::Mutex;

/// Bot Battle Mode: Continuous Matching Engine
///
/// Orders are matched immediately on submission — no batching delay.
/// No rate limits. Pure speed competition.
/// Same fee structure as people mode.
pub struct ContinuousEngine {
    book: Mutex<MarketBook>,
}

impl ContinuousEngine {
    pub fn new() -> Self {
        Self {
            book: Mutex::new(MarketBook::new()),
        }
    }

    /// Submit an order and immediately attempt to match.
    /// Returns the order ID and the match result (trades + cancelled ids).
    pub async fn submit(&self, order: Order) -> (OrderId, MatchResult) {
        let id = order.id;
        let tif = order.time_in_force;
        let side = order.side;

        let mut book = self.book.lock().await;

        // Fill-or-kill: reject entirely (reporting the id so funds are released)
        // if it cannot fully fill against the current book.
        if tif == TimeInForce::Fok && projected_fill(&book, &order) < order.quantity {
            return (
                id,
                MatchResult {
                    trades: Vec::new(),
                    cancelled: vec![id],
                },
            );
        }

        book.submit_order(order);

        // Match only within the order's own side.
        let mut result = matching::match_orders(book.side_mut(side), MarketMode::Unlimited, None);

        // Immediate-or-cancel: remove any unfilled remainder still resting and
        // report it so the reserved funds for that remainder are released.
        if tif == TimeInForce::Ioc && book.cancel_order(id).is_some() {
            result.cancelled.push(id);
        }

        (id, result)
    }

    /// Insert a resting order directly into the book without matching. Used to
    /// rehydrate open orders into the engine on startup.
    pub async fn rehydrate(&self, order: Order) {
        self.book.lock().await.submit_order(order);
    }

    /// Cancel a resting order.
    pub async fn cancel(&self, order_id: OrderId) -> Option<Order> {
        let mut book = self.book.lock().await;
        let mut order = book.cancel_order(order_id)?;
        order.status = OrderStatus::Cancelled;
        Some(order)
    }

    /// Get a snapshot of the current order book (YES side — the primary quote).
    pub async fn snapshot(&self) -> crate::book::BookSnapshot {
        self.book.lock().await.snapshot()
    }

    /// Snapshot of a specific outcome side.
    pub async fn snapshot_side(&self, side: Side) -> crate::book::BookSnapshot {
        self.book.lock().await.snapshot_side(side)
    }
}

fn projected_fill(book: &MarketBook, order: &Order) -> u32 {
    let mut projected = book.clone();
    projected.submit_order(order.clone());

    matching::match_orders(projected.side_mut(order.side), MarketMode::Unlimited, None)
        .trades
        .iter()
        .filter(|trade| trade.buyer_order_id == order.id || trade.seller_order_id == order.id)
        .map(|trade| trade.quantity)
        .sum()
}
