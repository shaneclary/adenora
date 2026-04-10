# AUTORESEARCH

Karpathy-style autoresearch works here if it is constrained hard enough.

The goal is not "let an agent rewrite Adenora". The goal is:

- one small editable surface
- one short repeatable evaluation loop
- hard safety gates before anything is kept
- explicit no-go zones for regulated or accounting-critical code

## Active Lane

The first active lane is `ultimate-market execution research`.

Scope:

- improve or clarify the local execution path for continuous matching
- preserve current Human vs Ultimate semantics
- keep all work local, testable, and reviewable

Primary files for this lane:

- `crates/adenora-orderbook/src/continuous.rs`
- `crates/adenora-orderbook/src/matching.rs`
- `crates/adenora-orderbook/src/tests.rs`

Agent instructions for this lane live in:

- `autoresearch/program.md`

Evaluation entry point for this lane:

- `scripts/autoresearch-orderbook.ps1`

## Safe Zones

Green zone for autonomous local experimentation:

- `crates/adenora-orderbook/src/continuous.rs`
- `crates/adenora-orderbook/src/matching.rs`
- `crates/adenora-orderbook/src/tests.rs`
- documentation under `AUTORESEARCH.md`, `autoresearch/`, and `c2c.md`

Yellow zone for human-reviewed follow-up only:

- `crates/adenora-orderbook/src/engine.rs`
- `web/src/routes/(app)/unlimited/+page.svelte`
- `web/src/routes/(app)/bots/+page.svelte`

## Forbidden Zones

Do not use autoresearch loops on these paths:

- `crates/adenora-gateway/src/routes/trading.rs`
- `crates/adenora-gateway/src/routes/campaigns.rs`
- `crates/adenora-gateway/src/routes/charity.rs`
- `crates/adenora-gateway/src/routes/wallet.rs`
- `crates/adenora-gateway/src/routes/kyc.rs`
- `crates/adenora-gateway/src/middleware/`
- `crates/adenora-users/`
- `migrations/`

Reason:

- ledger integrity
- compliance and geofencing
- wallet/accounting correctness
- auth and permissions
- schema compatibility

These areas can accept ideas from research, but not autonomous edits.

## Scorecard

Adenora should use gates first and metrics second.

Hard gates:

1. `cargo test -p adenora-orderbook`
2. `cargo test -p adenora-common fees`
3. No changes to API payload shape, database schema, or market mode semantics
4. No change that weakens self-trade prevention, IOC/FOK handling, or mode separation

Soft signals:

- smaller, more legible diffs
- fewer branches or duplicated paths
- better local comments where logic is subtle
- stronger deterministic test coverage
- improved `RESEARCH_SCORE` on the fixed continuous-execution workload

If a change fails a hard gate, discard it.

## Loop

1. Read `autoresearch/program.md`.
2. Make one small change in the active lane.
3. Run `powershell -ExecutionPolicy Bypass -File scripts/autoresearch-orderbook.ps1 -Label <short-name>`.
4. Check the reported `RESEARCH_SCORE` and keep the change only if all hard gates pass.
5. Record accepted changes in `c2c.md`.

## Review Rule

Passing the loop is necessary, not sufficient.

Anything kept from an autoresearch cycle still requires human review before merge because this repo touches regulated markets, funds routing, and user balances.
