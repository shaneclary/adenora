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
                        axum::http::header::HeaderName::from_static("x-bot-key"),
                    ])
                    .allow_credentials(true)
            }
        })
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    tracing::info!("Adenora listening on {bind_addr}");
    axum::serve(listener, app).await?;

    Ok(())
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
                let (_batch_id, trades) = engine.execute_people_batch().await;
                if !trades.is_empty() {
                    routes::trading::persist_trades(&st, &trades).await;
                    tracing::debug!(
                        market_id = %engine.market_id,
                        trades = trades.len(),
                        "batch executed"
                    );
                }
            });
        }
    }
}
