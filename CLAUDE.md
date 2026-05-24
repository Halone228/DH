# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

`dayhelper` — a Rust Telegram bot **plus a Linux desktop client**, sharing a single Cargo workspace. Two product surfaces, three responsibilities:

1. **Reminders.** One-shot, daily, weekly (any subset of weekdays), monthly. Surface: `/once`, `/daily`, `/list`, `/cancel` bot commands; TMA REST API + React frontend; also delivered to the desktop client as local notifications.
2. **Anti-procrastination nudges.** 5 randomized messages per day, per user, inside their active window (default 09:00–21:00 local). Independent of user-created reminders.
3. **Desktop activity tracking.** A Linux daemon (`dayhelper-cli daemon`) reports which app the user has focused, splits sessions on idle, and mirrors notifications via libnotify so the user gets nudges even with the phone away. Designed to work offline-tolerant: the daemon polls the server every minute and runs all already-known notifications from a local SQLite queue.

Project owner communicates in **Russian**. Code, identifiers, commits, and log lines stay in **English**. User-facing text (bot messages, nudge bank, desktop notification copy) is Russian and lives in localizable modules.

The TMA frontend is **active**. The backend skeleton in `crates/tma/` compiles and exposes REST endpoints. The frontend is a React SPA with Tailwind CSS served from the same Axum server via static file serving. See the TMA frontend in `frontend/`.

## Architecture

Hexagonal / Ports & Adapters, applied independently on the **server side** (`crates/{domain,ports,application,adapter-*,scheduler,bot,tma,app}`) and the **desktop side** (`crates/{desktop-domain,desktop-ports,desktop-application,desktop-adapter-*,desktop-app}`). The two sides are isolated — no domain types cross the boundary; they communicate only through the wire types in `crates/protocol/`.

Server-side dependency direction (always points inward toward `domain`):

```
        ┌──────────────── app (binary, composition root) ────────────────┐
        │                                                                │
        │   bot          tma           scheduler                         │
        │     \           |              /                               │
        │      \          |             /                                │
        │       └──────► application ◄─┘                                 │
        │                    │                                           │
        │                    ▼                                           │
        │                  ports ────────► domain                        │
        │                    ▲                                           │
        │      ┌─────────────┼──────────────┐                            │
        │      │             │              │                            │
        │   adapter-      adapter-       adapter-                        │
        │   sqlite        telegram       system                          │
        └────────────────────────────────────────────────────────────────┘
```

Desktop side mirrors the same shape:

```
       desktop-app (binary `dayhelper-cli`)
                  │
   ┌──────────────┼──────────────┐
   │              │              │
   v              v              v
 daemon ─► desktop-application ──┐
                  │              │
                  v              │
            desktop-ports ◄──────┘
                  ▲
   ┌──────┬───────┼───────┬───────┐
   │      │       │       │       │
adapter- adapter- adapter- adapter- adapter-
wayland  dbus     http    sqlite   (gnome…)
```

Rules:

- `domain` and `ports` have **zero** I/O dependencies. Domain has only chrono/uuid/serde; ports adds async-trait.
- `application` depends only on `domain` + `ports`. No knowledge of SQLite, teloxide, axum, or tokio's outside primitives. This is where use cases live.
- `adapter-*` crates implement port traits. Each adapter is its own crate so their dependency footprints stay independent — swapping SQLite for Postgres is one new crate, not a rewrite.
- `bot`, `tma`, and `scheduler` are runtime layers that depend on `application` (use cases) plus their respective transport (teloxide, axum, tokio).
- `app` is the **only** crate that names concrete adapters. It builds `Container` (see `crates/app/src/container.rs`) which constructs every `Arc<dyn Trait>` and hands them to the runtime layers. To swap an adapter, this is the only file that changes.

### Dependency injection

