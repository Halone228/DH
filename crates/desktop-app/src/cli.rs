use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "dayhelper-cli", version, about = "dayhelper desktop client")]
pub struct Cli {
    /// Server base URL (e.g. https://dayhelper.example.com).
    /// Read from DAYHELPER_SERVER_URL env var if not provided.
    #[arg(long, env = "DAYHELPER_SERVER_URL")]
    pub server_url: Option<String>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Pair this device with the bot. Get the code by sending /pair to the bot.
    Login {
        code: String,
        /// Free-form label, e.g. "halone-laptop".
        #[arg(long, default_value = "linux-desktop")]
        label: String,
    },
    /// Drop saved credentials.
    Logout,
    /// Print pairing/sync status.
    Status,
    /// Run the long-lived background process: tracker + idle + sync + fire loops.
    Daemon {
        /// Sync interval in seconds. Server expects ~60s.
        #[arg(long, default_value_t = 60)]
        sync_interval: u64,
        /// Idle threshold in seconds.
        #[arg(long, default_value_t = 300)]
        idle_after: u64,
    },
}
