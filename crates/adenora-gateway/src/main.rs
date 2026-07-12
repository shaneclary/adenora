use adenora_common::config::AdenoraConfig;
use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{get, post},
};
use tower_http::cors::{CorsLayer, AllowOrigin};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

mod auth_extractor;
mod middleware;
mod routes;
mod services;
mod state;
mod ws;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = AdenoraConfig::from_env();
    if let Err(e) = config.validate() {
        tracing::error!("{e}");
        anyhow::bail!("refusing to start with insecure configuration: {e}");
    }
    let bind_addr = config.server.bind_addr;
    tracing::info!("Adenora starting on {bind_addr}");

    let state = state::AppState::new(config)
        .await
        .expect("failed to connect to database");

    sqlx::migrate!("../../migrations")
        .run(&state.db)
        .await
        .expect("failed to run migrations");
    tracing::info!("migrations applied");

    // Rehydrate resting orders into in-memory engines before batches run, so a
    // restart doesn't strand open GTC orders (funds reserved, book empty).
    rehydrate_open_orders(&state).await;

    // Spawn background services
    let batch_state = state.clone();
    tokio::spawn(async move { batch_loop(batch_state).await });

    let settle_state = state.clone();
    tokio::spawn(async move { services::settlement::settlement_loop(settle_state).await });

    let draw_state = state.clone();
    tokio::spawn(async move { services::draw_executor::draw_loop(draw_state).await });

    let lb_state = state.clone();
    tokio::spawn(async move { services::leaderboard::leaderboard_loop(lb_state).await });

    let recovery_state = state.clone();
    tokio::spawn(async move { services::recovery::recovery_loop(recovery_state).await });

    let app = Router::new()
        // Health & Status
        .route("/", get(routes::health::root))
        .route("/healthz", get(routes::health::healthz))
        .route("/api/v1/status", get(routes::health::status))
        // Auth (public)
        .route("/api/v1/auth/register", post(routes::auth::register))
        .route("/api/v1/auth/login", post(routes::auth::login))
        // Markets (public read)
        .route("/api/v1/markets", get(routes::markets::list_markets))
        .route("/api/v1/markets/{id}", get(routes::markets::get_market))
        .route("/api/v1/markets/{id}/book", get(routes::markets::get_orderbook))
        // Markets (authenticated)
        .route("/api/v1/markets/propose", post(routes::markets::propose_market))
        // Predict (human-friendly, authenticated)
        .route("/api/v1/predict", post(routes::predict::predict))
        .route("/api/v1/predictions", get(routes::predict::get_predictions))
        .route("/api/v1/impact/feed", get(routes::predict::impact_feed))
        .route("/api/v1/markets/cards", get(routes::predict::list_market_cards))
        .route("/api/v1/markets/{id}/card", get(routes::predict::get_market_card))
        .route("/api/v1/markets/{id}/debate", get(routes::predict::get_market_debate))
        // Trading (authenticated — advanced)
        .route("/api/v1/orders", post(routes::trading::place_order))
        .route("/api/v1/orders/{id}/cancel", post(routes::trading::cancel_order))
        .route("/api/v1/positions", get(routes::trading::get_positions))
        // Lottery
        .route("/api/v1/lottery", get(routes::lottery::list_lotteries))
        .route("/api/v1/lottery/{id}/tickets", post(routes::lottery::buy_ticket))
        .route("/api/v1/lottery/{id}/draws", get(routes::lottery::list_draws))
        // Gaming
        .route("/api/v1/games", get(routes::gaming::list_games))
        .route("/api/v1/tournaments", get(routes::gaming::list_tournaments))
        .route("/api/v1/leaderboard", get(routes::gaming::leaderboard))
        .route("/api/v1/trivia/start", post(routes::gaming::start_trivia))
        .route("/api/v1/trivia/answer", post(routes::gaming::answer_trivia))
        .route("/api/v1/forecast/submit", post(routes::gaming::submit_forecast))
        // Bot Arena
        .route("/api/v1/bots", get(routes::bots::list_bots))
        .route("/api/v1/bots/register", post(routes::bots::register_bot))
        .route("/api/v1/bots/{id}", get(routes::bots::get_bot))
        .route("/api/v1/bots/leaderboard", get(routes::bots::bot_leaderboard))
        .route("/api/v1/bots/tournaments", get(routes::bots::list_bot_tournaments))
        // Charity
        .route("/api/v1/charity/projects", get(routes::charity::list_projects))
        .route("/api/v1/charity/ledger", get(routes::charity::get_ledger))
        .route("/api/v1/charity/summary", get(routes::charity::funding_summary))
        // Cause Campaigns
        .route("/api/v1/campaigns", get(routes::campaigns::list_campaigns))
        .route("/api/v1/campaigns/donate", post(routes::campaigns::donate))
        .route("/api/v1/campaigns/{slug}", get(routes::campaigns::get_campaign))
        .route("/api/v1/campaigns/{slug}/donors", get(routes::campaigns::get_donors))
        // Community Votes
        .route("/api/v1/votes", get(routes::votes::list_campaigns).post(routes::votes::create_campaign))
        .route("/api/v1/votes/{id}", get(routes::votes::get_campaign))
        .route("/api/v1/votes/{id}/vote", post(routes::votes::cast_vote))
        // Wallet (authenticated)
        .route("/api/v1/wallet", get(routes::wallet::get_wallet))
        .route("/api/v1/wallet/deposit", post(routes::wallet::deposit))
        .route("/api/v1/wallet/withdraw", post(routes::wallet::withdraw))
        // KYC
        .route("/api/v1/kyc/start", post(routes::kyc::start_kyc))
        .route("/api/v1/kyc/webhook", post(routes::kyc::kyc_webhook))
        .route("/api/v1/kyc/status", get(routes::kyc::kyc_status))
        // WebSocket
        .route("/ws", get(ws::ws_handler))
        // Middleware (applied outermost-last, innermost-first)
        .layer(from_fn_with_state(
            state.clone(),
            crate::middleware::rate_limit::rate_limit_middleware,
        ))
        .layer(TraceLayer::new_for_http())
        .layer({
            let origins = &state.config.server.cors_origins;
            if origins.len() == 1 && origins[0] == "*" {
                CorsLayer::permissive()
            } else {
                CorsLayer::new()
                    .allow_origin(AllowOrigin::list(
                        origins.iter().filter_map(|o| o.parse().ok()),
                    ))
                    .allow_methods([
                        axum::http::Method::GET,
                        axum::http::Method::POST,
                        axum::http::Method::PUT,
                        axum::http::Method::DELETE,
                        axum::http::Method::OPTIONS,
                    ])
                    .allow_headers([
                        axum::http::header::CONTENT_TYPE,
                        axum::http::header::AUTHORIZATION,
                    ])
                    .allow_credentials(true)
            }
        })
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    tracing::info!("Adenora listening on {bind_addr}");
    // `into_make_service_with_connect_info` exposes the socket peer address to
    // the rate-limit middleware so it can key on the real client IP.
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await?;

    Ok(())
}

