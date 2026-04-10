# Adenora Autoresearch Program

You are running short, local research loops on Adenora.

Your current mission is narrow:

- improve the Ultimate Markets execution layer only
- keep Human Markets semantics unchanged
- do not touch ledger, wallet, charity, auth, KYC, geofence, or migrations

## Editable Files

You may edit:

- `crates/adenora-orderbook/src/continuous.rs`
- `crates/adenora-orderbook/src/matching.rs`
- `crates/adenora-orderbook/src/tests.rs`
- `c2c.md` after an accepted change

Prefer the smallest possible diff.

## Non-Editable Files

Do not edit:

- anything in `crates/adenora-gateway/src/routes/`
- anything in `crates/adenora-users/`
- anything in `migrations/`
- anything in `crates/adenora-charity/`
- anything in `crates/adenora-common/src/fees.rs`

If you think one of those files must change, stop and hand the idea to a human.

## Success Criteria

Your change is acceptable only if all of the following remain true:

- `cargo test -p adenora-orderbook` passes
- `cargo test -p adenora-common fees` passes
- `cargo run -q -p adenora-orderbook --bin research_score` produces a valid `RESEARCH_SCORE`
- People/Human mode still maps to batch execution
- Unlimited/Ultimate mode still maps to continuous execution
- self-trade prevention still works
- IOC/FOK behavior still works
- dual-book separation still works

## Preferred Work

Good targets:

- removing local ambiguity in continuous execution
- tightening tests around edge cases
- clarifying comments where execution semantics are easy to misunderstand
- reducing accidental complexity without changing public behavior
- improving the deterministic `RESEARCH_SCORE` without weakening any hard gate

Bad targets:

- changing fees
- changing persistence
- changing API route payloads
- changing product copy
- changing compliance behavior

## Loop

1. Read the current orderbook code and tests.
2. Propose one small change.
3. Run:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/autoresearch-orderbook.ps1 -Label short-name
```

4. If the loop fails, revert your last change.
5. If the loop passes, summarize:
   - what changed
   - why it is better
   - what still needs human review
6. Append the accepted result to `c2c.md`.

## Stop Conditions

Stop immediately if:

- you need schema changes
- you need gateway or wallet changes
- you cannot explain the behavioral impact in one paragraph
- the diff stops being small and reviewable
