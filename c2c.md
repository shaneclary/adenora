# c2c

Purpose: change-to-change handoff for future AI systems and engineers working in this repo.

Date: 2026-04-09

Scope: reposition the product from a separate bot/unlimited marketplace story to one event-market system with:
- Human Markets as the default surface
- Ultimate Markets as the opt-in advanced surface
- API bot access as a layer inside Ultimate Markets, not a standalone marketplace

Related context:
- `FIX_GUIDE.md` contains the broader messaging and implementation direction used for this change set.

## Files Changed

Path: `web/src/routes/+page.svelte`
Change: rewrote homepage copy around human-first event markets, ultimate markets, and project funding.
Details: removed risky trust-bar claims like `GLC Licensed`, `Veriff KYC`, and `30+ Countries`; changed the launch-market section from live framing to illustrative framing; replaced the games/bot-tournament pillar with an Ultimate Markets pillar.

Path: `web/src/routes/(app)/markets/+page.svelte`
Change: reframed the main markets page as the Human Markets default experience.
Details: renamed the mode badges to `Human Markets` and `Ultimate Markets`; removed the old blocked-bot/second-market wording; added a preview-analytics note so derived volume/trader/probability numbers are not presented as live telemetry.

Path: `web/src/routes/(app)/unlimited/+page.svelte`
Change: repurposed the `/unlimited` route into the Ultimate Markets page.
Details: updated the page language to describe continuous matching on eligible markets with API bot access; fixed broken frontend behavior by switching from `api.getOrderBook` to `api.getOrderbook`; changed market title rendering to use `question`; normalized order book rendering to the advanced book shape returned by the API; changed order submission mode to `ultimate`; added a preview-analytics note; added explicit `for`/`id` label bindings on the order form fields to remove local accessibility warnings on this page.

Path: `web/src/routes/+layout.svelte`
Change: updated shared navigation and footer language.
Details: relabeled `/unlimited` as `Ultimate`; changed footer platform links from `Markets (People)` and `Unlimited Market` to `Human Markets` and `Ultimate Markets`; replaced the hard licensing footer claim with softer country-eligibility wording.

Path: `web/src/app.html`
Change: updated the global HTML description metadata.
Details: replaced the broad brand-only description with `human-first event markets with transparent project funding`.

Path: `web/static/manifest.json`
Change: updated the PWA description.
Details: aligned the manifest metadata with the new event-market positioning.

Path: `crates/adenora-gateway/src/routes/trading.rs`
Change: aligned backend trading mode parsing with the new UI naming.
Details: the route now accepts `people`/`human` for the default book and `unlimited`/`ultimate`/`bot` for the continuous book; response mode names now map to `human` or `ultimate`; persisted DB mode remains `people` or `unlimited` for compatibility with existing logic such as order cancellation.

Path: `CLAUDE.md`
Change: updated the repository summary and market-mode documentation.
Details: replaced the older gaming-heavy summary with human-first event-market language; renamed the orderbook description and Market Modes section to match Human/Ultimate terminology.

Path: `AUTORESEARCH.md`
Change: added the repo-level Karpathy-style autoresearch playbook.
Details: defines the active research lane, allowed files, forbidden files, hard gates, and the local workflow for bounded AI experimentation.

Path: `autoresearch/program.md`
Change: added the agent instruction file for bounded orderbook research.
Details: constrains future AI systems to small local loops on the orderbook execution layer and blocks autonomous edits in gateway, wallet, charity, auth, KYC, and migration paths.

Path: `scripts/autoresearch-orderbook.ps1`
Change: added a repeatable evaluation runner for the active autoresearch lane.
Details: runs the orderbook and fee gate commands plus a deterministic research scorer, writes a markdown report to `target/autoresearch`, and exits nonzero on failure.

