mod cli;
mod container;
mod daemon;
mod paths;

use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::Parser;
use dayhelper_desktop_adapter_sqlite as sqlite;
use tracing_subscriber::EnvFilter;

use crate::cli::{Cli, Command};
use crate::container::DesktopContainer;
use crate::daemon::DaemonOptions;
use crate::paths::DesktopPaths;

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let cli = Cli::parse();
    let paths = DesktopPaths::discover()?;
    let server_url = resolve_server_url(&cli)?;

    let pool = sqlite::connect(&paths.db_url()?).await.context("sqlite connect")?;
    sqlite::migrate(&pool).await.context("desktop migrations")?;

    let idle_after = match &cli.command {
        Command::Daemon { idle_after, .. } => Duration::from_secs(*idle_after),
        _ => Duration::from_secs(300),
    };

    let container = Arc::new(DesktopContainer::build(
        pool,
        &paths,
        server_url.clone(),
        idle_after,
    )?);

    match cli.command {
        Command::Login { code, label } => login(container, code, label, server_url).await,
        Command::Logout => logout(container).await,
        Command::Status => status(container).await,
        Command::Daemon { sync_interval, .. } => {
            daemon::run(
                container,
                DaemonOptions {
                    sync_interval: Duration::from_secs(sync_interval),
                },
            )
            .await
        }
    }
}

async fn login(
    container: Arc<DesktopContainer>,
    code: String,
    label: String,
    server_url: String,
) -> Result<()> {
    let creds = container
        .pair
        .execute(code, label, server_url)
        .await
        .context("pair")?;
    println!("paired as user {}", creds.user_id);
    Ok(())
}

async fn logout(container: Arc<DesktopContainer>) -> Result<()> {
    container.credentials.clear().await?;
    println!("credentials cleared");
    Ok(())
}

async fn status(container: Arc<DesktopContainer>) -> Result<()> {
    match container.credentials.load().await? {
        Some(creds) => {
            println!("paired: yes");
            println!("user_id: {}", creds.user_id);
            println!("server: {}", creds.server_url);
            println!("paired_at: {}", creds.paired_at);
        }
        None => println!("paired: no — run `dayhelper-cli login <code>`"),
    }
    Ok(())
}

fn resolve_server_url(cli: &Cli) -> Result<String> {
    cli.server_url
        .clone()
        .ok_or_else(|| anyhow::anyhow!("set --server-url or DAYHELPER_SERVER_URL"))
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();
}
