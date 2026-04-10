use crate::book::{OrderBook, Trade};
use adenora_common::fees;
use adenora_common::types::*;
use chrono::Utc;
use uuid::Uuid;

/// Match crossing orders in the book and produce trades.
/// Used by both batch and continuous engines after orders are placed.
pub fn match_orders(
    book: &mut OrderBook,
    mode: MarketMode,
    batch_id: Option<BatchId>,
) -> Vec<Trade> {
    let mut trades = Vec::new();

    loop {
        // Find best bid (highest buy price) and best ask (lowest sell price)
        let best_bid_price = match book.bids.levels.iter().rev().find(|(_, o)| !o.is_empty()) {
            Some((p, _)) => *p,
            None => break,
        };
        let best_ask_price = match book.asks.levels.iter().find(|(_, o)| !o.is_empty()) {
            Some((p, _)) => *p,
            None => break,
        };

        // No cross — done matching
        if best_bid_price < best_ask_price {
            break;
        }

        // Execute at the resting order's price (price-time priority)
        let exec_price = best_ask_price; // ask was resting first

        let bid_orders = book.bids.levels.get_mut(&best_bid_price).unwrap();
        let ask_orders = book.asks.levels.get_mut(&best_ask_price).unwrap();

        let bid = &mut bid_orders[0];
        let ask = &mut ask_orders[0];

        // Prevent self-trade
        if bid.user_id == ask.user_id {
            // Cancel the newer order
            if bid.created_at > ask.created_at {
                bid.status = OrderStatus::Cancelled;
                bid_orders.remove(0);
            } else {
                ask.status = OrderStatus::Cancelled;
                ask_orders.remove(0);
            }
            continue;
        }

        let fill_qty = bid.remaining().min(ask.remaining());
        let price_dollars = rust_decimal::Decimal::new(exec_price as i64, 2);

        // Calculate fees — taker pays taker rate, maker pays maker rate
        // The aggressive (newer) order is the taker
        let (buyer_fee, seller_fee) = if bid.created_at > ask.created_at {
            // Bid is taker (newer)
            (
                fees::calculate_taker_fee(fill_qty, price_dollars),
                fees::calculate_maker_fee(fill_qty, price_dollars),
            )
        } else {
            (
                fees::calculate_maker_fee(fill_qty, price_dollars),
                fees::calculate_taker_fee(fill_qty, price_dollars),
            )
        };

        let trade = Trade {
            id: Uuid::new_v4(),
            market_id: bid.market_id,
            batch_id,
            buyer_order_id: bid.id,
            seller_order_id: ask.id,
            buyer_user_id: bid.user_id,
            seller_user_id: ask.user_id,
            side: bid.side,
            price_cents: exec_price,
            quantity: fill_qty,
            buyer_fee,
            seller_fee,
            mode,
            executed_at: Utc::now(),
        };

        // Update fill quantities
        bid.filled_quantity += fill_qty;
        ask.filled_quantity += fill_qty;

        if bid.is_fully_filled() {
            bid.status = OrderStatus::Filled;
        } else {
            bid.status = OrderStatus::PartialFill;
        }
        if ask.is_fully_filled() {
            ask.status = OrderStatus::Filled;
        } else {
            ask.status = OrderStatus::PartialFill;
        }

        trades.push(trade);

        // Remove fully filled orders from the book
        if bid_orders[0].is_fully_filled() {
            bid_orders.remove(0);
        }
        if ask_orders[0].is_fully_filled() {
            ask_orders.remove(0);
        }

        // Clean up empty price levels
        if bid_orders.is_empty() {
            book.bids.levels.remove(&best_bid_price);
        }
        if ask_orders.is_empty() {
            book.asks.levels.remove(&best_ask_price);
        }
    }

    trades
}