Path: `crates/adenora-orderbook/src/bin/research_score.rs`
Change: added a deterministic continuous-execution research scorer for the autoresearch lane.
Details: replays a fixed Ultimate Markets order flow against `ContinuousEngine`, emits a `RESEARCH_SCORE`, and prints supporting metrics such as total quantity, final spread, and resting depth.

Path: `crates/adenora-orderbook/src/continuous.rs`
Change: tightened continuous-mode order handling for the bounded research lane.
Details: added a `projected_fill` pre-check so `FOK` orders in Ultimate Markets are rejected unless the full requested quantity is available; simplified `IOC` remainder cleanup by cancelling the resting remainder directly instead of mutating an unused local copy.

Path: `crates/adenora-orderbook/src/matching.rs`
Change: removed unused local variables in the matching path.
Details: dropped stale `Order` import and unused `bid_id` / `ask_id` bindings so the bounded lane builds with less warning noise.

Path: `crates/adenora-orderbook/src/batch.rs`
Change: removed unused imports in the batch engine.
Details: dropped unused `chrono` and `tokio::time` imports that were creating warning noise during the autoresearch gates.

Path: `crates/adenora-orderbook/src/book.rs`
Change: removed an unused import in the orderbook model.
Details: dropped unused `uuid::Uuid` import so the orderbook gate output is cleaner.

Path: `crates/adenora-orderbook/src/tests.rs`
Change: added new continuous-mode edge-case tests.
Details: added `test_continuous_ioc_partial_fill_cancels_remainder` and `test_continuous_fok_rejects_partial_fill` to lock in remainder cleanup and all-or-none behavior for the bounded Ultimate Markets lane.

Path: `FIX_GUIDE.md`
Change: added the fix guide used to drive this repositioning pass.
Details: documents the product framing, proposal-safe messaging, and the engineering priorities behind the Human Markets / Ultimate Markets / API bot access model.

Path: `c2c.md`
Change: added this handoff log.
Details: this file should be updated on future change sets that alter product positioning, routing semantics, or compatibility behavior.

## Behavior Changes

- The `/unlimited` route path is unchanged for compatibility, but the user-facing label is now `Ultimate`.
- The advanced market page now uses `mode: 'ultimate'` from the frontend.
- The backend accepts `ultimate` as an alias and maps it to the existing continuous/unlimited book.
- Human-vs-ultimate naming is now consistent across homepage, nav, markets page, advanced page, and internal repo docs.
- The homepage and market pages now explicitly avoid presenting sample or derived frontend analytics as live production data.
- Continuous `FOK` orders in the bounded orderbook lane now behave as true all-or-none orders: if full quantity is not available, they do not partially execute.

## Follow-Up Not Yet Done

- ~~Donation accounting and charity-ledger consistency are still a separate backend fix and were not changed in this pass.~~ **Fixed in deep pass (2026-04-09).**
- ~~KYC/geofence/responsible-gambling enforcement is still broader than the copy now claims, but not fully wired end-to-end.~~ **KYC and self-exclusion enforcement added in deep pass (2026-04-09). Geofencing still not enforced at route level.**
- The route path `/unlimited` is still a compatibility path; if you later rename it to `/ultimate`, update nav, links, and any external references together.
- The first autoresearch lane is in place, but no autonomous optimization changes have been accepted into the orderbook yet. This pass adds the deterministic scorer; it does not change matching behavior.
- The first accepted orderbook research cycle was a cleanup/stability pass. It kept `RESEARCH_SCORE` flat at `2191`; the next cycle can start optimizing against that baseline.
- Geofencing (country restrictions) is not yet enforced at trading or lottery route level — schema has `country_codes` on markets and lotteries but no middleware checks user country against them.
- Rate limiting applies per-IP but not per-user-id for authenticated users.
- `persist_trades` now logs errors but still does not wrap the full trade lifecycle in a DB transaction — a partial failure (e.g., position update succeeds, wallet credit fails) would leave inconsistent state. Wrapping in `BEGIN/COMMIT` with rollback on error is the next step.
- The position upsert formula only handles buys (quantity increase). Sells should decrease quantity, not increase it. This needs a separate code path for sell-side position tracking.

