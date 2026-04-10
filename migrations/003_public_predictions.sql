-- Public predictions — "money where your mouth is" debate feed
-- Users opt-in to show their prediction + amount publicly on a market

CREATE TABLE IF NOT EXISTS public_predictions (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    market_id   UUID NOT NULL REFERENCES markets(id) ON DELETE CASCADE,
    order_id    UUID NOT NULL REFERENCES orders(id),
    side        TEXT NOT NULL CHECK (side IN ('yes', 'no')),
    amount_eur  NUMERIC NOT NULL CHECK (amount_eur > 0),
    comment     TEXT NOT NULL DEFAULT '',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_public_pred_market ON public_predictions(market_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_public_pred_user ON public_predictions(user_id);

-- Add performance indexes identified during audit
CREATE INDEX IF NOT EXISTS idx_draws_lottery_status ON draws(lottery_id, status, scheduled_at);
CREATE INDEX IF NOT EXISTS idx_charity_projects_active ON charity_projects(is_active) WHERE is_active = true;
