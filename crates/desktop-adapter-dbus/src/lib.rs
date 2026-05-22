//! Desktop notifications via the freedesktop.org D-Bus interface
//! (`org.freedesktop.Notifications`). Works with any Linux DE that
//! implements the spec — KDE, GNOME, Sway/Mako, dunst, niri+mako, etc.

use async_trait::async_trait;
use dayhelper_desktop_ports::{DesktopNotifier, NotifyError};
use notify_rust::Notification;

pub struct DbusNotifier {
    app_name: String,
}

impl DbusNotifier {
    pub fn new(app_name: impl Into<String>) -> Self {
        Self {
            app_name: app_name.into(),
        }
    }
}

#[async_trait]
impl DesktopNotifier for DbusNotifier {
    async fn show(&self, title: &str, body: &str) -> Result<(), NotifyError> {
        // notify-rust is synchronous and short — run it in the blocking pool
        // so we don't park the tokio reactor on D-Bus round-trips.
        let app = self.app_name.clone();
        let title = title.to_string();
        let body = body.to_string();
        tokio::task::spawn_blocking(move || {
            Notification::new()
                .summary(&title)
                .body(&body)
                .appname(&app)
                .show()
                .map(|_| ())
        })
        .await
        .map_err(|e| NotifyError::Transport(Box::new(e)))?
        .map_err(|e| NotifyError::Transport(Box::new(e)))?;
        Ok(())
    }
}
