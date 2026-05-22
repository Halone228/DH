//! HTTP API consumed by the `dayhelper-cli` desktop daemon.
//!
//! Auth model is bearer-token (different from TMA's `initData`), so this
//! lives in its own crate with its own router. The composition root
//! merges this and the TMA router behind the same axum listener.

pub mod auth;
pub mod router;
pub mod state;

pub use router::build_router;
pub use state::ServerDesktopState;
