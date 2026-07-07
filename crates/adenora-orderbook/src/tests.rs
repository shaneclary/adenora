use crate::batch::BatchEngine;
use crate::book::{Order, OrderBook};
use crate::continuous::ContinuousEngine;
use crate::matching;
use adenora_common::types::*;
use chrono::Utc;
use uuid::Uuid;

fn make_order(side: Side, action: Action, price: u32, qty: u32, mode: MarketMode) -> Order {
    Order {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        market_id: Uuid::new_v4(),
        side,
        action,
        price_cents: price,
        quantity: qty,
        filled_quantity: 0,
        time_in_force: TimeInForce::Gtc,
        status: OrderStatus::Pending,
        mode,
        bot_id: None,
        created_at: Utc::now(),
    }
}

fn buy(price: u32, qty: u32) -> Order {
    make_order(Side::Yes, Action::Buy, price, qty, MarketMode::People)
}

fn sell(price: u32, qty: u32) -> Order {
    make_order(Side::Yes, Action::Sell, price, qty, MarketMode::People)
}

// === OrderBook basic tests ===

#[test]
fn test_empty_book() {
    let book = OrderBook::new();
    assert_eq!(book.best_bid(), None);
    assert_eq!(book.best_ask(), None);
    assert_eq!(book.spread(), None);
}

#[test]
fn test_submit_bid() {
    let mut book = OrderBook::new();
    book.submit_order(buy(50, 10));
    assert_eq!(book.best_bid(), Some(50));
    assert_eq!(book.best_ask(), None);
}

#[test]
fn test_submit_ask() {
    let mut book = OrderBook::new();
    book.submit_order(sell(60, 10));
    assert_eq!(book.best_ask(), Some(60));
    assert_eq!(book.best_bid(), None);
}

#[test]
fn test_spread_calculation() {
    let mut book = OrderBook::new();
    book.submit_order(buy(48, 10));
    book.submit_order(sell(52, 10));
    assert_eq!(book.spread(), Some(4));
}

#[test]
fn test_cancel_order() {
    let mut book = OrderBook::new();
    let order = buy(50, 10);
    let id = order.id;
    book.submit_order(order);
    assert_eq!(book.best_bid(), Some(50));

    let cancelled = book.cancel_order(id);
    assert!(cancelled.is_some());
    assert_eq!(book.best_bid(), None);
}

#[test]
fn test_cancel_nonexistent() {
    let mut book = OrderBook::new();
    assert!(book.cancel_order(Uuid::new_v4()).is_none());
}

#[test]
fn test_best_bid_is_highest() {
    let mut book = OrderBook::new();
    book.submit_order(buy(40, 5));
    book.submit_order(buy(60, 5));
    book.submit_order(buy(50, 5));
    assert_eq!(book.best_bid(), Some(60));
}

#[test]
fn test_best_ask_is_lowest() {
    let mut book = OrderBook::new();
    book.submit_order(sell(70, 5));
    book.submit_order(sell(50, 5));
    book.submit_order(sell(60, 5));
    assert_eq!(book.best_ask(), Some(50));
}

#[test]
fn test_depth() {
    let mut book = OrderBook::new();
    book.submit_order(buy(50, 10));
    book.submit_order(buy(50, 5));
    book.submit_order(buy(48, 20));

    let depth = book.bids.depth();
    assert_eq!(depth.len(), 2); // two price levels
    let level_50 = depth.iter().find(|l| l.price_cents == 50).unwrap();
    assert_eq!(level_50.total_quantity, 15);
    assert_eq!(level_50.order_count, 2);
}

// === Matching engine tests ===

#[test]
fn test_no_match_when_no_cross() {
    let mut book = OrderBook::new();
    book.submit_order(buy(40, 10));
    book.submit_order(sell(60, 10));

    let trades = matching::match_orders(&mut book, MarketMode::People, None).trades;
    assert!(trades.is_empty());
}

