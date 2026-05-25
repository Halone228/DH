//! Locale enum for the desktop client.
//!
//! Currently only English is shipped; the enum exists so a future locale
//! switch can be wired in without touching domain types.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Locale {
    #[default]
    En,
    Ru,
}
