//! wars-server: lobbies, authoritative lockstep matches, saves, scenarios and players.

mod auth;
mod db;
mod flags;
mod game;
mod lobby;
mod routes;
mod ws;

use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub accounts_url: String,
    pub accounts_internal_url: String,
    pub guest_signing_key: String,
    pub public_url: String,
    pub version: String,
    /// exp (feature flags): evaluation API base and environment client key. An
    /// empty key means "no flags": the browser client is open to everyone.
    pub exp_url: String,
    pub exp_client_key: String,
    /// Boolean flag that opens the browser beta; `<flag>-testers` (json list of
    /// handles) lets named accounts in while it is off.
    pub play_flag: String,
}

impl Config {
    fn from_env() -> Config {
        let env = |k: &str, d: &str| std::env::var(k).unwrap_or_else(|_| d.to_string());
        let accounts_url = env("ACCOUNTS_URL", "https://accounts.3m4.net");
        Config {
            port: env("PORT", "8086").parse().unwrap_or(8086),
            database_url: env("DATABASE_URL", "postgres://wars:wars@localhost:5432/wars"),
            accounts_internal_url: env("ACCOUNTS_INTERNAL_URL", &accounts_url),
            accounts_url,
            guest_signing_key: env("GUEST_SIGNING_KEY", "dev-guest-key-change-me"),
            public_url: env("PUBLIC_URL", "http://localhost:8086"),
            version: env!("CARGO_PKG_VERSION").to_string(),
            exp_url: env("EXP_URL", "https://exp.3m4.net"),
            exp_client_key: env("EXP_CLIENT_KEY", ""),
            play_flag: env("PLAY_FLAG", "wars-play"),
        }
    }
}

pub struct AppState {
    pub cfg: Config,
    pub db: sqlx::PgPool,
    pub auth: auth::Auth,
    pub lobbies: Mutex<lobby::Lobbies>,
    pub data: &'static wars_sim::types::GameData,
    pub flags: flags::Flags,
}

pub type App = Arc<AppState>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,sqlx=warn".into()),
        )
        .init();
    let cfg = Config::from_env();
    let db = db::connect(&cfg.database_url).await?;
    sqlx::migrate!("./migrations").run(&db).await?;
    let state: App = Arc::new(AppState {
        auth: auth::Auth::new(&cfg),
        flags: flags::Flags::new(&cfg.exp_url, &cfg.exp_client_key, &cfg.play_flag),
        cfg: cfg.clone(),
        db,
        lobbies: Mutex::new(lobby::Lobbies::default()),
        data: wars_data::data(),
    });
    let app = routes::router(state.clone());
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], cfg.port));
    tracing::info!("wars-server {} listening on {}", cfg.version, addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
