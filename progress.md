# Progress

## Status
In Progress

## Tasks
- [x] **setMyCommands**: Register bot commands with Telegram on startup (`bot/src/lib.rs::setup_commands()`)
- [x] **Nudge Bank**: Expanded `MESSAGES_RU` from 7 to 50 messages across 5 categories
- [x] CI/CD: GitHub Actions workflow (`.github/workflows/ci.yml`)
- [x] Docker: Dockerfile + .dockerignore
- [ ] Graceful Shutdown + SQLite Backup (Worker D — in progress, backup.rs needs tokio dep)
- [ ] Structured Logging
- [ ] Bot Error UX
- [ ] Interactive /cancel
- [ ] Rate Limiting
- [ ] Frontend Polish
- [ ] Tests

## Files Changed (this worker)
- `crates/bot/src/lib.rs` — added `setup_commands()` function
- `crates/app/src/main.rs` — call `setup_commands` before spawning bot
- `crates/application/src/messages.rs` — expanded from 7 to 50 nudge messages

## Validation
- `cargo build -p dayhelper-application -p dayhelper-bot` ✅
- `cargo test -p dayhelper-application -p dayhelper-domain` ✅ (3 tests pass)

## Notes
- sqlx uses runtime `query()` / `query_as()`, NOT compile-time `query!` macro
- Desktop Wayland/DBUS crates need system libs for compilation
- Docker build only produces `dayhelper-app` server binary
