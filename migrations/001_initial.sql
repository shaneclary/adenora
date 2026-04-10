-- Adenora: Initial Schema
-- Unified platform for gaming, prediction markets, and public benefit

-- =============================================================================
-- USERS & AUTH
-- =============================================================================

CREATE TABLE users (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email           TEXT NOT NULL UNIQUE,
    password_hash   TEXT NOT NULL,
    display_name    TEXT NOT NULL,
    locale          TEXT NOT NULL DEFAULT 'en',
    preferred_currency TEXT NOT NULL DEFAULT 'EUR',
    kyc_status      TEXT NOT NULL DEFAULT 'none',  -- none, pending, verified, rejected
    date_of_birth   DATE,
    country_code    TEXT NOT NULL DEFAULT 'XK',     -- ISO 3166-1 alpha-2
    is_bot_account  BOOLEAN NOT NULL DEFAULT FALSE,
    is_admin        BOOLEAN NOT NULL DEFAULT FALSE,
    -- Responsible gambling limits
    daily_deposit_limit   NUMERIC,
    weekly_deposit_limit  NUMERIC,
    monthly_deposit_limit NUMERIC,
    self_exclusion_until  TIMESTAMPTZ,
    cooling_off_until     TIMESTAMPTZ,
    loss_alert_threshold  NUMERIC,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE refresh_tokens (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash  TEXT NOT NULL UNIQUE,
    expires_at  TIMESTAMPTZ NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_refresh_tokens_user ON refresh_tokens(user_id);

-- =============================================================================
-- WALLETS & TRANSACTIONS
-- =============================================================================

CREATE TABLE wallets (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    currency    TEXT NOT NULL,    -- EUR, ALL, MKD, USDC, etc.
    available   NUMERIC NOT NULL DEFAULT 0,
    reserved    NUMERIC NOT NULL DEFAULT 0,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, currency)
);

CREATE INDEX idx_wallets_user ON wallets(user_id);

CREATE TABLE transactions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id),
    wallet_id       UUID NOT NULL REFERENCES wallets(id),
    tx_type         TEXT NOT NULL,  -- deposit, withdrawal, trade_buy, trade_sell, fee, prize, ticket, refund
    amount          NUMERIC NOT NULL,
    currency        TEXT NOT NULL,
    reference_type  TEXT,           -- order, trade, ticket, draw, tournament
    reference_id    UUID,
    description     TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_transactions_user ON transactions(user_id);
CREATE INDEX idx_transactions_wallet ON transactions(wallet_id);
CREATE INDEX idx_transactions_created ON transactions(created_at);

-- =============================================================================
-- CHARITY PROJECTS & LEDGER
-- =============================================================================

