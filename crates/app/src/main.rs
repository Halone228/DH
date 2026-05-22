mod config;
mod container;

use anyhow::{Context, Result};
use dayhelper_bot::{build_dispatcher, BotDeps};
use dayhelper_server_desktop_api::{
    build_router as build_desktop_router, ServerDesktopState,
};
use dayhelper_tma::{build_router, TmaState};
use teloxide::Bot;
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
    let container = Container::build(cfg, pool, bot.clone());

    let scheduler = container.scheduler.clone();
    let scheduler_task = tokio::spawn({
        let scheduler = scheduler.clone();
        async move { scheduler.run().await }
    });

    let bot_task = tokio::spawn({
        let deps = BotDeps {
            ensure_user: container.ensure_user.clone(),
            create_reminder: container.create_reminder.clone(),
            list_reminders: container.list_reminders.clone(),
            cancel_reminder: container.cancel_reminder.clone(),
            issue_pair_code: container.issue_pair_code.clone(),
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
            scheduler: container.scheduler.handle(),
        };
        let desktop_state = ServerDesktopState {
            redeem_pair_code: container.redeem_pair_code.clone(),
            accept_sync: container.accept_desktop_sync.clone(),
            tokens: container.desktop_tokens.clone(),
            users: container.users.clone(),
        };
        let bind_addr = container.config.bind_addr;
        async move {
            let app = build_router(tma_state).merge(build_desktop_router(desktop_state));
            let listener = tokio::net::TcpListener::bind(bind_addr)
                .await
                .expect("bind http");
            info!(%bind_addr, "http listening (tma + desktop)");
            axum::serve(listener, app).await.expect("axum serve");
        }
    });

    tokio::select! {
        _ = scheduler_task => info!("scheduler exited"),
        _ = bot_task => info!("bot exited"),
        _ = http_task => info!("http exited"),
        _ = tokio::signal::ctrl_c() => info!("ctrl-c"),
    }

    Ok(())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}
