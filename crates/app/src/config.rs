use std::net::SocketAddr;

use anyhow::{Context, Result};
use chrono_tz::Tz;

pub struct Config {
    pub bot_token: String,
    pub database_url: String,
    pub tma_public_url: String,
    pub bind_addr: SocketAddr,
    pub default_timezone: Tz,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let bot_token = std::env::var("TELOXIDE_TOKEN")
            .context("TELOXIDE_TOKEN missing — see .env.example")?;
        let database_url =
            std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://dayhelper.db".to_string());
        let tma_public_url = std::env::var("TMA_PUBLIC_URL")
            .unwrap_or_else(|_| "https://example.invalid/".to_string());
        let bind_addr_raw =
            std::env::var("TMA_BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
        let bind_addr: SocketAddr = bind_addr_raw
            .parse()
            .with_context(|| format!("invalid TMA_BIND_ADDR `{bind_addr_raw}`"))?;

        let tz_raw = std::env::var("DEFAULT_TIMEZONE").unwrap_or_else(|_| "Europe/Moscow".into());
        let default_timezone: Tz = tz_raw
            .parse()
            .with_context(|| format!("invalid DEFAULT_TIMEZONE `{tz_raw}`"))?;

        Ok(Self {
            bot_token,
            database_url,
            tma_public_url,
            bind_addr,
            default_timezone,
        })
    }
}