#[test]
fn test_match_at_ask_price() {
    let mut book = OrderBook::new();
    // Resting ask at 50
    book.submit_order(sell(50, 10));
    // Aggressive bid at 55 (crosses the ask)
    book.submit_order(buy(55, 10));

    let trades = matching::match_orders(&mut book, MarketMode::People, None).trades;
    assert_eq!(trades.len(), 1);
    assert_eq!(trades[0].price_cents, 50); // executes at resting ask price
    assert_eq!(trades[0].quantity, 10);
}

#[test]
fn test_partial_fill() {
    let mut book = OrderBook::new();
    book.submit_order(sell(50, 20));
    book.submit_order(buy(50, 10));

    let trades = matching::match_orders(&mut book, MarketMode::People, None).trades;
    assert_eq!(trades.len(), 1);
    assert_eq!(trades[0].quantity, 10);

    // 10 remaining on the ask
    assert_eq!(book.best_ask(), Some(50));
    let depth = book.asks.depth();
    assert_eq!(depth[0].total_quantity, 10);
}

#[test]
fn test_multiple_fills() {
    let mut book = OrderBook::new();
    book.submit_order(sell(50, 5));
    book.submit_order(sell(51, 5));
    book.submit_order(buy(55, 8)); // sweeps both levels

    let trades = matching::match_orders(&mut book, MarketMode::People, None).trades;
    assert_eq!(trades.len(), 2);
    assert_eq!(trades[0].price_cents, 50);
    assert_eq!(trades[0].quantity, 5);
    assert_eq!(trades[1].price_cents, 51);
    assert_eq!(trades[1].quantity, 3);
}

#[test]
fn test_self_trade_prevention() {
    let mut book = OrderBook::new();
    let user_id = Uuid::new_v4();

    let mut bid = buy(50, 10);
    bid.user_id = user_id;
    let mut ask = sell(50, 10);
    ask.user_id = user_id;

    book.submit_order(ask);
    book.submit_order(bid);

    let trades = matching::match_orders(&mut book, MarketMode::People, None).trades;
    assert!(trades.is_empty()); // self-trade cancelled
}

#[test]
fn test_fees_applied_to_trades() {
    let mut book = OrderBook::new();
    let mut ask = sell(50, 10);
    ask.created_at = Utc::now() - chrono::Duration::seconds(10); // resting (maker)
    book.submit_order(ask);
    book.submit_order(buy(50, 10)); // aggressive (taker)

    let trades = matching::match_orders(&mut book, MarketMode::People, None).trades;
    assert_eq!(trades.len(), 1);
    // Buyer is taker (newer), seller is maker
    assert!(trades[0].buyer_fee > trades[0].seller_fee);
    assert!(trades[0].buyer_fee > rust_decimal::Decimal::ZERO);
    assert!(trades[0].seller_fee > rust_decimal::Decimal::ZERO);
}

// === Batch engine tests ===

#[tokio::test]
async fn test_batch_submit_and_execute() {
    let engine = BatchEngine::new(500);

    engine.submit(buy(55, 10)).await;
    engine.submit(sell(50, 10)).await;

    let (batch_id, result) = engine.execute_batch().await;
    let trades = result.trades;
    assert!(!batch_id.is_nil());
    assert_eq!(trades.len(), 1);
    // Uniform-price call auction: bid 55 / ask 50 clear at the midpoint (52).
    assert_eq!(trades[0].price_cents, 52);
    assert!(trades[0].batch_id.is_some());
}

#[tokio::test]
async fn test_batch_empty_produces_no_trades() {
    let engine = BatchEngine::new(500);
    let (_, result) = engine.execute_batch().await;
    assert!(result.trades.is_empty());
}

#[tokio::test]
async fn test_batch_cancel_from_queue() {
    let engine = BatchEngine::new(500);
    let id = engine.submit(buy(50, 10)).await;

    let cancelled = engine.cancel(id).await;
    assert!(cancelled.is_some());

    let (_, result) = engine.execute_batch().await;
    assert!(result.trades.is_empty());
}

