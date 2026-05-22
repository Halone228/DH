//! Wayland tracker + idle detector for wlroots-family compositors
//! (niri, sway, hyprland, river, wayfire — anything that exposes
//! `zwlr_foreign_toplevel_management_v1` + `ext_idle_notifier_v1`).
//!
//! GNOME/KDE Plasma do not advertise these protocols and will hit
//! [`TrackerError::UnsupportedCompositor`] at startup. Adding GNOME/KDE
//! support is a *new adapter crate* (e.g. `desktop-adapter-gnome` over D-Bus),
//! not a patch here — keeps each adapter's blast radius small.

mod idle;
mod tracker;

pub use idle::WaylandIdleDetector;
pub use tracker::WaylandWindowTracker;
