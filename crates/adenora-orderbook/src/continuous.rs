use crate::book::{Order, OrderBook, Trade};
use crate::matching;
use adenora_common::types::*;
use tokio::sync::Mutex;

/// Bot Battle Mode: Continuous Matching Engine
///
/// Orders are matched immediately on submission — no batching delay.
/// No rate limits. Pure speed competition.
/// Same fee structure as people mode.
pub struct ContinuousEngine {
    book: Mutex<OrderBook>,
}

impl ContinuousEngine {
    pub fn new() -> Self {
        Self {
            book: Mutex::new(OrderBook::new()),
        }
    }

    /// Submit an order and immediately attempt to match.
    /// Returns the order ID and any trades produced.
    pub async fn submit(&self, order: Order) -> (OrderId, Vec<Trade>) {
        let id = order.id;
        let tif = order.time_in_force;

        let mut book = self.book.lock().await;

        if tif == TimeInForce::Fok && projected_fill(&book, &order) < order.quantity {
            return (id, Vec::new());
        }

        book.submit_order(order);

        let trades = matching::match_orders(&mut book, MarketMode::Unlimited, None);

        // Handle IOC: cancel unfilled remainder
        if tif == TimeInForce::Ioc {
            // Check if our order is still in the book (partially or unfilled)
            let _ = book.cancel_order(id);
        }

        (id, trades)
    }

    /// Cancel a resting order.
    pub async fn cancel(&self, order_id: OrderId) -> Option<Order> {
        let mut book = self.book.lock().await;
        let mut order = book.cancel_order(order_id)?;
        order.status = OrderStatus::Cancelled;
        Some(order)
    }

    /// Get a snapshot of the current order book.
    pub async fn snapshot(&self) -> crate::book::BookSnapshot {
        self.book.lock().await.snapshot()
    }
}

fn projected_fill(book: &OrderBook, order: &Order) -> u32 {
    let mut projected = book.clone();
    projected.submit_order(order.clone());

    matching::match_orders(&mut projected, MarketMode::Unlimited, None)
        .iter()
        .filter(|trade| trade.buyer_order_id == order.id || trade.seller_order_id == order.id)
        .map(|trade| trade.quantity)
        .sum()
}