#[tokio::test]
async fn test_batch_ioc_cancelled_if_unfilled() {
    let engine = BatchEngine::new(500);

    let mut ioc = buy(50, 10);
    ioc.time_in_force = TimeInForce::Ioc;
    engine.submit(ioc).await;
    // No matching sell — IOC should be removed

    let (_, result) = engine.execute_batch().await;
    assert!(result.trades.is_empty());

    let snap = engine.snapshot().await;
    assert_eq!(snap.best_bid, None); // IOC was cleaned up
}

// === Continuous engine tests ===

#[tokio::test]
async fn test_continuous_immediate_match() {
    let engine = ContinuousEngine::new();

    // Place resting sell
    engine.submit(sell(50, 10)).await;

    // Aggressive buy matches immediately
    let (_, result) = engine.submit(buy(50, 10)).await;
    let trades = result.trades;
    assert_eq!(trades.len(), 1);
    assert_eq!(trades[0].price_cents, 50);
    assert!(trades[0].batch_id.is_none()); // continuous has no batch_id
}

#[tokio::test]
async fn test_continuous_no_match() {
    let engine = ContinuousEngine::new();
    let (_, result) = engine.submit(buy(40, 10)).await;
    assert!(result.trades.is_empty());

    let snap = engine.snapshot().await;
    assert_eq!(snap.best_bid, Some(40));
}

#[tokio::test]
async fn test_continuous_cancel() {
    let engine = ContinuousEngine::new();
    let (id, _) = engine.submit(buy(50, 10)).await;

    let cancelled = engine.cancel(id).await;
    assert!(cancelled.is_some());

    let snap = engine.snapshot().await;
    assert_eq!(snap.best_bid, None);
}

#[tokio::test]
async fn test_continuous_ioc_partial_fill_cancels_remainder() {
    let engine = ContinuousEngine::new();

    engine.submit(sell(50, 5)).await;

    let mut ioc = buy(50, 10);
    ioc.mode = MarketMode::Unlimited;
    ioc.time_in_force = TimeInForce::Ioc;

    let (_, result) = engine.submit(ioc).await;
    let trades = result.trades;
    assert_eq!(trades.len(), 1);
    assert_eq!(trades[0].quantity, 5);

    let snap = engine.snapshot().await;
    assert_eq!(snap.best_bid, None);
    assert_eq!(snap.best_ask, None);
}

#[tokio::test]
async fn test_continuous_fok_rejects_partial_fill() {
    let engine = ContinuousEngine::new();

    engine.submit(sell(50, 5)).await;

    let mut fok = buy(50, 10);
    fok.mode = MarketMode::Unlimited;
    fok.time_in_force = TimeInForce::Fok;

    let (_, result) = engine.submit(fok).await;
    assert!(result.trades.is_empty());

    let snap = engine.snapshot().await;
    assert_eq!(snap.best_bid, None);
    assert_eq!(snap.best_ask, Some(50));
    assert_eq!(snap.asks[0].total_quantity, 5);
}

// === Dual mode engine tests ===

#[tokio::test]
async fn test_dual_mode_separate_books() {
    use crate::engine::DualModeEngine;

    let market_id = Uuid::new_v4();
    let engine = DualModeEngine::new(market_id, 500);

    // Submit to people mode (goes to queue)
    engine.submit(buy(50, 10)).await;

    // Execute batch to move order from queue to book
    engine.execute_people_batch().await;

    // Submit to bot mode (goes directly to continuous book)
    let mut bot_sell = sell(50, 10);
    bot_sell.mode = MarketMode::Unlimited;
    engine.submit(bot_sell).await;

    let snaps = engine.snapshots().await;

    // People book has a bid, bot book has an ask — they don't cross each other
    assert_eq!(snaps.people.best_bid, Some(50));
    assert_eq!(snaps.people.best_ask, None);
    assert_eq!(snaps.bot.best_bid, None);
    assert_eq!(snaps.bot.best_ask, Some(50));
}