## Verification

- `npm.cmd run build` completed successfully in `web/` on 2026-04-09.
- `powershell -ExecutionPolicy Bypass -File scripts/autoresearch-orderbook.ps1 -Label score-baseline` completed successfully on 2026-04-09 and wrote `target/autoresearch/20260409-021028-score-baseline.md`.
- The first deterministic baseline for the bounded orderbook lane is `RESEARCH_SCORE=2191` with `TOTAL_TRADES=5`, `TOTAL_QUANTITY=23`, `FINAL_SPREAD=9`, `RESTING_BID_QTY=7`, and `RESTING_ASK_QTY=12`.
- `cargo test -p adenora-orderbook` completed successfully after the cleanup cycle on 2026-04-09 with `25` passing tests and no orderbook warning noise.
- `powershell -ExecutionPolicy Bypass -File scripts/autoresearch-orderbook.ps1 -Label cleanup-cycle` completed successfully on 2026-04-09 and wrote `target/autoresearch/20260409-022038-cleanup-cycle.md`.
- The cleanup cycle kept the deterministic score unchanged at `RESEARCH_SCORE=2191`, which is the expected outcome for a non-optimizing stability pass.
- Remaining build warnings are pre-existing outside this change set:
  - ~~accessibility warnings in `web/src/routes/(app)/charity/+page.svelte`~~ **Fixed in deep pass.**
  - ~~accessibility warnings in `web/src/routes/(app)/bots/+page.svelte`~~ **No longer present after page rewrite.**
  - placeholder `href="#"` links in `web/src/routes/+layout.svelte`
  - Svelte/SvelteKit dependency export warnings during build

---

# Deep Pass — 2026-04-09

Date: 2026-04-09

Scope: full-system audit and fix pass covering financial integrity, security enforcement, data consistency, and frontend correctness. Triggered by three-way concurrent audit (Rust backend, SvelteKit frontend, SQL schema) that identified 21 critical/high backend issues, 34 frontend issues, and 12 schema issues.

Related context:
- `FIX_GUIDE.md` P0 section drove the priority ordering.
- Audit was run against the state after the repositioning pass above.

## Files Changed

Path: `crates/adenora-gateway/src/routes/trading.rs`
Change: fixed wallet race condition, trade persistence, KYC enforcement, self-exclusion check.
Details:
- **Wallet race condition (CRITICAL):** replaced SELECT-then-UPDATE balance check with single atomic `UPDATE wallets SET available = available - $1 ... WHERE available >= $1 RETURNING available`. Double-spend via concurrent requests is no longer possible.
- **persist_trades error logging (HIGH):** replaced all 9 `.ok()` silent swallows with `if let Err(e) { tracing::error!(...) }` blocks. Trade insert failure now skips dependent operations via `continue`. Every DB failure is logged with trade_id, user_id, and error.
- **ON CONFLICT fix (CRITICAL):** `ON CONFLICT DO NOTHING` on trades INSERT changed to `ON CONFLICT (id) DO NOTHING` — the bare clause was a PostgreSQL ambiguity.
- **KYC + self-exclusion enforcement (CRITICAL):** added pre-trade check: queries `kyc_status` and `self_exclusion_until` from users table. Rejects orders unless `kyc_status` is `verified` or `approved`. Rejects if `self_exclusion_until > NOW()`. Mirrors the existing pattern in `lottery.rs`.