Pure constructor injection via `Arc<dyn Trait + Send + Sync>`. No service-locator, no macro framework (`shaku`/`dependency_inject`/`syrette`). The container in `crates/app/src/container.rs` builds every dependency once and passes them by `Arc::clone`. Tests can replace any port with an in-memory fake by constructing the use case directly with the desired implementation.

When adding a new use case:

1. Define ports it needs (or reuse existing ones in `crates/ports/`).
2. Add the struct in `crates/application/src/use_cases/`. Constructor takes `Arc<dyn Port>` for each dependency.
3. Re-export from `crates/application/src/lib.rs`.
4. Wire it in `Container::build`. Inject into whichever runtime crate calls it.

### Scheduler model

Persistent job queue in SQL (`scheduled_jobs` table). The scheduler loop:

1. drains all rows where `fire_at <= now() AND fired_at IS NULL` via atomic `UPDATE … RETURNING`;
2. peeks `MIN(fire_at)` to decide how long to sleep;
3. sleeps until then or until `SchedulerHandle::wakeup()` is called (bot/TMA call this on every mutation that may change the soonest event).

A second loop (hourly) plans nudges for every user with `enabled = 1`. `ScheduleDailyNudges` **is idempotent** — it counts already-pending nudges in today's window via `JobQueue::count_pending_nudges_in_window` and skips planning if any exist. So restarting the process or running the planner ten times an hour produces no extra nudges. Re-planning a day is an explicit action: `JobQueue::cancel_nudges_for_user` first, then call the use case.

A third loop (daily) runs `PruneOldData`, which deletes:
  - `scheduled_jobs` rows with `fired_at < now - 30d`,
  - `desktop_activity` rows with `received_at < now - 90d`.
Retention is configured via `PruneRetention` in the `Container::build` call site.

### Desktop client model

`dayhelper-cli` subcommands:

