# Adenora

**Win. Grow. Give.** - Where Value Flows.

Human-first event markets with advanced execution and public-benefit funding.
Built for Global Lottery Consultants (GLC) + GAMLOT.

## Architecture

Rust workspace - 10 crates, 1 binary (`adenora`), ~8K lines Rust + 500 lines SQL + 1.6K lines frontend.

| Crate | Purpose |
|---|---|
| `adenora-common` | Types, errors, config, parabolic fees, i18n (21 locales), currency (12 currencies) |
| `adenora-gateway` | axum HTTP/WS server, 36+ API routes, 5 background services, middleware |
| `adenora-orderbook` | Dual-mode CLOB: batch auction 500ms (human) + continuous (ultimate/API) |
| `adenora-markets` | Market lifecycle (Proposed->Settled), proposals, content policy, geofencing |
| `adenora-oracle` | Multi-source resolution, evidence logging, free disputes, 11-person jury |
| `adenora-lottery` | "Donate and Play": draws, scratch cards, RNG, linked jackpots, free entries |
| `adenora-gaming` | Trivia (time-bonus scoring), forecasting (Brier), tournaments, achievements |
| `adenora-botarena` | Bot registry, cage matches, spectator WebSocket feed, bot tournaments |
| `adenora-charity` | GLC fund tracking, transparent ledger, revenue splits from all sources |
| `adenora-users` | Auth (JWT+argon2), KYC (Veriff), wallets (multi-currency), gambling limits |

## Market Modes

- **Human Markets**: 500ms batch auctions - default retail-facing experience
- **Ultimate Markets**: continuous matching on eligible markets - humans opt in, API bot access lives here
- **Bot Arena**: bots-only cage matches + tournaments; separate from the main market pitch
- Same underlying event, same oracle resolution, separate order books when advanced mode is enabled

## Fee Model

Kalshi parabolic: `taker = ceil(0.07 * C * P * (1-P))`, `maker = ceil(0.0175 * C * P * (1-P))`.
Max ~1.75c taker / ~0.44c maker at 50/50. Symmetric - approaches zero at extremes.
Split: 40% ops / 40% charity / 20% market creator. Deposits and withdrawals free.

## Background Services

| Service | Interval | Purpose |
|---|---|---|
| Batch auction | 500ms | Execute people-mode order matching + persist trades |
| Settlement | 10s | Auto-close expired markets, settle resolved markets past dispute window |
| Draw executor | 30s | Execute lottery draws, determine winners, distribute prizes |
| Leaderboard | 5min | Snapshot overall, bot, and gaming leaderboards |
| Recovery | 60s | Expire orphan orders, release stuck reserves, cleanup stale engines |

## Languages

Launch: Albanian (sq), Macedonian (mk), Serbian (sr), English (en).
System supports 21 locales for GLC's 30+ country portfolio.

## Currencies

EUR, ALL, MKD, RSD, BAM, BGN, RON, HUF, TRY, USD, USDC.

## Commands

```bash
# Development
cargo build                              # build all crates
cargo test --workspace                   # run 97 tests
cargo run --bin adenora                   # start server (needs DATABASE_URL)
cd web && npm run dev                    # start frontend dev server
powershell -ExecutionPolicy Bypass -File scripts/autoresearch-orderbook.ps1 -Label smoke

# Docker (local)
docker-compose -f deploy/docker-compose.yml up

# Docker (production)
docker-compose -f deploy/docker-compose.prod.yml up -d

# Database
psql $DATABASE_URL < scripts/seed.sql    # seed sample data

# Deploy
./scripts/deploy.sh <server-ip>          # deploy to Hetzner
./scripts/backup.sh                      # backup database
```

## API Routes (36+)

| Group | Endpoints |
|---|---|
| Health | `GET /`, `GET /healthz`, `GET /api/v1/status` |
| Auth | `POST register`, `POST login` |
| Markets | `GET list`, `GET {id}`, `GET {id}/book`, `POST propose` |
| Trading | `POST orders`, `POST orders/{id}/cancel`, `GET positions` |
| Lottery | `GET list`, `POST {id}/tickets`, `GET {id}/draws` |
| Gaming | `GET games`, `GET tournaments`, `GET leaderboard`, `POST trivia/start`, `POST trivia/answer`, `POST forecast/submit` |
| Bots | `GET list`, `POST register`, `GET {id}`, `GET leaderboard`, `GET tournaments` |
| Charity | `GET projects`, `GET ledger`, `GET summary` |
| Wallet | `GET wallet`, `POST deposit`, `POST withdraw` |
| WebSocket | `GET /ws` (subscribe, snapshot, book_update, ping/pong) |

## Autoresearch

- Repo playbook: `AUTORESEARCH.md`
- Agent prompt file: `autoresearch/program.md`
- Current active lane: `adenora-orderbook` execution research only
- Current scorer: `cargo run -q -p adenora-orderbook --bin research_score`
- Do not run autonomous loops on gateway, wallet, charity, KYC, auth, or migrations

## Project Structure

```text
adenora/
|- CLAUDE.md
|- Cargo.toml                    # Workspace
|- crates/                       # 10 Rust crates
|- web/                          # SvelteKit PWA (12 pages, 4 locales)
|- migrations/001_initial.sql    # 30+ PostgreSQL tables
|- config/                       # default.toml, production.toml
|- deploy/                       # Dockerfile, docker-compose (dev+prod), Caddyfile
|- scripts/                      # seed.sql, deploy.sh, backup.sh, autoresearch-orderbook.ps1
|- autoresearch/                 # agent program files for bounded research loops
|- .github/workflows/ci.yml      # CI: test + build + docker
`- .env.example                  # Production env template
```

## Safety & Compliance

- Responsible gambling: deposit limits (daily/weekly/monthly), self-exclusion, cooling-off, loss alerts
- Content policy: death/suffering markets auto-blocked
- Geofencing: per-market/lottery country restrictions, blocked country list
- Rate limiting: token bucket per IP (60 req/min people mode)
- Input validation: XSS sanitization, email/password strength, country validation
- KYC: Veriff integration interface, age 21+ verification
- Free disputes: no bond required, 11-person community jury
- Error recovery: orphan order cleanup, stuck reserve release

## Brand

- Logo: Three swooshes (teal/gold/crimson) radiating from golden sphere
- Colors: Navy #1B2B4B, Teal #2CA6C4, Gold #D4A629, Crimson #C42B3B
- Tagline: "Where Value Flows"
- CTA: "Win. Grow. Give."
- Teal = Gaming (Win), Gold = Prediction (Grow), Crimson = Public Benefit (Give)

## Private

GLC company overview PDF is confidential - do not commit or reference contents publicly.
