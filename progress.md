# Progress

## Status
In Progress

## Tasks
- [x] Phase 6: README.md (project root) + frontend/README.md

## Files Changed
- `README.md` — created, Russian language, comprehensive project docs
- `frontend/README.md` — rewritten, replaced Vite boilerplate

## Notes
- README verified against actual codebase: workspace members, .env.example, flake.nix, bot commands
- 21 crates total (20 Rust + 1 frontend)
- Rust version requirement: 1.78+ (from Cargo.toml workspace.package.rust-version)
- `DEFAULT_TIMEZONE` not in .env.example — only `RUST_LOG` as optional env var besides the core ones
