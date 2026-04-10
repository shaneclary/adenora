# Fix Guide

## Goal
Create a proposal, product story, and implementation plan that match what Adenora can credibly claim today while defining the next fixes needed before partner and investor rollout.

## Core Positioning
- Lead with `licensed event markets with transparent charity and project funding`.
- Keep event markets as the primary product.
- Treat gaming, lotteries, and bot execution as supporting layers, not equal top-level businesses.
- Do not pitch this as "better than Polymarket/Kalshi". Pitch it as more regulated, more locally deployable, and more accountable.

## Product Architecture To Use In The Proposal

### Human Markets
- Default web and mobile experience.
- 500ms batch auctions.
- Retail-first language: fair pricing, easier onboarding, responsible controls.

### Bot Mode
- API-only access for eligible markets.
- This is not a separate marketplace.
- This is an execution mode on the same underlying event markets.
- Public UI should educate users about it, but not center it.
- Best external language: `API access for advanced traders and market makers`.

### Ultimate Markets
- Use this only if it means one eligible market can support both human and bot participation.
- Publicly, this should still look like one market with an execution toggle or access layer.
- It should not read like a third disconnected product.
- If `Ultimate` feels too vague in partner conversations, consider `Unified` or `Pro` as the external label.

## Decision To Lock In
- Bot access is a toggle on betting/event markets, not a whole marketplace.
- Human mode is the default public experience.
- Bot mode is API-first, education-supported, and advanced.
- Ultimate markets are just the deepest version of the same market model, not a new vertical.

## Proposal Language To Use Now
Use a line close to this:

`Adenora is a licensed event-market platform built for human-first trading, optional API-based bot execution, and transparent funding flows to approved charity and project campaigns.`

Short version:

`Human-first event markets with advanced API execution and public-benefit funding built into platform economics.`

## Messaging Changes

### What To Lead With
- Regulated event markets
- Country-aware deployment and licensing strategy
- Transparent fee routing to projects and charity campaigns
- Human-first market design
- Advanced API access for professional participants

### What To Downplay
- Bot arena style wording
- Too many equal product pillars on the homepage
- Broad claims about global availability without jurisdiction detail
- General gaming as a co-equal headline next to the core market product

### Rename Or Reframe
- `Bot Battle Mode` -> `API Mode`, `Bot Access`, or `Advanced Execution`
- `Prediction markets, gaming, and public benefit` -> `Event markets with transparent project funding`
- `GLC Licensed` -> specific entity and jurisdiction language only

## Claims To Stop Making Until Fixed
- `GLC Licensed` without naming the entity, jurisdiction, and scope of the license
- `Veriff KYC` as if live end-to-end
- `30+ Countries` unless that is truly operational and legally supported
- `Every cent is tracked and auditable` until donation and ledger accounting are fully consistent
- Live market probability, volume, or trader counts when the UI is using mock or derived data

## Engineering Fixes

### P0: Must Fix Before Serious External Use
- Fix the donation wallet debit path so it uses the real wallet fields.
- Make charity ledger writes mandatory for donations and keep project totals in sync.
- Remove silent fallbacks that can report success without a valid ledger record.
- Enforce KYC, geofencing, and responsible gambling checks at route level for trading and lottery flows.
- Make market proposal submission use authenticated users and the actual proposal/content-policy validators.
- Add route-level tests for donation, trading, lottery, and proposal submission.

### P1: Product And UX Fixes
- Present human/bot/ultimate as market access modes, not separate marketplaces.
- Keep the default `/markets` experience human-first.
- Move bot education into API docs, advanced tooltips, or a developer page.
- Add a clear market-page toggle for eligible markets instead of mode-heavy homepage messaging.
- Label demo or mock data clearly in the UI.

### P2: Proposal And Operations Hardening
- Add jurisdiction-by-jurisdiction licensing notes for proposal use.
- Add partner verification workflow for campaigns and project operators.
- Add proof-of-impact updates and campaign completion evidence.
- Add a developer/API section for bot participants instead of broad consumer marketing.

## Homepage And Deck Fixes

### Homepage
- Lead with one idea: event markets that fund real projects.
- Keep charity visible as the differentiator, not a side feature.
- Reduce the visual emphasis on bots for first-time users.
- If bot mode is shown, frame it as advanced API access on selected markets.

### Pitch Deck
- Slide 1: Licensed event markets with transparent project funding
- Slide 2: Why the market exists now
- Slide 3: Human-first market structure
- Slide 4: Advanced API bot access on eligible markets
- Slide 5: Charity and project-funding flywheel
- Slide 6: Licensing and rollout by jurisdiction
- Slide 7: Compliance and trust
- Slide 8: Revenue model

## Demo Rules
- If data is mock, say it is mock.
- If a compliance feature is scaffolded but not enforced, say `integration in progress` instead of implying production readiness.
- If a license exists, describe exactly what activity it permits.
- Do not imply that one country's approval automatically carries into another.

## Recommended External Narrative
`Adenora is a human-first event market platform that can deploy by jurisdiction, route a defined share of platform economics to approved campaigns, and support advanced API execution on eligible markets without turning the product into a bot-only venue.`

## Internal Narrative
`One marketplace. Three access layers.`

- Human: retail UI, fair execution
- Bot: API-only advanced participation
- Ultimate: eligible markets where both layers operate together

## Immediate Next Steps
1. Update proposal and deck language to the positioning above.
2. Remove separate-marketplace bot language from the website copy.
3. Fix donation accounting and ledger consistency first.
4. Wire real enforcement into trading and lottery routes.
5. Add an advanced/API narrative instead of consumer bot marketing.