// === YES/NO segregation (C1) ===

#[tokio::test]
async fn test_yes_and_no_do_not_cross() {
    use crate::continuous::ContinuousEngine;
    let engine = ContinuousEngine::new();

    // A resting YES sell and an incoming NO buy at a crossing price must NOT
    // trade — they are opposite instruments living in separate books.
    let mut yes_sell = make_order(Side::Yes, Action::Sell, 50, 10, MarketMode::Unlimited);
    yes_sell.user_id = Uuid::new_v4();
    engine.submit(yes_sell).await;

    let mut no_buy = make_order(Side::No, Action::Buy, 60, 10, MarketMode::Unlimited);
    no_buy.user_id = Uuid::new_v4();
    let (_, result) = engine.submit(no_buy).await;

    assert!(result.trades.is_empty(), "YES and NO must not match each other");
    // YES sell still rests; NO buy rests in its own book.
    assert_eq!(engine.snapshot_side(Side::Yes).await.best_ask, Some(50));
    assert_eq!(engine.snapshot_side(Side::No).await.best_bid, Some(60));
}

#[tokio::test]
async fn test_same_side_still_matches() {
    use crate::continuous::ContinuousEngine;
    let engine = ContinuousEngine::new();

    let mut no_sell = make_order(Side::No, Action::Sell, 40, 10, MarketMode::Unlimited);
    no_sell.user_id = Uuid::new_v4();
    engine.submit(no_sell).await;

    let mut no_buy = make_order(Side::No, Action::Buy, 45, 10, MarketMode::Unlimited);
    no_buy.user_id = Uuid::new_v4();
    let (_, result) = engine.submit(no_buy).await;

    assert_eq!(result.trades.len(), 1);
    assert_eq!(result.trades[0].side, Side::No);
    assert_eq!(result.trades[0].price_cents, 40); // resting NO ask price
}

// === Cancel propagation (H1) ===

#[tokio::test]
async fn test_ioc_remainder_reported_as_cancelled() {
    use crate::continuous::ContinuousEngine;
    let engine = ContinuousEngine::new();

    engine.submit(sell(50, 5)).await;

    let mut ioc = buy(50, 10);
    ioc.mode = MarketMode::Unlimited;
    ioc.time_in_force = TimeInForce::Ioc;
    let ioc_id = ioc.id;

    let (_, result) = engine.submit(ioc).await;
    assert_eq!(result.trades.len(), 1); // 5 filled
    assert!(result.cancelled.contains(&ioc_id)); // 5 unfilled remainder reported
}

#[tokio::test]
async fn test_batch_ioc_reports_cancelled() {
    let engine = BatchEngine::new(500);
    let mut ioc = buy(50, 10);
    ioc.time_in_force = TimeInForce::Ioc;
    let ioc_id = ioc.id;
    engine.submit(ioc).await;

    let (_, result) = engine.execute_batch().await;
    assert!(result.trades.is_empty());
    assert!(result.cancelled.contains(&ioc_id)); // unfilled IOC released, not left resting
}

// === Uniform-price auction (M2) ===

#[test]
fn test_auction_uniform_clearing_price() {
    let mut book = OrderBook::new();
    // Two buyers and two sellers crossing; all clear at one price.
    book.submit_order(buy(60, 10));
    book.submit_order(buy(58, 10));
    book.submit_order(sell(50, 10));
    book.submit_order(sell(52, 10));

    let result = matching::match_auction(&mut book, MarketMode::People, None);
    // All fills print at the same clearing price.
    let prices: std::collections::HashSet<u32> =
        result.trades.iter().map(|t| t.price_cents).collect();
    assert_eq!(prices.len(), 1, "auction must use a single clearing price");
    // Both sides pay the maker rate (equal treatment).
    for t in &result.trades {
        assert_eq!(t.buyer_fee, t.seller_fee);
    }
}
