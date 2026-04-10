use adenora_common::types::{Action, MarketMode, OrderStatus, Side, TimeInForce};
use adenora_orderbook::book::{BookSnapshot, Order, Trade};
use adenora_orderbook::continuous::ContinuousEngine;
use chrono::{Duration, TimeZone, Utc};
use uuid::Uuid;

fn make_order(
    seq: i64,
    user_index: u128,
    action: Action,
    price_cents: u32,
    quantity: u32,
    time_in_force: TimeInForce,
) -> Order {
    let market_id = Uuid::from_u128(1);
    let created_at = Utc
        .with_ymd_and_hms(2026, 1, 1, 0, 0, 0)
        .single()
        .unwrap()
        + Duration::milliseconds(seq);

    Order {
        id: Uuid::from_u128(10_000 + seq as u128),
        user_id: Uuid::from_u128(20_000 + user_index),
        market_id,
        side: Side::Yes,
        action,
        price_cents,
        quantity,
        filled_quantity: 0,
        time_in_force,
        status: OrderStatus::Pending,
        mode: MarketMode::Unlimited,
        bot_id: None,
        created_at,
    }
}

fn total_resting_qty(levels: &[adenora_orderbook::book::PriceLevel]) -> u32 {
    levels.iter().map(|level| level.total_quantity).sum()
}

fn score_snapshot(snapshot: &BookSnapshot, trades: &[Trade]) -> i32 {
    let total_quantity: u32 = trades.iter().map(|trade| trade.quantity).sum();
    let remaining_qty = total_resting_qty(&snapshot.bids) + total_resting_qty(&snapshot.asks);
    let spread_penalty = snapshot.spread.unwrap_or(0).max(0) * 10;
    (total_quantity as i32 * 100) - spread_penalty - remaining_qty as i32
}

#[tokio::main]
async fn main() {
    let engine = ContinuousEngine::new();
    let flow = [
        make_order(1, 1, Action::Sell, 52, 10, TimeInForce::Gtc),
        make_order(2, 2, Action::Sell, 54, 6, TimeInForce::Gtc),
        make_order(3, 3, Action::Sell, 56, 12, TimeInForce::Gtc),
        make_order(4, 4, Action::Buy, 47, 12, TimeInForce::Gtc),
        make_order(5, 5, Action::Buy, 55, 8, TimeInForce::Gtc),
        make_order(6, 6, Action::Sell, 47, 5, TimeInForce::Ioc),
        make_order(7, 7, Action::Buy, 54, 10, TimeInForce::Gtc),
        make_order(8, 8, Action::Sell, 54, 2, TimeInForce::Fok),
        make_order(9, 9, Action::Buy, 50, 4, TimeInForce::Ioc),
    ];

    let mut all_trades = Vec::new();
    for order in flow {
        let (_, trades) = engine.submit(order).await;
        all_trades.extend(trades);
    }

    let snapshot = engine.snapshot().await;
    let total_quantity: u32 = all_trades.iter().map(|trade| trade.quantity).sum();
    let total_notional_cents: u32 = all_trades
        .iter()
        .map(|trade| trade.price_cents * trade.quantity)
        .sum();
    let resting_bid_qty = total_resting_qty(&snapshot.bids);
    let resting_ask_qty = total_resting_qty(&snapshot.asks);
    let research_score = score_snapshot(&snapshot, &all_trades);

    println!("RESEARCH_SCORE={research_score}");
    println!("TOTAL_TRADES={}", all_trades.len());
    println!("TOTAL_QUANTITY={total_quantity}");
    println!("TOTAL_NOTIONAL_CENTS={total_notional_cents}");
    println!(
        "FINAL_BEST_BID={}",
        snapshot
            .best_bid
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string())
    );
    println!(
        "FINAL_BEST_ASK={}",
        snapshot
            .best_ask
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string())
    );
    println!(
        "FINAL_SPREAD={}",
        snapshot
            .spread
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string())
    );
    println!("RESTING_BID_QTY={resting_bid_qty}");
    println!("RESTING_ASK_QTY={resting_ask_qty}");
}