CREATE TABLE charity_projects (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            TEXT NOT NULL,
    description     TEXT NOT NULL DEFAULT '',
    category        TEXT NOT NULL DEFAULT 'other',
    country_codes   TEXT[] NOT NULL DEFAULT '{}',
    logo_url        TEXT,
    website_url     TEXT,
    total_received  NUMERIC NOT NULL DEFAULT 0,
    currency        TEXT NOT NULL DEFAULT 'EUR',
    is_active       BOOLEAN NOT NULL DEFAULT TRUE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE charity_ledger (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id      UUID NOT NULL REFERENCES charity_projects(id),
    source          TEXT NOT NULL,  -- prediction_fee, lottery_revenue, tournament_fee, direct_donation, help_game
    amount          NUMERIC NOT NULL,
    currency        TEXT NOT NULL,
    description     TEXT NOT NULL DEFAULT '',
    reference_id    TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_charity_ledger_project ON charity_ledger(project_id);
CREATE INDEX idx_charity_ledger_created ON charity_ledger(created_at);

-- =============================================================================
-- PREDICTION MARKETS
-- =============================================================================

CREATE TABLE markets (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    question            TEXT NOT NULL,
    description         TEXT NOT NULL DEFAULT '',
    category            TEXT NOT NULL DEFAULT 'custom',
    outcomes            JSONB NOT NULL DEFAULT '["Yes","No"]',
    status              TEXT NOT NULL DEFAULT 'proposed', -- proposed, active, closed, resolving, disputed, settled
    creator_id          UUID NOT NULL REFERENCES users(id),
    charity_project_id  UUID REFERENCES charity_projects(id),
    resolution_source   TEXT,
    resolution_criteria TEXT,
    outcome             TEXT,         -- yes, no, index:N, void
    country_codes       TEXT[] NOT NULL DEFAULT '{}',
    opens_at            TIMESTAMPTZ NOT NULL,
    closes_at           TIMESTAMPTZ NOT NULL,
    resolved_at         TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_markets_status ON markets(status);
CREATE INDEX idx_markets_category ON markets(category);
CREATE INDEX idx_markets_closes ON markets(closes_at);
CREATE INDEX idx_markets_creator ON markets(creator_id);

CREATE TABLE market_proposals (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    proposer_id     UUID NOT NULL REFERENCES users(id),
    request         JSONB NOT NULL,
    status          TEXT NOT NULL DEFAULT 'pending',  -- pending, approved, rejected
    review_notes    TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- =============================================================================
-- ORDERS & TRADES
-- =============================================================================

CREATE TABLE orders (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id),
    market_id       UUID NOT NULL REFERENCES markets(id),
    side            TEXT NOT NULL,       -- yes, no
    action          TEXT NOT NULL,       -- buy, sell
    price_cents     INTEGER NOT NULL,
    quantity        INTEGER NOT NULL,
    filled_quantity INTEGER NOT NULL DEFAULT 0,
    time_in_force   TEXT NOT NULL DEFAULT 'ioc',   -- ioc, gtc, fok
    status          TEXT NOT NULL DEFAULT 'pending', -- pending, partial_fill, filled, cancelled, expired
    mode            TEXT NOT NULL DEFAULT 'people',  -- people, bot
    bot_id          UUID,
    batch_id        UUID,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_orders_user ON orders(user_id);
CREATE INDEX idx_orders_market ON orders(market_id);
CREATE INDEX idx_orders_status ON orders(status);
CREATE INDEX idx_orders_mode ON orders(mode);
CREATE INDEX idx_orders_batch ON orders(batch_id);

CREATE TABLE trades (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    market_id       UUID NOT NULL REFERENCES markets(id),
    batch_id        UUID,
    buyer_order_id  UUID NOT NULL REFERENCES orders(id),
    seller_order_id UUID NOT NULL REFERENCES orders(id),
    buyer_user_id   UUID NOT NULL REFERENCES users(id),
    seller_user_id  UUID NOT NULL REFERENCES users(id),
    side            TEXT NOT NULL,
    price_cents     INTEGER NOT NULL,
    quantity        INTEGER NOT NULL,
    buyer_fee       NUMERIC NOT NULL DEFAULT 0,
    seller_fee      NUMERIC NOT NULL DEFAULT 0,
    mode            TEXT NOT NULL,       -- people, bot
    executed_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_trades_market ON trades(market_id);
CREATE INDEX idx_trades_buyer ON trades(buyer_user_id);
CREATE INDEX idx_trades_seller ON trades(seller_user_id);
CREATE INDEX idx_trades_executed ON trades(executed_at);

CREATE TABLE positions (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id),
    market_id   UUID NOT NULL REFERENCES markets(id),
    side        TEXT NOT NULL,
    quantity    INTEGER NOT NULL DEFAULT 0,
    avg_price   NUMERIC NOT NULL DEFAULT 0,
    mode        TEXT NOT NULL DEFAULT 'people',
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, market_id, side, mode)
);

CREATE INDEX idx_positions_user ON positions(user_id);
CREATE INDEX idx_positions_market ON positions(market_id);

-- =============================================================================
-- ORACLE & DISPUTES
-- =============================================================================

CREATE TABLE oracle_evidence (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    market_id           UUID NOT NULL REFERENCES markets(id),
    source_name         TEXT NOT NULL,
    source_url          TEXT NOT NULL,
    raw_data            JSONB NOT NULL,
    interpreted_outcome TEXT NOT NULL,
    confidence          DOUBLE PRECISION NOT NULL DEFAULT 0,
    collected_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_oracle_evidence_market ON oracle_evidence(market_id);

CREATE TABLE disputes (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    market_id       UUID NOT NULL REFERENCES markets(id),
    challenger_id   UUID NOT NULL REFERENCES users(id),
    reason          TEXT NOT NULL,
    evidence_links  TEXT[] NOT NULL DEFAULT '{}',
    status          TEXT NOT NULL DEFAULT 'filed',  -- filed, voting, resolved, escalated
    outcome         TEXT,
    resolution_method TEXT,  -- oracle_consensus, jury_verdict, panel_decision, void
    resolution_notes TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at     TIMESTAMPTZ
);

CREATE INDEX idx_disputes_market ON disputes(market_id);

CREATE TABLE jury_votes (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    dispute_id  UUID NOT NULL REFERENCES disputes(id),
    juror_id    UUID NOT NULL REFERENCES users(id),
    vote        TEXT NOT NULL,
    reasoning   TEXT,
    voted_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(dispute_id, juror_id)
);

-- =============================================================================
-- LOTTERY
-- =============================================================================

CREATE TABLE lotteries (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            TEXT NOT NULL,
    description     TEXT NOT NULL DEFAULT '',
    game_type       TEXT NOT NULL,       -- draw, scratch, fifty_fifty, prize_raffle, numbers
    project_id      UUID NOT NULL REFERENCES charity_projects(id),
    country_codes   TEXT[] NOT NULL DEFAULT '{}',
    ticket_price    NUMERIC NOT NULL,
    currency        TEXT NOT NULL DEFAULT 'EUR',
    prize_pct       INTEGER NOT NULL DEFAULT 50,
    project_pct     INTEGER NOT NULL DEFAULT 25,
    company_pct     INTEGER NOT NULL DEFAULT 25,
    status          TEXT NOT NULL DEFAULT 'draft',  -- draft, active, suspended, completed
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE draws (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    lottery_id      UUID NOT NULL REFERENCES lotteries(id),
    draw_number     INTEGER NOT NULL,
    scheduled_at    TIMESTAMPTZ NOT NULL,
    executed_at     TIMESTAMPTZ,
    winning_numbers INTEGER[],
    prize_pool      NUMERIC NOT NULL DEFAULT 0,
    status          TEXT NOT NULL DEFAULT 'scheduled', -- scheduled, open, closed, drawn, paid
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_draws_lottery ON draws(lottery_id);
CREATE INDEX idx_draws_status ON draws(status);

CREATE TABLE tickets (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    lottery_id      UUID NOT NULL REFERENCES lotteries(id),
    draw_id         UUID NOT NULL REFERENCES draws(id),
    user_id         UUID NOT NULL REFERENCES users(id),
    numbers         INTEGER[] NOT NULL,
    is_free_entry   BOOLEAN NOT NULL DEFAULT FALSE,
    purchased_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_tickets_user ON tickets(user_id);
CREATE INDEX idx_tickets_draw ON tickets(draw_id);

CREATE TABLE winners (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    draw_id     UUID NOT NULL REFERENCES draws(id),
    user_id     UUID NOT NULL REFERENCES users(id),
    ticket_id   UUID NOT NULL REFERENCES tickets(id),
    tier        TEXT NOT NULL,  -- jackpot, second, third, fourth, fifth, free
    amount      NUMERIC NOT NULL DEFAULT 0,
    paid_at     TIMESTAMPTZ
);

CREATE INDEX idx_winners_draw ON winners(draw_id);
CREATE INDEX idx_winners_user ON winners(user_id);

CREATE TABLE linked_jackpots (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            TEXT NOT NULL,
    linked_lottery_ids UUID[] NOT NULL DEFAULT '{}',
    combined_pool   NUMERIC NOT NULL DEFAULT 0,
    country_codes   TEXT[] NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- =============================================================================
-- GAMING
-- =============================================================================

CREATE TABLE games (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            TEXT NOT NULL,
    description     TEXT NOT NULL DEFAULT '',
    game_type       TEXT NOT NULL,  -- trivia, forecasting, strategy, speed_predict
    min_players     INTEGER NOT NULL DEFAULT 1,
    max_players     INTEGER NOT NULL DEFAULT 100,
    is_free_to_play BOOLEAN NOT NULL DEFAULT TRUE,
    entry_fee       NUMERIC,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE tournaments (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name                TEXT NOT NULL,
    game_id             UUID NOT NULL REFERENCES games(id),
    entry_fee           NUMERIC NOT NULL DEFAULT 0,
    prize_pool          NUMERIC NOT NULL DEFAULT 0,
    house_cut_pct       INTEGER NOT NULL DEFAULT 1,
    max_participants    INTEGER NOT NULL DEFAULT 100,
    current_participants INTEGER NOT NULL DEFAULT 0,
    status              TEXT NOT NULL DEFAULT 'registration', -- registration, in_progress, completed, cancelled
    starts_at           TIMESTAMPTZ NOT NULL,
    ends_at             TIMESTAMPTZ NOT NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_tournaments_status ON tournaments(status);
CREATE INDEX idx_tournaments_starts ON tournaments(starts_at);

CREATE TABLE tournament_entries (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tournament_id   UUID NOT NULL REFERENCES tournaments(id),
    user_id         UUID NOT NULL REFERENCES users(id),
    score           BIGINT,
    rank            INTEGER,
    prize_amount    NUMERIC,
    entered_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tournament_id, user_id)
);

CREATE TABLE game_results (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    game_id     UUID NOT NULL REFERENCES games(id),
    user_id     UUID NOT NULL REFERENCES users(id),
    score       BIGINT NOT NULL DEFAULT 0,
    rank        INTEGER,
    prize_amount NUMERIC,
    played_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_game_results_user ON game_results(user_id);

CREATE TABLE achievements (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    icon        TEXT NOT NULL DEFAULT '',
    criteria    JSONB NOT NULL
);

CREATE TABLE user_achievements (
    user_id         UUID NOT NULL REFERENCES users(id),
    achievement_id  UUID NOT NULL REFERENCES achievements(id),
    unlocked_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY(user_id, achievement_id)
);

-- =============================================================================
-- BOT ARENA
-- =============================================================================

CREATE TABLE bots (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id        UUID NOT NULL REFERENCES users(id),
    name            TEXT NOT NULL UNIQUE,
    description     TEXT,
    avatar_url      TEXT,
    is_open_source  BOOLEAN NOT NULL DEFAULT FALSE,
    source_url      TEXT,
    status          TEXT NOT NULL DEFAULT 'active',  -- active, suspended, retired
    -- Stats (denormalized for leaderboard performance)
    total_trades    BIGINT NOT NULL DEFAULT 0,
    total_volume    NUMERIC NOT NULL DEFAULT 0,
    total_pnl       NUMERIC NOT NULL DEFAULT 0,
    win_rate        DOUBLE PRECISION NOT NULL DEFAULT 0,
    sharpe_ratio    DOUBLE PRECISION NOT NULL DEFAULT 0,
    max_drawdown    DOUBLE PRECISION NOT NULL DEFAULT 0,
    markets_traded  INTEGER NOT NULL DEFAULT 0,
    tournaments_entered INTEGER NOT NULL DEFAULT 0,
    tournaments_won INTEGER NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    stats_updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_bots_owner ON bots(owner_id);
CREATE INDEX idx_bots_pnl ON bots(total_pnl DESC);
CREATE INDEX idx_bots_win_rate ON bots(win_rate DESC);

CREATE TABLE cage_matches (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    market_id   UUID NOT NULL REFERENCES markets(id),
    bot_a       UUID NOT NULL REFERENCES bots(id),
    bot_b       UUID NOT NULL REFERENCES bots(id),
    bot_a_pnl   NUMERIC NOT NULL DEFAULT 0,
    bot_b_pnl   NUMERIC NOT NULL DEFAULT 0,
    winner      UUID REFERENCES bots(id),
    status      TEXT NOT NULL DEFAULT 'scheduled', -- scheduled, live, completed
    started_at  TIMESTAMPTZ NOT NULL,
    ended_at    TIMESTAMPTZ
);

CREATE TABLE bot_tournaments (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            TEXT NOT NULL,
    description     TEXT NOT NULL DEFAULT '',
    market_ids      UUID[] NOT NULL DEFAULT '{}',
    entry_fee       NUMERIC NOT NULL DEFAULT 0,
    prize_pool      NUMERIC NOT NULL DEFAULT 0,
    max_bots        INTEGER NOT NULL DEFAULT 50,
    ranking_metric  TEXT NOT NULL DEFAULT 'total_pnl', -- total_pnl, sharpe_ratio, win_rate, combined
    status          TEXT NOT NULL DEFAULT 'registration',
    starts_at       TIMESTAMPTZ NOT NULL,
    ends_at         TIMESTAMPTZ NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE bot_tournament_entries (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tournament_id   UUID NOT NULL REFERENCES bot_tournaments(id),
    bot_id          UUID NOT NULL REFERENCES bots(id),
    total_pnl       NUMERIC NOT NULL DEFAULT 0,
    trades          INTEGER NOT NULL DEFAULT 0,
    win_rate        DOUBLE PRECISION NOT NULL DEFAULT 0,
    rank            INTEGER,
    prize           NUMERIC NOT NULL DEFAULT 0,
    UNIQUE(tournament_id, bot_id)
);

-- =============================================================================
-- LEADERBOARDS (materialized for performance)
-- =============================================================================

CREATE TABLE leaderboard_snapshots (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    scope       TEXT NOT NULL,     -- all_time, season, monthly, weekly, daily
    category    TEXT NOT NULL,     -- overall, predictions, gaming, bot_battle, trivia, donate_and_play
    entries     JSONB NOT NULL,
    total_participants INTEGER NOT NULL DEFAULT 0,
    generated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_leaderboard_scope ON leaderboard_snapshots(scope, category);

-- =============================================================================
-- FUNCTIONS
-- =============================================================================

-- Auto-update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER users_updated_at BEFORE UPDATE ON users FOR EACH ROW EXECUTE FUNCTION update_updated_at();
CREATE TRIGGER wallets_updated_at BEFORE UPDATE ON wallets FOR EACH ROW EXECUTE FUNCTION update_updated_at();
CREATE TRIGGER markets_updated_at BEFORE UPDATE ON markets FOR EACH ROW EXECUTE FUNCTION update_updated_at();
CREATE TRIGGER orders_updated_at BEFORE UPDATE ON orders FOR EACH ROW EXECUTE FUNCTION update_updated_at();
CREATE TRIGGER positions_updated_at BEFORE UPDATE ON positions FOR EACH ROW EXECUTE FUNCTION update_updated_at();
