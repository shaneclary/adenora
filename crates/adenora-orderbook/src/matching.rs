use crate::book::{OrderBook, Trade};
use adenora_common::fees;
use adenora_common::types::*;
use chrono::Utc;
use uuid::Uuid;

/// Result of a matching pass: the trades produced plus the ids of any orders the
/// engine removed internally (self-trade prevention). Callers must reconcile the
/// cancelled ids back to the database so reserved funds are released and the
/// order is not left `pending` forever.
#[derive(Debug, Default)]
pub struct MatchResult {
    pub trades: Vec<Trade>,
    pub cancelled: Vec<OrderId>,
}

/// Match crossing orders in the book and produce trades.
/// Used by both batch and continuous engines after orders are placed.
///
/// Both orders in any match are on the same side (YES or NO) because the book
/// this operates on holds a single outcome side — YES and NO never cross here.
pub fn match_orders(
    book: &mut OrderBook,
    mode: MarketMode,
    batch_id: Option<BatchId>,
) -> MatchResult {
    let mut trades = Vec::new();
    let mut cancelled = Vec::new();

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

        let bid_orders = book.bids.levels.get_mut(&best_bid_price).unwrap();
        let ask_orders = book.asks.levels.get_mut(&best_ask_price).unwrap();

        let bid = &mut bid_orders[0];
        let ask = &mut ask_orders[0];

        // Prevent self-trade
        if bid.user_id == ask.user_id {
            // Cancel the newer order and report it so its funds can be released.
            if bid.created_at > ask.created_at {
                bid.status = OrderStatus::Cancelled;
                cancelled.push(bid.id);
                bid_orders.remove(0);
            } else {
                ask.status = OrderStatus::Cancelled;
                cancelled.push(ask.id);
                ask_orders.remove(0);
            }
            continue;
        }

        // Execute at the resting order's price: the maker (older order) set the
        // price first, so an aggressor crossing it trades at the maker's price —
        // whether the maker is the bid or the ask.
        let exec_price = if ask.created_at <= bid.created_at {
            best_ask_price // ask is the resting maker
        } else {
            best_bid_price // bid is the resting maker
        };

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

    MatchResult { trades, cancelled }
}