- `login <code>` — exchange a one-time pairing code (issued by the bot's `/pair` command) for a long-lived bearer token. Token is written to `~/.config/dayhelper/credentials.toml` with mode 0600.
- `logout` — wipe the credentials file.
- `status` — print pairing state.
- `daemon` — run the long-lived process: window tracker + idle detector + sync loop + fire loop.

The daemon's tasks:

- **WindowTracker** (`desktop-adapter-wayland`) listens to `zwlr_foreign_toplevel_management_v1`, emits one `FocusChange` per activation transition. Works on niri and any other wlroots-based compositor (sway, hyprland, river, wayfire). Will fail with `TrackerError::UnsupportedCompositor` on GNOME/KDE — adding GNOME/KDE is a *new adapter crate*, not a patch to this one.
- **IdleDetector** uses `ext_idle_notify_v1` with a configurable timeout (default 5 min).
- **SessionAggregator** turns the focus + idle streams into closed `ActivityEvent` rows. Sessions are split on focus change, idle, or process restart. Sessions shorter than 2 s are dropped (window flicker).
- **SyncWithServer** runs every 60 s: POSTs accumulated activity + ack'd notifications, receives the next window of pending notifications, persists them locally.
- **FireDueLocalNotifications** runs every 5 s: picks `pending` notifications whose `fire_at <= now`, fires them via D-Bus, marks `fired`. Notifications older than 15 minutes are marked `skipped` (don't show a 4 AM nudge at 9 AM).

### Server endpoints for the desktop client

Implemented in `crates/server-desktop-api/`. Auth is bearer-token (different from TMA's `initData`), so the crate is separate from `tma/`. Both routers are `Router::merge`'d behind the same listener in `app/src/main.rs`.

- `POST /api/desktop/pair` — validates the 6-digit pair-code, mints a 32-byte URL-safe-base64 token, stores its **SHA-256 hex** in `desktop_tokens`. Plaintext token returned once; never persisted.
- `POST /api/desktop/sync` — bearer-auth'd via `AuthedDesktop` extractor (hashes the bearer, looks up `desktop_tokens.token_hash`, then resolves the user). Returns jobs `fire_at <= now + 1h`. The client's `fired_notifications[]` ack is informational right now (a future `desktop_notification_deliveries` table can persist it for per-device stats).
- Pair-code lifecycle lives in `MemoryPairCodeStore` (`adapter-system`) — single-process, 5-min TTL, single-use. Multi-server deployment requires a Redis-backed adapter.

Bot command `/pair` (in `crates/bot/`) calls `IssuePairCode` and prints the code.

Wire types are frozen in `crates/protocol/`. Bump `PROTOCOL_VERSION` if the shape changes incompatibly.

### Cross-cutting concerns to remember

- **Timezones live on `User`.** Every recurrence calculation goes through `Recurrence::next_after(after, tz)`. Don't compute "next 9 AM" in UTC.
- **TMA auth boundary.** Every TMA endpoint that touches user data goes through the `AuthedUser` extractor in `crates/tma/src/auth.rs`, which validates Telegram's `initData` HMAC and inflates the `User`. Never trust an `id` field that arrived without going through that extractor.
- **DST.** `combine()` in `crates/domain/src/recurrence.rs` handles ambiguous and skipped wall-clock times. Don't build new datetime construction paths that bypass it.
- **Restart safety.** Jobs that should fire but didn't (because the process was down) are still in the queue with `fired_at IS NULL`. On restart, the scheduler picks them up and fires immediately — for nudges this is usually wrong (don't ping at 4am because you missed it), so a future enhancement should check `now - fire_at > threshold` and skip nudges in that case. Reminders fire late by design. **Desktop client already does this** (`STALE_AFTER = 15 min` in `fire_due.rs`).
- **Desktop ↔ server protocol stability.** `crates/protocol/` is the only crate both sides depend on. Treat its types like a published API — adding fields with sensible defaults is fine, removing or renaming requires a `PROTOCOL_VERSION` bump.

## Build & run

Toolchain isn't installed system-wide on NixOS — use the dev shell:

```bash
nix develop                                  # rust + sqlx-cli + sqlite
cp .env.example .env                         # fill in TELOXIDE_TOKEN
cargo build
cargo run -p dayhelper-app                   # server: bot dispatcher + scheduler + TMA HTTP
cargo run -p dayhelper-desktop-app -- daemon # desktop daemon (after `login`)
cargo test --workspace
cargo test -p dayhelper-domain recurrence::tests::daily_advances
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
```

The desktop daemon needs `DAYHELPER_SERVER_URL` set (or `--server-url`) and a saved credentials file. The server-side `/api/desktop/*` endpoints are not yet implemented, so right now `daemon` will fail at the first sync tick with a 404 — the rest (Wayland tracking, idle, local notification firing from manually-inserted DB rows) works in isolation.

Database migrations live in `crates/adapter-sqlite/migrations/` and are applied on startup by `dayhelper_adapter_sqlite::migrate`. There is no separate `sqlx migrate run` step in the normal flow; `sqlx-cli` is in the dev shell only for ad-hoc inspection.

Required env vars (see `.env.example`):

- `TELOXIDE_TOKEN` — bot token from @BotFather.
- `DATABASE_URL` — `sqlite://dayhelper.db` for dev. Migrations create the file on first run.
- `TMA_PUBLIC_URL` — public HTTPS origin of the Mini App (Telegram requires HTTPS).
- `TMA_BIND_ADDR` — `0.0.0.0:8080` by default.
- `DEFAULT_TIMEZONE` — IANA name, e.g. `Europe/Moscow`. Used when a brand-new user is created before they pick their own.

## Conventions

- Communication with the project owner: Russian. Code/identifiers/log lines/commits: English.
- User-facing strings (bot replies, nudge bank) live in modules dedicated to localization (`crates/application/src/messages.rs` for nudges; bot replies still inline — fold them into a similar module when a second locale is added).
- Don't add new I/O traits in `ports` unless an `application` use case actually needs them. The pattern is "use case appears, port appears, adapter appears" — not "speculative trait now, maybe-implement later."
