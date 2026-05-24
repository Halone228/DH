# Progress

## Status
In Progress

## Tasks
- [x] CI/CD: GitHub Actions workflow (`.github/workflows/ci.yml`)
  - Rust job: fmt + clippy + test with `DATABASE_URL=sqlite::memory:`
  - Frontend job: npm ci + build
  - Installs system deps: libsqlite3-dev, libwayland-dev, libdbus-1-dev
- [x] Docker: Dockerfile + .dockerignore
  - 3-stage build: Rust builder → Node frontend → Debian slim runtime
  - Installs libwayland-dev + libdbus-1-dev in builder for desktop crate compilation
  - Caches dependencies via dummy source + `cargo build` before copying real source
  - Runtime only includes ca-certificates + libsqlite3-0
  - Exposes port 8080

## Files Changed
- `.github/workflows/ci.yml` (created)
- `Dockerfile` (created)
- `.dockerignore` (created)

## Notes
- sqlx uses runtime `query()` / `query_as()`, NOT compile-time `query!` macro, so no `.sqlx/` offline directory needed
- `DATABASE_URL=sqlite::memory:` is set as workflow-level env var for sqlx compile-time checks
- Desktop Wayland/DBUS crates need system libs installed for compilation (in both CI and Docker)
- Docker build only produces `dayhelper-app` server binary; desktop client is a native Wayland app
