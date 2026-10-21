use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub async fn connect(url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(8)
        .acquire_timeout(std::time::Duration::from_secs(10))
        .connect(url)
        .await?;
    Ok(pool)
}

/// Create or refresh the player row from identity claims; returns nothing.
pub async fn upsert_player(
    pool: &PgPool,
    id: &str,
    handle: &str,
    name: Option<&str>,
    avatar_url: Option<&str>,
    guest: bool,
) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO players (id, handle, name, avatar_url, guest) VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (id) DO UPDATE SET handle = EXCLUDED.handle, name = EXCLUDED.name, avatar_url = EXCLUDED.avatar_url, updated_at = now()",
    )
    .bind(id)
    .bind(handle)
    .bind(name)
    .bind(avatar_url)
    .bind(guest)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn nickname(pool: &PgPool, id: &str) -> Option<String> {
    sqlx::query_scalar::<_, Option<String>>("SELECT nickname FROM players WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .flatten()
}
