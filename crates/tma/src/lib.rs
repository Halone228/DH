//! HTTP API consumed by the Telegram Mini App. Auth is via Telegram's
//! signed `initData`: every request goes through [`auth`] which validates
//! the HMAC and inflates the requesting user.

pub mod auth;
pub mod rate_limit;
pub mod router;
pub mod state;

pub use router::build_router;
pub use state::TmaState;
