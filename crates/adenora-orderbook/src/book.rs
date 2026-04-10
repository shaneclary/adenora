use adenora_common::types::*;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A single order in the book.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: OrderId,
    pub user_id: UserId,
    pub market_id: MarketId,
    pub side: Side,
    pub action: Action,
    pub price_cents: u32,
    pub quantity: u32,
    pub filled_quantity: u32,
    pub time_in_force: TimeInForce,
    pub status: OrderStatus,
    pub mode: MarketMode,
    pub bot_id: Option<BotId>,
    pub created_at: DateTime<Utc>,
}

impl Order {
    pub fn remaining(&self) -> u32 {
        self.quantity - self.filled_quantity
    }

    pub fn is_fully_filled(&self) -> bool {
        self.filled_quantity >= self.quantity
    }

    pub fn price_dollars(&self) -> Decimal {
        Decimal::new(self.price_cents as i64, 2)
    }
}

/// A matched trade between two orders.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: TradeId,
    pub market_id: MarketId,
    pub batch_id: Option<BatchId>,
    pub buyer_order_id: OrderId,
    pub seller_order_id: OrderId,
    pub buyer_user_id: UserId,
    pub seller_user_id: UserId,
    pub side: Side,
    pub price_cents: u32,
    pub quantity: u32,
    pub buyer_fee: Decimal,
    pub seller_fee: Decimal,
    pub mode: MarketMode,
    pub executed_at: DateTime<Utc>,
}

/// Price level in the order book — aggregated quantity at a price.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceLevel {
    pub price_cents: u32,
    pub total_quantity: u32,
    pub order_count: u32,
}

/// One side of the order book (all bids or all asks).
#[derive(Debug, Clone, Default)]
pub struct BookSide {
    /// Price -> list of orders at that price (time-ordered)
    pub levels: BTreeMap<u32, Vec<Order>>,
}

impl BookSide {
    pub fn insert(&mut self, order: Order) {
        self.levels
            .entry(order.price_cents)
            .or_default()
            .push(order);
    }

    pub fn remove(&mut self, order_id: OrderId) -> Option<Order> {
        let mut found_price = None;
        let mut found_order = None;

        for (price, orders) in self.levels.iter_mut() {
            if let Some(pos) = orders.iter().position(|o| o.id == order_id) {
                found_order = Some(orders.remove(pos));
                if orders.is_empty() {
                    found_price = Some(*price);
                }
                break;
            }
        }

        if let Some(price) = found_price {
            self.levels.remove(&price);
        }

        found_order
    }

    pub fn best_price(&self) -> Option<u32> {
        self.levels
            .iter()
            .find(|(_, orders)| !orders.is_empty())
            .map(|(price, _)| *price)
    }

    pub fn depth(&self) -> Vec<PriceLevel> {
        self.levels
            .iter()
            .filter(|(_, orders)| !orders.is_empty())
            .map(|(price, orders)| PriceLevel {
                price_cents: *price,
                total_quantity: orders.iter().map(|o| o.remaining()).sum(),
                order_count: orders.len() as u32,
            })
            .collect()
    }

    pub fn total_quantity(&self) -> u32 {
        self.levels
            .values()
            .flat_map(|orders| orders.iter())
            .map(|o| o.remaining())
            .sum()
    }
}

/// Full order book for one side of a market (YES or NO) in one mode (People or Bot).
#[derive(Debug, Clone, Default)]
pub struct OrderBook {
    /// Buy orders — sorted ascending by price (best bid = highest)
    pub bids: BookSide,
    /// Sell orders — sorted ascending by price (best ask = lowest)
    pub asks: BookSide,
}

impl OrderBook {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn best_bid(&self) -> Option<u32> {
        // Highest bid price
        self.bids.levels.keys().rev().next().copied()
    }

    pub fn best_ask(&self) -> Option<u32> {
        // Lowest ask price
        self.asks.levels.keys().next().copied()
    }

    pub fn spread(&self) -> Option<i32> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some(ask as i32 - bid as i32),
            _ => None,
        }
    }

    pub fn submit_order(&mut self, order: Order) {
        match order.action {
            Action::Buy => self.bids.insert(order),
            Action::Sell => self.asks.insert(order),
        }
    }

    pub fn cancel_order(&mut self, order_id: OrderId) -> Option<Order> {
        self.bids
            .remove(order_id)
            .or_else(|| self.asks.remove(order_id))
    }

    pub fn snapshot(&self) -> BookSnapshot {
        BookSnapshot {
            bids: self.bids.depth(),
            asks: self.asks.depth(),
            best_bid: self.best_bid(),
            best_ask: self.best_ask(),
            spread: self.spread(),
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookSnapshot {
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
    pub best_bid: Option<u32>,
    pub best_ask: Option<u32>,
    pub spread: Option<i32>,
    pub timestamp: DateTime<Utc>,
}
