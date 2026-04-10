-- Adenora Seed Data
-- Run after migrations: psql $DATABASE_URL < scripts/seed.sql

-- =============================================================================
-- TEST USERS (required before markets — creator_id FK)
-- =============================================================================

INSERT INTO users (id, email, password_hash, display_name, country_code, kyc_status) VALUES
  ('a0000000-0000-0000-0000-000000000001', 'admin@adenora.app',
   '$argon2id$v=19$m=19456,t=2,p=1$SEED_SALT_NOT_FOR_PROD$placeholder_hash',
   'Adenora Admin', 'XK', 'verified'),
  ('a0000000-0000-0000-0000-000000000002', 'demo@adenora.app',
   '$argon2id$v=19$m=19456,t=2,p=1$SEED_SALT_NOT_FOR_PROD$placeholder_hash',
   'Demo User', 'AL', 'verified')
ON CONFLICT (email) DO NOTHING;

-- Seed wallets for test users
INSERT INTO wallets (user_id, currency, available) VALUES
  ('a0000000-0000-0000-0000-000000000001', 'EUR', 10000),
  ('a0000000-0000-0000-0000-000000000002', 'EUR', 5000)
ON CONFLICT (user_id, currency) DO NOTHING;

-- Seed a trivia game record (referenced by game_results)
INSERT INTO games (id, name, game_type, config) VALUES
  ('b0000000-0000-0000-0000-000000000001', 'Trivia', 'trivia', '{}')
ON CONFLICT DO NOTHING;

-- =============================================================================
-- CHARITY PROJECTS (GLC humanitarian projects)
-- =============================================================================

INSERT INTO charity_projects (id, name, description, category, country_codes, currency, is_active) VALUES
  (gen_random_uuid(), 'Roots of Peace', 'Landmine removal and sustainable agriculture — turning MINES to VINES in over 70 countries', 'landmine_removal', '{XK,AL,BA}', 'EUR', true),
  (gen_random_uuid(), 'H.E.L.P. Disaster Relief', 'Humanitarian Effort Lottery Project — strategically placed relief warehouses for rapid disaster response', 'disaster_relief', '{XK,AL,MK,RS,BA,ME,HR}', 'EUR', true),
  (gen_random_uuid(), 'H.E.L.P. Ukraine Rebuild', 'Lottery proceeds escrowed for Ukraine reconstruction — managed by GLC, no funds to government directly', 'infrastructure', '{XK,AL,MK}', 'EUR', true),
  (gen_random_uuid(), 'Balkan Education Fund', 'Scholarships and school infrastructure across Kosovo, Albania, and North Macedonia', 'education', '{XK,AL,MK}', 'EUR', true),
  (gen_random_uuid(), 'Clean Water Balkans', 'Water purification systems for rural communities in the Western Balkans', 'water_purification', '{XK,AL,MK,BA}', 'EUR', true),
  (gen_random_uuid(), 'Kosovo Youth Sports', 'Sports facilities and youth development programs across Kosovo', 'sports', '{XK}', 'EUR', true);

-- =============================================================================
-- SAMPLE PREDICTION MARKETS
-- =============================================================================

INSERT INTO markets (id, question, description, category, outcomes, status, creator_id, charity_project_id, country_codes, opens_at, closes_at) VALUES
  (gen_random_uuid(),
   'Will Albania begin EU accession negotiations before January 2027?',
   'Based on official EU Council decision. Must be formal opening of negotiations, not screening process.',
   'geopolitics', '["Yes","No"]'::jsonb, 'active',
   (SELECT id FROM users LIMIT 1),
   (SELECT id FROM charity_projects WHERE name = 'Balkan Education Fund'),
   '{XK,AL,MK,RS,BA,ME,HR,BG,GR,TR}',
   NOW(), NOW() + INTERVAL '6 months'),

  (gen_random_uuid(),
   'Will Kosovo join UEFA before 2028?',
   'Full membership, not observer status.',
   'sports', '["Yes","No"]'::jsonb, 'active',
   (SELECT id FROM users LIMIT 1),
   (SELECT id FROM charity_projects WHERE name = 'Kosovo Youth Sports'),
   '{XK,AL,MK,RS}',
   NOW(), NOW() + INTERVAL '18 months'),

  (gen_random_uuid(),
   'Will average EUR/ALL exchange rate exceed 120 in Q4 2026?',
   'Based on ECB reference rate, averaged over Oct-Dec 2026.',
   'economy', '["Yes","No"]'::jsonb, 'active',
   (SELECT id FROM users LIMIT 1),
   (SELECT id FROM charity_projects WHERE name = 'Clean Water Balkans'),
   '{XK,AL,MK}',
   NOW(), NOW() + INTERVAL '9 months'),

  (gen_random_uuid(),
   'Will North Macedonia hold early parliamentary elections before March 2027?',
   'Elections must be called and date set, not just speculation.',
   'geopolitics', '["Yes","No"]'::jsonb, 'active',
   (SELECT id FROM users LIMIT 1),
   (SELECT id FROM charity_projects WHERE name = 'Balkan Education Fund'),
   '{MK,XK,AL}',
   NOW(), NOW() + INTERVAL '12 months'),

  (gen_random_uuid(),
   'Will any Western Balkan country join the EU Schengen zone by end of 2027?',
   'Full Schengen membership with border control removal.',
   'geopolitics', '["Yes","No"]'::jsonb, 'active',
   (SELECT id FROM users LIMIT 1),
   (SELECT id FROM charity_projects WHERE name = 'Balkan Education Fund'),
   '{XK,AL,MK,RS,BA,ME,HR}',
   NOW(), NOW() + INTERVAL '20 months');