Path: `crates/adenora-gateway/src/routes/campaigns.rs`
Change: fixed donation wallet field, charity ledger integrity, KYC enforcement, ledger-donation linkage.
Details:
- **Wallet field fix (CRITICAL, done in prior pass):** `balance` column (doesn't exist) → `available` column. Already fixed before this pass.
- **Charity ledger integrity (CRITICAL, done in prior pass):** silent fallback with fake UUID → mandatory ledger write or hard 500 error. Already fixed before this pass.
- **KYC enforcement on donations (CRITICAL):** added KYC status check before accepting donations. Users with status other than `verified`/`approved` get a 403.
- **Ledger-donation linkage (HIGH):** charity ledger INSERT now uses `RETURNING id` to capture `ledger_entry_id`. The `direct_donations` INSERT now includes `ledger_entry_id = $8` binding — donations are traceable to their ledger entries.

Path: `crates/adenora-gateway/src/routes/kyc.rs`
Change: added HMAC-SHA256 signature verification to KYC webhook.
Details: webhook now reads raw `Bytes` body and `HeaderMap`. Computes `HMAC-SHA256(body, KYC_WEBHOOK_SECRET)` and compares against `X-HMAC-Signature` header. In production (when secret is not the default placeholder), unsigned requests are rejected with 401. Forged webhook attacks that could mark arbitrary users as KYC-verified are now blocked. Added `hmac`, `sha2`, `hex` crate dependencies.

Path: `crates/adenora-gateway/src/routes/markets.rs`
Change: added auth requirement and typed request struct to market proposal endpoint.
Details: `propose_market` now requires `AuthUser` extractor — uses `auth.user_id` instead of `Uuid::new_v4()`. Replaced `Json<Value>` with typed `ProposeMarketRequest` struct (question, description, category, outcomes, closes_at, country_codes). Added length validation on question (1-500 chars) and description (1-5000 chars). Added `tracing::info` on proposal submission.

Path: `crates/adenora-gateway/src/routes/gaming.rs`
Change: fixed hardcoded game_id foreign key violation.
Details: replaced `'00000000-0000-0000-0000-000000000001'` hardcoded UUID (which violated FK constraint to non-existent games row) with dynamic lookup: `SELECT id FROM games WHERE game_type = 'trivia' LIMIT 1`. Falls back to creating a trivia game record if none exists. Insert failure now logged instead of silently swallowed.

Path: `crates/adenora-gateway/src/services/draw_executor.rs`
Change: made lottery draw execution idempotent.
Details: the `UPDATE draws SET winning_numbers = ... status = 'drawn'` now includes a CAS guard: `WHERE ... AND winning_numbers IS NULL`. If another process already executed the draw, `rows_affected() == 0` triggers early return with a warning log instead of generating a second set of winning numbers.

Path: `crates/adenora-gateway/src/main.rs`
Change: replaced permissive CORS with configured origin allowlist.
Details: `CorsLayer::permissive()` replaced with conditional logic: if `ADENORA_CORS_ORIGINS` is `*`, remains permissive (dev mode); otherwise constructs `AllowOrigin::list()` from parsed origins. Allows GET/POST/PUT/DELETE/OPTIONS methods. Allows `Content-Type`, `Authorization`, and `X-Bot-Key` headers. Enables `allow_credentials`.

Path: `crates/adenora-common/src/config.rs`
Change: added KYC config section.
Details: new `KycConfig` struct with `webhook_secret` (from `KYC_WEBHOOK_SECRET` env var, defaults to `CHANGE_ME_IN_PRODUCTION`) and `provider` (from `KYC_PROVIDER`, defaults to `veriff`). Added to `AdenoraConfig`.

Path: `crates/adenora-gateway/Cargo.toml`
Change: added cryptographic dependencies for webhook verification.
Details: `hmac = "0.12"`, `sha2 = "0.10"`, `hex = "0.4"`.

Path: `scripts/seed.sql`
Change: added test users, wallets, and trivia game record before market seeds.
Details: two test users (`admin@adenora.app`, `demo@adenora.app`) with pre-set UUIDs, KYC status `verified`, and EUR wallets (10000, 5000). A trivia game record with fixed UUID. All use `ON CONFLICT DO NOTHING` for idempotency. This prevents FK violations when market INSERTs reference `(SELECT id FROM users LIMIT 1)`.

Path: `web/src/app.css`
Change: fixed missing CSS variables and syntax error.
Details: added `--pillar-win: var(--teal)`, `--pillar-grow: var(--gold)`, `--pillar-give: var(--crimson)` aliases used by games and lottery page dots. Fixed `--warning: --gold` (invalid — bare custom property name) to `--warning: var(--gold)`.

Path: `web/src/routes/(app)/charity/+page.svelte`
Change: fixed a11y warning on donate modal overlay.
Details: added `on:keydown` handler (Escape key closes modal) alongside `on:click|self` on the overlay div. Eliminates the Svelte a11y warning about non-interactive elements with click handlers.

Path: `web/src/routes/(app)/bots/+page.svelte`
Change: full page rewrite (done in mode-separation pass, before this deep pass).
Details: new Bot Arena page with spectator WebSocket feed, cage match panel, leaderboard with PnL/Sharpe/drawdown columns, bot registration form with API docs. Replaces the minimal leaderboard-only page.

Path: `web/src/routes/(app)/unlimited/+page.svelte`
Change: new Unlimited/Ultimate Markets page (done in mode-separation pass, refined by user).
Details: 3-column trading layout: market list, order book, order entry panel. Disclaimer acknowledgement gate. Continuous order submission with `mode: 'ultimate'`.

## Behavior Changes

- Trading now requires KYC verification (`verified` or `approved` status). Unverified users get 403 with their current KYC status in the response.
- Self-excluded users are blocked from trading (same check already existed for lottery).
- Wallet debits for trades are atomic — no window for double-spend between balance check and reservation.
- Market proposals require authentication — anonymous proposals no longer accepted.
- Market proposals are validated against a typed schema — arbitrary JSON no longer accepted.
- KYC webhook requires HMAC signature in production — forged verification attacks blocked.
- CORS is configurable via `ADENORA_CORS_ORIGINS` env var — defaults to permissive for dev, locked down with explicit origin list in production.
- Lottery draws are idempotent — concurrent execution attempts for the same draw are safely no-ops.
- Donations are linked to their charity ledger entries via `ledger_entry_id` — full audit trail from wallet debit to ledger to donation record.
- All trade persistence failures are logged with structured tracing fields instead of silently swallowed.
- Frontend CSS variables `--pillar-win`, `--pillar-grow`, `--pillar-give` now resolve correctly.
- `--warning` CSS variable now evaluates to gold instead of being a broken reference.

## Follow-Up Not Yet Done

- Geofencing: `country_codes` exists on markets and lotteries but is not checked against user's `country_code` at order/ticket submission time.
- Per-user rate limiting: rate limiter uses IP only. Authenticated users with valid JWT bypass IP limits. Should add user_id-based limits.
- Transaction wrapping for persist_trades: individual DB operations log errors but the full trade lifecycle is not wrapped in a PostgreSQL transaction. A partial failure (e.g., position update succeeds but wallet credit fails) leaves inconsistent state.
- Position tracking for sells: the position upsert formula only handles buy-side quantity increases. Sell-side should decrease quantity, not apply the average-price formula additively.
- Frontend error handling: most pages use empty `catch (e) {}` blocks. Users see blank screens on API failure. Should show user-friendly error messages.
- Frontend auth guards: protected pages (portfolio, wallet, account) do not redirect unauthenticated users to login.
- WebSocket reconnection: bots page spectator feed has no retry logic on disconnect.

## Verification

- `cargo build --workspace` completed with 0 errors, 42 warnings (all pre-existing unused imports/variables) on 2026-04-09.
- `npm run build` completed with 0 errors, 0 a11y warnings on 2026-04-09.
- Deployed to Hetzner VPS at `5.78.182.190`. Server health check returned `{"ok":true}` after restart.
