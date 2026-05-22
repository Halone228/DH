//! Telegram-flavoured implementations of the outbound ports.
//!
//! Today only [`TelegramNotifier`] lives here. If we add inline-keyboard
//! senders or media uploads later, they slot in alongside it.

mod notifier;

pub use notifier::TelegramNotifier;