/// Uniform-price call auction for the batch (people) engine.
///
/// Every order in the batch is treated as arriving simultaneously: a single
/// clearing price is computed for the whole book and *all* crossing orders
/// execute at that one price. There is no intra-batch time priority and no
/// maker/taker distinction — both sides pay the maker rate — so a submission at
/// 1ms and one at 499ms in the same window are treated identically, which is the
/// latency-neutral behavior the batch design promises.
///
/// The clearing price maximizes matched volume; ties are broken toward the
/// midpoint of the volume-maximizing range. Orders are still filled in
/// price-then-FIFO order for rationing when supply and demand are unequal.
pub fn match_auction(
    book: &mut OrderBook,
    mode: MarketMode,
    batch_id: Option<BatchId>,
) -> MatchResult {
    let mut trades = Vec::new();
    let mut cancelled = Vec::new();

    let Some(clearing_price) = clearing_price(book) else {
        return MatchResult { trades, cancelled };
    };

    loop {
        // Best eligible bid: highest price >= clearing, FIFO within the level.
        let best_bid_price = match book
            .bids
            .levels
            .iter()
            .rev()
            .find(|(p, o)| **p >= clearing_price && !o.is_empty())
        {
            Some((p, _)) => *p,
            None => break,
        };
        // Best eligible ask: lowest price <= clearing, FIFO within the level.
        let best_ask_price = match book
            .asks
            .levels
            .iter()
            .find(|(p, o)| **p <= clearing_price && !o.is_empty())
        {
            Some((p, _)) => *p,
            None => break,
        };

        let bid_orders = book.bids.levels.get_mut(&best_bid_price).unwrap();
        let ask_orders = book.asks.levels.get_mut(&best_ask_price).unwrap();
        let bid = &mut bid_orders[0];
        let ask = &mut ask_orders[0];

        // Self-trade prevention: drop the newer order and report it.
        if bid.user_id == ask.user_id {
            if bid.created_at > ask.created_at {
                cancelled.push(bid.id);
                bid_orders.remove(0);
            } else {
                cancelled.push(ask.id);
                ask_orders.remove(0);
            }
            continue;
        }

        let fill_qty = bid.remaining().min(ask.remaining());
        let price_dollars = rust_decimal::Decimal::new(clearing_price as i64, 2);
        // Equal treatment: both sides pay the maker rate in the auction.
        let fee = fees::calculate_maker_fee(fill_qty, price_dollars);

        let trade = Trade {
            id: Uuid::new_v4(),
            market_id: bid.market_id,
            batch_id,
            buyer_order_id: bid.id,
            seller_order_id: ask.id,
            buyer_user_id: bid.user_id,
            seller_user_id: ask.user_id,
            side: bid.side,
            price_cents: clearing_price,
            quantity: fill_qty,
            buyer_fee: fee,
            seller_fee: fee,
            mode,
            executed_at: Utc::now(),
        };

        bid.filled_quantity += fill_qty;
        ask.filled_quantity += fill_qty;
        bid.status = if bid.is_fully_filled() { OrderStatus::Filled } else { OrderStatus::PartialFill };
        ask.status = if ask.is_fully_filled() { OrderStatus::Filled } else { OrderStatus::PartialFill };
        trades.push(trade);

        if bid_orders[0].is_fully_filled() {
            bid_orders.remove(0);
        }
        if ask_orders[0].is_fully_filled() {
            ask_orders.remove(0);
        }
        if bid_orders.is_empty() {
            book.bids.levels.remove(&best_bid_price);
        }
        if ask_orders.is_empty() {
            book.asks.levels.remove(&best_ask_price);
        }
    }

    MatchResult { trades, cancelled }
}

/// Compute the uniform clearing price that maximizes matched volume, or `None`
/// if the book does not cross. Ties in volume are broken by the smallest
/// order imbalance, then by the midpoint of the tied price range.
fn clearing_price(book: &OrderBook) -> Option<u32> {
    // Candidate prices: every distinct price present in the book.
    let mut candidates: Vec<u32> = book
        .bids
        .levels
        .keys()
        .chain(book.asks.levels.keys())
        .copied()
        .collect();
    candidates.sort_unstable();
    candidates.dedup();

    let qty_at_or_above = |threshold: u32| -> u32 {
        book.bids
            .levels
            .iter()
            .filter(|(p, _)| **p >= threshold)
            .flat_map(|(_, o)| o.iter())
            .map(|o| o.remaining())
            .sum()
    };
    let qty_at_or_below = |threshold: u32| -> u32 {
        book.asks
            .levels
            .iter()
            .filter(|(p, _)| **p <= threshold)
            .flat_map(|(_, o)| o.iter())
            .map(|o| o.remaining())
            .sum()
    };

    // best = (matched_volume, -imbalance) maximized; collect the tied price range.
    let mut best_volume = 0u32;
    let mut best_imbalance = u32::MAX;
    let mut lo = None;
    let mut hi = None;

    for &p in &candidates {
        let demand = qty_at_or_above(p);
        let supply = qty_at_or_below(p);
        let matched = demand.min(supply);
        if matched == 0 {
            continue;
        }
        let imbalance = demand.abs_diff(supply);
        if matched > best_volume || (matched == best_volume && imbalance < best_imbalance) {
            best_volume = matched;
            best_imbalance = imbalance;
            lo = Some(p);
            hi = Some(p);
        } else if matched == best_volume && imbalance == best_imbalance {
            // Extend the tied range for the midpoint tie-break.
            lo = Some(lo.map_or(p, |l| l.min(p)));
            hi = Some(hi.map_or(p, |h| h.max(p)));
        }
    }

    match (lo, hi) {
        (Some(lo), Some(hi)) => Some((lo + hi) / 2),
        _ => None,
    }
}
