mod config;
mod container;

use anyhow::{Context, Result};
use dayhelper_bot::{build_dispatcher, BotDeps};
use dayhelper_server_desktop_api::{
    build_router as build_desktop_router, ServerDesktopState,
};
use dayhelper_tma::{build_router, TmaState};
use teloxide::Bot;
use tower_http::services::{ServeDir, ServeFile};
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::config::Config;
use crate::container::Container;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();
    init_tracing();

    let cfg = Config::from_env().context("config")?;
    let pool = dayhelper_adapter_sqlite::connect(&cfg.database_url)
        .await
        .context("connecting to sqlite")?;
    dayhelper_adapter_sqlite::migrate(&pool)
        .await
        .context("running migrations")?;

    let bot = Bot::new(&cfg.bot_token);
    let container = Container::build(cfg, pool.clone(), bot.clone());

    // Shutdown channel: broadcast to all long-lived tasks.
    let (shutdown_tx, _) = tokio::sync::broadcast::channel::<()>(1);

    let scheduler = container.scheduler.clone();
    let scheduler_task = tokio::spawn({
        let scheduler = scheduler.clone();
        let shutdown_rx = shutdown_tx.subscribe();
        async move { scheduler.run(shutdown_rx).await }
    });

    let bot_task = tokio::spawn({
        let deps = BotDeps {
            ensure_user: container.ensure_user.clone(),
            create_reminder: container.create_reminder.clone(),
            list_reminders: container.list_reminders.clone(),
            cancel_reminder: container.cancel_reminder.clone(),
            issue_pair_code: container.issue_pair_code.clone(),
            update_timezone: container.update_timezone.clone(),
            update_nudge_settings: container.update_nudge_settings.clone(),
            scheduler: container.scheduler.handle(),
            default_timezone: container.config.default_timezone,
            tma_url: container.config.tma_public_url.clone(),
        };
        let bot = bot.clone();
        async move {
            let mut dispatcher = build_dispatcher(bot, deps);
            dispatcher.dispatch().await;
        }
    });

    let http_task = tokio::spawn({
        let tma_state = TmaState {
            bot_token: container.config.bot_token.clone().into(),
            default_timezone: container.config.default_timezone,
            ensure_user: container.ensure_user.clone(),
            create_reminder: container.create_reminder.clone(),
            list_reminders: container.list_reminders.clone(),
            cancel_reminder: container.cancel_reminder.clone(),
            update_timezone: container.update_timezone.clone(),
            update_nudge_settings: container.update_nudge_settings.clone(),
            reminder_repo: container.reminders.clone(),
            scheduler: container.scheduler.handle(),
        };
        let desktop_state = ServerDesktopState {
            redeem_pair_code: container.redeem_pair_code.clone(),
            accept_sync: container.accept_desktop_sync.clone(),
            tokens: container.desktop_tokens.clone(),
            users: container.users.clone(),
        };
        let bind_addr = container.config.bind_addr;
        let mut shutdown_rx = shutdown_tx.subscribe();
        async move {
            let tma_router = build_router(tma_state);
            let static_service = ServeDir::new("frontend/dist")
                .fallback(ServeFile::new("frontend/dist/index.html"));
            let app = tma_router
                .merge(build_desktop_router(desktop_state))
                .fallback_service(static_service);
            let listener = tokio::net::TcpListener::bind(bind_addr)
                .await
                .expect("bind http");
            info!(%bind_addr, "http listening (tma + desktop)");
            let shutdown_signal = async {
                let _ = shutdown_rx.recv().await;
            };
            axum::serve(listener, app)
                .with_graceful_shutdown(shutdown_signal)
                .await
                .expect("axum serve");
        }
    });

    // SQLite backup loop — only for file-backed databases.
    let backup_task = {
        let db_path = container.config.database_url
            .trim_start_matches("sqlite://")
            .to_string();
        if db_path != ":memory:" && !db_path.is_empty() {
            let backup = dayhelper_adapter_sqlite::backup::SqliteBackup::new(
                pool,
                db_path,
                std::time::Duration::from_secs(3600),
            );
            let shutdown_rx = shutdown_tx.subscribe();
            Some(tokio::spawn(async move { backup.run_loop(shutdown_rx).await }))
        } else {
            None
        }
    };

    // Wait for ctrl-c, then signal all tasks to drain.
    tokio::signal::ctrl_c().await?;
    info!("shutting down...");
    let _ = shutdown_tx.send(());

    let drain_deadline = tokio::time::sleep(std::time::Duration::from_secs(10));
    tokio::select! {
        _ = scheduler_task => { info!("scheduler drained"); }
        _ = http_task => { info!("http drained"); }
        _ = drain_deadline => { info!("shutdown timeout, forcing exit"); }
    }

    // Bot dispatcher has no clean shutdown API — just drop.
    bot_task.abort();

    if let Some(bt) = backup_task {
        let _ = bt.await;
    }

    Ok(())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}
