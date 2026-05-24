# Progress

## Status
In Progress

## Tasks

### Phase 1 — Foundation (completed)
- [x] setMyCommands — register bot commands with Telegram on startup
- [x] Nudge bank — expanded from 7 to 50+ messages
- [x] CI/CD — GitHub Actions workflow for Rust + frontend
- [x] Docker — multi-stage Dockerfile + .dockerignore

### Phase 2 — Core Reliability (completed)
- [x] Graceful shutdown — broadcast channel + drain with 10s timeout
- [x] SQLite backup — WAL checkpoint + file copy every hour

### Phase 3 — Observability & UX (completed)
- [x] Structured logging — JSON format behind RUST_LOG_FORMAT=json, command spans, scheduler spans
- [x] Bot error UX — friendly Russian error messages
- [x] Interactive /cancel — inline keyboard buttons

### Phase 4 — Production Hardening (in progress)
- [ ] Rate limiting — per-user on TMA and desktop API
- [x] Frontend polish — error boundary, offline detection, API retry

### Phase 5 — Tests (not started)
- [ ] Use case fakes + unit tests

## Files Changed

### This session
- `Cargo.toml` (workspace) — added `json` feature to tracing-subscriber
- `crates/app/src/main.rs` — `init_tracing()` supports `RUST_LOG_FORMAT=json`, added `Arc` import
- `crates/bot/src/lib.rs` — `#[instrument(skip(bot, deps, msg))]` on `handle_command`, records `user_id` and `cmd`
- `crates/scheduler/src/lib.rs` — `#[instrument(skip_all)]` on `fire_loop`, `nudge_planner_loop`, `prune_loop`
- `crates/server-desktop-api/src/router.rs` — fixed MSRV (LazyLock → OnceLock), fixed rate limiter init
- `crates/server-desktop-api/Cargo.toml` — removed unused `tokio` dep

## Notes
- All validation passes: `cargo build`, `cargo clippy -D warnings`, `cargo test --workspace`
- Scheduler spans use `#[instrument(skip_all)]` — safe for async (`Send` compatible)
- Bot span records `user_id` and `cmd` dynamically after EnsureUser resolution
- Fixed pre-existing build breakages from parallel workers (Arc import, MSRV, OnceLock)