-- =============================================================================
-- SAMPLE LOTTERY GAMES ("Donate and Play")
-- =============================================================================

INSERT INTO lotteries (id, name, description, game_type, project_id, country_codes, ticket_price, currency, prize_pct, project_pct, company_pct, status) VALUES
  (gen_random_uuid(),
   'Kosovo Dream 6/49',
   'Classic 6-number draw. Pick your numbers and dream big while supporting education.',
   'draw',
   (SELECT id FROM charity_projects WHERE name = 'Balkan Education Fund'),
   '{XK,AL,MK}', 2.00, 'EUR', 50, 25, 25, 'active'),

  (gen_random_uuid(),
   'Balkan Scratch & Win',
   'Instant scratch cards with prizes up to 1000 EUR. Every card supports clean water.',
   'scratch',
   (SELECT id FROM charity_projects WHERE name = 'Clean Water Balkans'),
   '{XK,AL,MK,RS,BA,ME}', 1.00, 'EUR', 50, 25, 25, 'active'),

  (gen_random_uuid(),
   'H.E.L.P. 50/50',
   'Simple 50/50 raffle — half the pot goes to you, half to disaster relief.',
   'fifty_fifty',
   (SELECT id FROM charity_projects WHERE name = 'H.E.L.P. Disaster Relief'),
   '{XK,AL,MK,RS,BA,ME,HR}', 5.00, 'EUR', 50, 25, 25, 'active');

-- Create initial draws for lotteries
INSERT INTO draws (lottery_id, draw_number, scheduled_at, status)
SELECT id, 1, NOW() + INTERVAL '7 days', 'open'
FROM lotteries WHERE status = 'active';

INSERT INTO draws (lottery_id, draw_number, scheduled_at, status)
SELECT id, 2, NOW() + INTERVAL '14 days', 'scheduled'
FROM lotteries WHERE status = 'active';

-- =============================================================================
-- SAMPLE GAMES
-- =============================================================================

INSERT INTO games (name, description, game_type, min_players, max_players, is_free_to_play, entry_fee) VALUES
  ('Balkan Trivia', 'Test your knowledge of Balkan history, geography, and current events', 'trivia', 1, 1, true, NULL),
  ('Forecast Challenge', 'Predict probabilities for upcoming events. Scored by Brier score.', 'forecasting', 1, 100, true, NULL),
  ('Speed Predict', 'Rapid-fire yes/no predictions. How many can you get right in 60 seconds?', 'speed_predict', 1, 1, true, NULL),
  ('Pro Forecaster Tournament', 'Weekly forecasting competition with entry fee and prize pool', 'forecasting', 10, 100, false, 5.00);

-- =============================================================================
-- SAMPLE ACHIEVEMENTS
-- =============================================================================

INSERT INTO achievements (name, description, icon, criteria) VALUES
  ('First Steps', 'Make your first prediction', 'target', '{"FirstPrediction": null}'::jsonb),
  ('Winner', 'Win your first trade', 'trophy', '{"FirstWin": null}'::jsonb),
  ('Philanthropist', 'Buy your first lottery ticket', 'heart', '{"DonateAndPlay": null}'::jsonb),
  ('Regular', 'Play 10 games', 'gamepad', '{"GamesPlayed": 10}'::jsonb),
  ('Veteran', 'Play 100 games', 'star', '{"GamesPlayed": 100}'::jsonb),
  ('Hot Streak', '5 correct predictions in a row', 'flame', '{"PredictionStreak": 5}'::jsonb),
  ('Oracle', '10 correct predictions in a row', 'eye', '{"PredictionStreak": 10}'::jsonb),
  ('Champion', 'Win a tournament', 'crown', '{"TournamentsWon": 1}'::jsonb),
  ('High Scorer', 'Reach 10,000 total points', 'zap', '{"TotalScore": 10000}'::jsonb);
