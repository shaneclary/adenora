-- Community Votes — participatory budgeting with conviction voting
-- "We don't just predict the future, we create it."

-- Vote campaigns: a sponsor creates a decision with proposals
CREATE TABLE IF NOT EXISTS vote_campaigns (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    creator_id      UUID NOT NULL REFERENCES users(id),
    title           TEXT NOT NULL,
    description     TEXT NOT NULL DEFAULT '',
    category        TEXT NOT NULL DEFAULT 'community',

    -- Vote mode: 'one_person' | 'capped' | 'uncapped'
    vote_mode       TEXT NOT NULL DEFAULT 'capped'
                    CHECK (vote_mode IN ('one_person', 'capped', 'uncapped')),
    -- Max votes per user (only applies to 'capped' mode)
    vote_cap        INT NOT NULL DEFAULT 100,

    -- Fund allocation for losing proposals
    -- 'winner_take_all' | 'winner_impact' | 'winner_pool'
    fund_mode       TEXT NOT NULL DEFAULT 'winner_impact'
                    CHECK (fund_mode IN ('winner_take_all', 'winner_impact', 'winner_pool')),
    -- Split percentages (must sum to 100)
    winner_pct      INT NOT NULL DEFAULT 60 CHECK (winner_pct >= 0 AND winner_pct <= 100),
    charity_pct     INT NOT NULL DEFAULT 20 CHECK (charity_pct >= 0 AND charity_pct <= 100),
    pool_pct        INT NOT NULL DEFAULT 20 CHECK (pool_pct >= 0 AND pool_pct <= 100),

    -- Seed funding from sponsor
    seed_amount     NUMERIC NOT NULL DEFAULT 0,
    currency        TEXT NOT NULL DEFAULT 'EUR',

    -- Linked charity project for charity_pct
    charity_project_id UUID REFERENCES charity_projects(id),

    -- Community pool for pool_pct (links to a reusable fund)
    community_pool_id UUID,  -- self-referencing or external fund

    -- Lifecycle
    status          TEXT NOT NULL DEFAULT 'open'
                    CHECK (status IN ('draft', 'open', 'voting', 'closed', 'resolved')),
    opens_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    closes_at       TIMESTAMPTZ NOT NULL,
    resolved_at     TIMESTAMPTZ,
    winning_proposal_id UUID,

    -- Metadata
    country_codes   TEXT[] NOT NULL DEFAULT '{}',
    hero_image_url  TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT valid_split CHECK (winner_pct + charity_pct + pool_pct = 100)
);

-- Proposals within a vote campaign
CREATE TABLE IF NOT EXISTS vote_proposals (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    campaign_id     UUID NOT NULL REFERENCES vote_campaigns(id) ON DELETE CASCADE,
    title           TEXT NOT NULL,
    description     TEXT NOT NULL DEFAULT '',
    image_url       TEXT,
    proposer_id     UUID REFERENCES users(id),

    -- Tallies (updated by trigger or application)
    vote_count      INT NOT NULL DEFAULT 0,
    fund_total      NUMERIC NOT NULL DEFAULT 0,
    voter_count     INT NOT NULL DEFAULT 0,

    sort_order      INT NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Individual votes: one row per user per campaign (can update proposal choice)
CREATE TABLE IF NOT EXISTS community_votes (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    campaign_id     UUID NOT NULL REFERENCES vote_campaigns(id) ON DELETE CASCADE,
    proposal_id     UUID NOT NULL REFERENCES vote_proposals(id) ON DELETE CASCADE,
    user_id         UUID NOT NULL REFERENCES users(id),

    -- Money put in
    amount_eur      NUMERIC NOT NULL CHECK (amount_eur > 0),
    -- Actual votes counted (capped by vote_mode)
    votes_counted   INT NOT NULL DEFAULT 1,

    -- Optional public comment
    comment         TEXT NOT NULL DEFAULT '',
    is_public       BOOLEAN NOT NULL DEFAULT false,

    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- One vote record per user per campaign (they pick one proposal)
    UNIQUE(campaign_id, user_id)
);

-- Community pools — reusable funds that accumulate across vote campaigns
CREATE TABLE IF NOT EXISTS community_pools (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            TEXT NOT NULL,
    description     TEXT NOT NULL DEFAULT '',
    balance         NUMERIC NOT NULL DEFAULT 0,
    currency        TEXT NOT NULL DEFAULT 'EUR',
    country_codes   TEXT[] NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Update the FK now that community_pools exists
ALTER TABLE vote_campaigns
    ADD CONSTRAINT fk_community_pool
    FOREIGN KEY (community_pool_id) REFERENCES community_pools(id);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_vote_campaigns_status ON vote_campaigns(status, closes_at);
CREATE INDEX IF NOT EXISTS idx_vote_proposals_campaign ON vote_proposals(campaign_id, vote_count DESC);
CREATE INDEX IF NOT EXISTS idx_community_votes_campaign ON community_votes(campaign_id, proposal_id);
CREATE INDEX IF NOT EXISTS idx_community_votes_user ON community_votes(user_id);

-- Trigger: update proposal tallies when votes are cast
CREATE OR REPLACE FUNCTION update_proposal_tallies()
RETURNS TRIGGER AS $$
BEGIN
    -- Recalculate for the affected proposal
    UPDATE vote_proposals SET
        vote_count = COALESCE((SELECT SUM(votes_counted) FROM community_votes WHERE proposal_id = NEW.proposal_id), 0),
        fund_total = COALESCE((SELECT SUM(amount_eur) FROM community_votes WHERE proposal_id = NEW.proposal_id), 0),
        voter_count = COALESCE((SELECT COUNT(*) FROM community_votes WHERE proposal_id = NEW.proposal_id), 0)
    WHERE id = NEW.proposal_id;

    -- If updating (changing proposal), also recalculate the old proposal
    IF TG_OP = 'UPDATE' AND OLD.proposal_id != NEW.proposal_id THEN
        UPDATE vote_proposals SET
            vote_count = COALESCE((SELECT SUM(votes_counted) FROM community_votes WHERE proposal_id = OLD.proposal_id), 0),
            fund_total = COALESCE((SELECT SUM(amount_eur) FROM community_votes WHERE proposal_id = OLD.proposal_id), 0),
            voter_count = COALESCE((SELECT COUNT(*) FROM community_votes WHERE proposal_id = OLD.proposal_id), 0)
        WHERE id = OLD.proposal_id;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_update_proposal_tallies
    AFTER INSERT OR UPDATE ON community_votes
    FOR EACH ROW EXECUTE FUNCTION update_proposal_tallies();

-- Seed: sample community pool and vote campaign
INSERT INTO community_pools (id, name, description, country_codes) VALUES
    ('c0000000-0000-0000-0000-000000000001',
     'Pristina Community Fund',
     'Reusable participatory budgeting fund for Pristina municipal projects',
     '{XK}')
ON CONFLICT DO NOTHING;
