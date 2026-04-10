-- =============================================================================
-- Migration 002: Cause Campaigns, Direct Donations, Purpose Lotteries
-- "Where Value Flows" — Adenora impact layer
-- =============================================================================

-- ─── Cause Campaigns ─────────────────────────────────────────────────────────
-- Standalone fundraising campaigns with a goal, deadline, and live progress.
-- Examples: rebuild a desal plant, restore a school, landmine clearance.

CREATE TABLE cause_campaigns (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug            TEXT NOT NULL UNIQUE,           -- URL-friendly id, e.g. "kosovo-desal-2026"
    title           TEXT NOT NULL,
    tagline         TEXT NOT NULL DEFAULT '',       -- one-liner for cards
    description     TEXT NOT NULL DEFAULT '',       -- full rich description
    category        TEXT NOT NULL DEFAULT 'infrastructure',
    -- infrastructure, water, education, healthcare, disaster_relief,
    -- clean_energy, landmine_removal, housing, food_security, other

    -- Funding goal
    goal_amount     NUMERIC NOT NULL,
    currency        TEXT NOT NULL DEFAULT 'EUR',
    amount_raised   NUMERIC NOT NULL DEFAULT 0,    -- maintained by trigger/app
    donor_count     INTEGER NOT NULL DEFAULT 0,

    -- Media
    hero_image_url  TEXT,
    logo_url        TEXT,
    impact_metric   TEXT,                          -- e.g. "families served", "hectares cleared"
    impact_value    TEXT,                          -- e.g. "12,000" or "450"
    location        TEXT,                          -- e.g. "Mitrovica, Kosovo"
    country_codes   TEXT[] NOT NULL DEFAULT '{}',

    -- Partners
    partner_org     TEXT,                          -- e.g. "H.E.L.P.", "Roots of Peace"
    partner_url     TEXT,

    -- Lifecycle
    status          TEXT NOT NULL DEFAULT 'active',
    -- draft, active, funded, completed, paused
    starts_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ends_at         TIMESTAMPTZ,                   -- NULL = no deadline
    funded_at       TIMESTAMPTZ,                   -- set when goal reached
    completed_at    TIMESTAMPTZ,                   -- set when work verified done

    -- Transparency
    update_count    INTEGER NOT NULL DEFAULT 0,    -- number of progress updates posted
    is_featured     BOOLEAN NOT NULL DEFAULT FALSE,
    allow_anonymous BOOLEAN NOT NULL DEFAULT TRUE,

    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_campaigns_status   ON cause_campaigns(status);
CREATE INDEX idx_campaigns_featured ON cause_campaigns(is_featured) WHERE is_featured = TRUE;
CREATE INDEX idx_campaigns_category ON cause_campaigns(category);

-- ─── Campaign Progress Updates ────────────────────────────────────────────────
-- Field dispatches, milestone posts, proof of work. Public feed per campaign.

CREATE TABLE campaign_updates (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    campaign_id     UUID NOT NULL REFERENCES cause_campaigns(id) ON DELETE CASCADE,
    title           TEXT NOT NULL,
    body            TEXT NOT NULL,
    image_url       TEXT,
    milestone_pct   INTEGER,   -- e.g. 25, 50, 100 — optional milestone marker
    author          TEXT NOT NULL DEFAULT 'Adenora Team',
    published_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_campaign_updates_campaign ON campaign_updates(campaign_id);

-- ─── Direct Donations ────────────────────────────────────────────────────────
-- One-click wallet → campaign. No lottery involved.

CREATE TABLE direct_donations (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    campaign_id     UUID NOT NULL REFERENCES cause_campaigns(id),
    user_id         UUID REFERENCES users(id),     -- NULL if anonymous
    amount          NUMERIC NOT NULL,
    currency        TEXT NOT NULL DEFAULT 'EUR',
    message         TEXT,                          -- optional donor message
    is_anonymous    BOOLEAN NOT NULL DEFAULT FALSE,
    display_name    TEXT,                          -- shown on donor wall if not anon
    status          TEXT NOT NULL DEFAULT 'completed',
    -- pending, completed, refunded
    ledger_entry_id UUID REFERENCES charity_ledger(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_donations_campaign ON direct_donations(campaign_id);
CREATE INDEX idx_donations_user     ON direct_donations(user_id);

-- ─── Purpose Lotteries ───────────────────────────────────────────────────────
-- A lottery tied 100% (or near-100%) to a specific campaign.
-- "Buy a ticket for €5 — fund Kosovo's water plant, win €50,000"

CREATE TABLE purpose_lotteries (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    campaign_id     UUID NOT NULL REFERENCES cause_campaigns(id),
    lottery_id      UUID NOT NULL REFERENCES lotteries(id),

    -- Override revenue split for this purpose lottery
    campaign_pct    INTEGER NOT NULL DEFAULT 70,   -- % to campaign (higher than standard)
    prize_pct       INTEGER NOT NULL DEFAULT 25,   -- % to prize pool
    company_pct     INTEGER NOT NULL DEFAULT 5,    -- minimal ops cut

    -- Linked jackpot option: ticket buyers enter both the prize draw AND fund the cause
    jackpot_label   TEXT NOT NULL DEFAULT 'Campaign Jackpot',
    campaign_target NUMERIC,                       -- mirrors campaign goal_amount

    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(campaign_id, lottery_id)
);

-- ─── Donor Wall ──────────────────────────────────────────────────────────────
-- Public recognition for top donors (opt-in). Displayed on campaign page.

CREATE TABLE donor_wall (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    campaign_id     UUID NOT NULL REFERENCES cause_campaigns(id),
    user_id         UUID REFERENCES users(id),
    display_name    TEXT NOT NULL,
    total_given     NUMERIC NOT NULL DEFAULT 0,
    currency        TEXT NOT NULL DEFAULT 'EUR',
    message         TEXT,
    badge           TEXT,    -- "Founder", "Champion", "Supporter"
    is_visible      BOOLEAN NOT NULL DEFAULT TRUE,
    last_donated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(campaign_id, user_id)
);

CREATE INDEX idx_donor_wall_campaign ON donor_wall(campaign_id);

-- ─── Trigger: keep amount_raised in sync ─────────────────────────────────────

CREATE OR REPLACE FUNCTION update_campaign_raised()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE cause_campaigns
    SET
        amount_raised = (
            SELECT COALESCE(SUM(amount), 0)
            FROM direct_donations
            WHERE campaign_id = NEW.campaign_id
              AND status = 'completed'
        ),
        donor_count = (
            SELECT COUNT(*)
            FROM direct_donations
            WHERE campaign_id = NEW.campaign_id
              AND status = 'completed'
        ),
        funded_at = CASE
            WHEN (
                SELECT COALESCE(SUM(amount), 0)
                FROM direct_donations
                WHERE campaign_id = NEW.campaign_id AND status = 'completed'
            ) >= goal_amount AND funded_at IS NULL THEN NOW()
            ELSE funded_at
        END,
        updated_at = NOW()
    WHERE id = NEW.campaign_id;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_update_campaign_raised
    AFTER INSERT OR UPDATE ON direct_donations
    FOR EACH ROW EXECUTE FUNCTION update_campaign_raised();

-- ─── Seed: example campaigns ─────────────────────────────────────────────────

INSERT INTO cause_campaigns
    (slug, title, tagline, description, category, goal_amount, currency,
     impact_metric, impact_value, location, country_codes, partner_org, is_featured)
VALUES
(
    'kosovo-desal-2026',
    'Kosovo Clean Water Initiative',
    'Fund a desalination plant for 12,000 families in northern Kosovo',
    'Northern Kosovo has faced chronic water shortages since 2018. This campaign funds the construction of a solar-powered desalination and filtration plant in Mitrovica, providing clean drinking water to 12,000 families year-round. Built with H.E.L.P. and local engineers.',
    'water',
    250000.00, 'EUR',
    'families served', '12,000',
    'Mitrovica, Kosovo',
    ARRAY['XK'],
    'H.E.L.P.', TRUE
),
(
    'tirana-school-rebuild',
    'Tirana School Rebuild',
    'Rebuild earthquake-damaged classrooms for 800 students in Tirana',
    'The 2024 aftershock damaged three primary school buildings in the Kombinat district of Tirana. This campaign funds full structural rebuilding and modernisation, restoring safe education for 800 students.',
    'education',
    120000.00, 'EUR',
    'students served', '800',
    'Tirana, Albania',
    ARRAY['AL'],
    'Balkan Education Fund', TRUE
),
(
    'ohrid-landmine-clearance',
    'Ohrid Region Landmine Clearance',
    'Clear 450 hectares of farmland for safe use in North Macedonia',
    'Legacy landmines from the 2001 conflict continue to restrict 450 hectares of agricultural land in the Ohrid region. Partnering with Roots of Peace, this campaign funds professional clearance teams to make the land safe and productive again.',
    'landmine_removal',
    180000.00, 'EUR',
    'hectares cleared', '450',
    'Ohrid Region, North Macedonia',
    ARRAY['MK'],
    'Roots of Peace', TRUE
),
(
    'balkans-solar-schools',
    'Solar Schools Balkans',
    'Install solar panels on 30 schools across Kosovo, Albania and N. Macedonia',
    'Energy poverty forces many schools to operate without reliable electricity. This campaign installs solar panel arrays on 30 schools across all three GLC-licensed territories, cutting costs and reducing CO₂ emissions by an estimated 420 tonnes per year.',
    'clean_energy',
    95000.00, 'EUR',
    'schools powered', '30',
    'Western Balkans',
    ARRAY['XK','AL','MK'],
    'GLC Foundation', FALSE
);