/// Reload resting GTC orders on active markets into their in-memory engines.
/// Runs once at startup. Only GTC orders are meant to rest; IOC/FOK are
/// immediate and must not be rehydrated.
async fn rehydrate_open_orders(state: &state::AppState) {
    use adenora_common::types::*;
    use adenora_orderbook::book::Order;
    use uuid::Uuid;

    /// Row for an open order rehydrated from the database on startup.
    type OpenOrderRow = (
        Uuid, Uuid, Uuid, String, String, i32, i32, i32, String, Option<Uuid>,
        chrono::DateTime<chrono::Utc>,
    );

    let rows: Vec<OpenOrderRow> =
        sqlx::query_as(
            "SELECT o.id, o.user_id, o.market_id, o.side, o.action, o.price_cents,
                    o.quantity, o.filled_quantity, o.mode, o.bot_id, o.created_at
             FROM orders o
             JOIN markets m ON m.id = o.market_id
             WHERE o.status IN ('pending', 'partial_fill')
               AND o.time_in_force = 'gtc'
               AND m.status = 'active'"
        )
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

    let mut count = 0u32;
    for (id, user_id, market_id, side_s, action_s, price_cents, quantity, filled, mode_s, bot_id, created_at) in rows {
        let side = if side_s == "no" { Side::No } else { Side::Yes };
        let action = if action_s == "sell" { Action::Sell } else { Action::Buy };
        let mode = if mode_s == "unlimited" { MarketMode::Unlimited } else { MarketMode::People };
        let status = if filled > 0 { OrderStatus::PartialFill } else { OrderStatus::Pending };

        let order = Order {
            id,
            user_id,
            market_id,
            side,
            action,
            price_cents: price_cents.max(0) as u32,
            quantity: quantity.max(0) as u32,
            filled_quantity: filled.max(0) as u32,
            time_in_force: TimeInForce::Gtc,
            status,
            mode,
            bot_id,
            created_at,
        };

        let engine = state.get_engine(market_id).await;
        engine.rehydrate(order).await;
        count += 1;
    }

    if count > 0 {
        tracing::info!(orders = count, "rehydrated resting orders into engines");
    }
}

/// Background: execute batch auctions + persist trades for all people-mode markets.
async fn batch_loop(state: state::AppState) {
    let interval_ms = state.config.trading.batch_interval_ms;
    let mut ticker = tokio::time::interval(tokio::time::Duration::from_millis(interval_ms));

    loop {
        ticker.tick().await;

        let engines = state.engines.read().await;
        for (_market_id, engine) in engines.iter() {
            let engine = engine.clone();
            let st = state.clone();
            tokio::spawn(async move {
                let (_batch_id, result) = engine.execute_people_batch().await;
                // Persist fills first so filled_quantity is up to date, then
                // release funds for any orders the auction cancelled (unfilled
                // IOC/FOK, self-trade) using the post-fill quantities.
                if !result.trades.is_empty() {
                    routes::trading::persist_trades(&st, &result.trades).await;
                    tracing::debug!(
                        market_id = %engine.market_id,
                        trades = result.trades.len(),
                        "batch executed"
                    );
                }
                if !result.cancelled.is_empty() {
                    routes::trading::release_cancelled_orders(&st, &result.cancelled).await;
                }
            });
        }
    }
}
