# FluxRadar

A self-hosted dashboard and Telegram alerting service for [Flux](https://runonflux.io) node operators.

Paste a public wallet address, and FluxRadar shows the full fleet of nodes tied to it —
their tier, status, uptime, rank in the payout queue, estimated earnings, and hosted apps.
Link the same wallet to the [`@FluxRadarBot`](https://t.me/FluxRadarBot) Telegram bot and you get
a push the moment a node drops offline or changes status.

**Live instance:** [fluxradar.ru](https://fluxradar.ru)

## Why this exists

Running Flux nodes means watching for two things: *is my node still earning?* and *did it just go down?*
The official explorer answers the first only shallowly and the second not at all. FluxRadar fills both gaps:

- **Wallet-first, no accounts.** A Flux wallet address is already a public identifier. You don't sign up,
  you don't hand over credentials — you paste an address and see its fleet. Linking Telegram alerts is done
  *through the bot* (`/link <wallet>`), so the website never needs a login system at all.
- **One dashboard for a whole fleet.** Operators with dozens or hundreds of nodes can't eyeball each one.
  FluxRadar aggregates the fleet: total earnings (day/month/year), APR against collateral value,
  Parallel Assets breakdown, per-node maintenance windows, and time-to-next-payout.
- **Alerts you actually receive.** A background worker polls the network on a schedule and notifies you
  via Telegram when something changes — no dashboard tab needs to stay open.

## Why a backend proxy (and not call Flux API from the browser)

The browser never talks to the Flux API directly. Every request goes through FluxRadar's own REST API.
That indirection is deliberate:

- **Caching.** An operator with 100+ nodes would hammer the upstream API on every page load.
  Results are cached in Redis, so repeated views are cheap and fast.
- **One request, not N.** The fleet is resolved from a single network-wide node list and filtered locally,
  instead of one upstream call per node.
- **CORS & rate-limit safety.** The proxy sidesteps browser CORS limits and shields the upstream from abuse.
- **Secrets stay server-side.** The Telegram bot token and any credentials never reach the client.

```
Browser ──▶ FluxRadar API ──▶ Redis cache ──▶ Flux API (api.runonflux.io)
                  ▲
Worker ──polls──┘ ──compares snapshots──▶ Telegram bot ──▶ alerts
```

## Architecture

A Cargo workspace of three binaries and three libraries, split so each binary can scale independently:

| Crate | Kind | Responsibility |
|---|---|---|
| `crates/api` | binary | REST proxy in front of the Flux API + business logic (Axum) |
| `crates/worker` | binary | Scheduled polling of the Flux API + alert computation (Tokio) |
| `crates/bot` | binary | Telegram bot: commands + sending alerts (teloxide) |
| `crates/flux-client` | lib | Typed client for the Flux API |
| `crates/domain` | lib | Models and formulas (earnings, tier, uptime); shared types |
| `crates/storage` | lib | PostgreSQL repositories (sqlx) + Redis cache (bb8) |

**Data stores.** PostgreSQL 16 holds Telegram↔wallet links, node-status snapshots, and alert history.
Redis 7 handles the Flux-API cache, rate-limiting, and alert deduplication. The website has no access to
the subscriptions table — only the worker and bot do.

**Frontend.** React + Vite + TypeScript + Tailwind, served as static files by Nginx, with i18n (en/ru/zh).

## Running locally (development)

Requires a Rust toolchain (cargo/rustc), Docker, and [sqlx-cli](https://crates.io/crates/sqlx-cli).

```bash
# 1. Start PostgreSQL + Redis
cp .env.example .env          # fill in TELEGRAM_BOT_TOKEN
docker compose -f infra/docker-compose.yml up -d

# 2. Apply migrations
sqlx migrate run

# 3. Run the services (separate terminals)
cargo run -p api
cargo run -p worker
cargo run -p bot
```

All environment variables are documented in [`.env.example`](.env.example).

## Running locally (production mode)

The full vertical slice: Nginx serves the built frontend and proxies `/api` to the release `api` binary,
which talks to PostgreSQL, Redis, and the Flux API.

```bash
./scripts/prod-up.sh      # PG+Redis → web/dist → release api(:5049) → Nginx(:8080)
# dashboard: http://localhost:8080
./scripts/prod-down.sh    # stop Nginx + api (leave PG/Redis running)
./scripts/prod-down.sh --all   # stop everything (data volumes are preserved)
```

Production-mode configs: [`infra/docker-compose.prod.yml`](infra/docker-compose.prod.yml),
[`infra/nginx.prod.conf`](infra/nginx.prod.conf).

## Development commands

```bash
cargo build                    # build the workspace
cargo test                     # run tests (includes golden earnings tests)
cargo fmt --check              # formatting check (CI)
cargo clippy -- -D warnings    # lint with zero warnings
```

The earnings formulas are domain logic that must not break silently, so they are frozen by golden tests
on known values.

## Security

- **Public addresses only.** FluxRadar never asks for private keys or seed phrases.
- Wallet addresses are validated by format *before* any upstream call — invalid input is rejected with a
  400 and never leaves the server.
- The Telegram bot token lives only in the server's `.env` (git-ignored); the repo ships a placeholder
  `.env.example`.

## Support

FluxRadar is free and self-hosted. If it helps you, a donation in FLUX keeps development going:

```
t1TfYPwJdZvKu6yHccqfJxLAsCKoZ9nfevr
```

## License

[MIT](LICENSE). The code is written from scratch; the original Fluxnode project was used only as a
factual reference (which Flux endpoints exist, what the earnings formulas mean), never as source to copy.
