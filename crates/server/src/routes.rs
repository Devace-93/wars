//! REST API and router.

use crate::auth::{token_from_headers, Identity};
use crate::App;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;
use sqlx::Row;
use tower_http::cors::{AllowOrigin, CorsLayer};
use wars_proto::*;

pub struct ApiError(StatusCode, &'static str, String);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.0,
            Json(json!({ "error": { "code": self.1, "message": self.2 } })),
        )
            .into_response()
    }
}

fn err(status: StatusCode, code: &'static str, msg: impl Into<String>) -> ApiError {
    ApiError(status, code, msg.into())
}

async fn identity(app: &App, headers: &HeaderMap) -> Result<Identity, ApiError> {
    let token = token_from_headers(headers)
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, "no_session", "Log in first"))?;
    let id = app
        .auth
        .verify(&token)
        .await
        .map_err(|code| err(StatusCode::UNAUTHORIZED, code, "Session invalid or expired"))?;
    let _ = crate::db::upsert_player(
        &app.db,
        &id.id,
        &id.handle,
        id.name.as_deref(),
        id.avatar_url.as_deref(),
        id.guest,
    )
    .await;
    Ok(id)
}

pub fn router(app: App) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(|origin, _| {
            let o = origin.to_str().unwrap_or("");
            o.ends_with(".3m4.net")
                || o == "https://3m4.net"
                || o.starts_with("http://localhost")
                || o.starts_with("http://127.0.0.1")
        }))
        .allow_credentials(true)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PATCH,
            axum::http::Method::DELETE,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
        ]);
    Router::new()
        .route("/", get(index))
        .route("/api/play", get(play_status))
        .route("/og-image.png", get(og_image))
        .route("/healthz", get(healthz))
        .route("/api/healthz", get(healthz))
        .route("/api/guest", post(guest))
        .route("/api/me", get(me).patch(patch_me))
        .route("/api/lobbies", get(lobbies))
        .route("/api/saves", get(list_saves).post(upload_save))
        .route("/api/saves/:id", get(get_save).delete(delete_save))
        .route("/api/scenarios", get(list_scenarios).post(upload_scenario))
        .route(
            "/api/scenarios/:id",
            get(get_scenario).delete(delete_scenario),
        )
        .route("/api/leaderboard", get(leaderboard))
        .route("/api/matches", get(my_matches))
        .route("/api/replays/:id", get(get_replay))
        .route("/api/civs", get(civs))
        .route("/ws", get(crate::ws::ws_handler))
        .route("/play", get(play_index))
        .route("/play/", get(play_index))
        .nest_service(
            "/play/pkg",
            tower_http::services::ServeDir::new(format!("{}/pkg", web_dir())),
        )
        .layer(cors)
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(app)
}

/// Who is looking, for the browser-beta flag: account id and handle when the request
/// carries a valid session (cookie or bearer), nothing for guests and visitors.
async fn viewer(app: &App, headers: &HeaderMap) -> crate::flags::Viewer {
    let Some(token) = token_from_headers(headers) else {
        return crate::flags::Viewer::default();
    };
    match app.auth.verify(&token).await {
        Ok(id) if !id.guest => crate::flags::Viewer {
            distinct_id: Some(id.id.clone()),
            handle: Some(id.handle.clone()),
        },
        _ => crate::flags::Viewer::default(),
    }
}

async fn play_open(app: &App, headers: &HeaderMap) -> (bool, crate::flags::Viewer) {
    let v = viewer(app, headers).await;
    let open = app.flags.play_access(&v).await == crate::flags::Access::Open;
    (open, v)
}

/// `{ "enabled": bool }`: may this viewer open `/play/`?
async fn play_status(State(app): State<App>, headers: HeaderMap) -> Json<serde_json::Value> {
    let (open, _) = play_open(&app, &headers).await;
    Json(json!({ "enabled": open, "beta": !app.flags.disabled() }))
}

/// Landing page. The "Play in your browser" card is only shown to viewers the
/// `wars-play` flag lets in; everyone else sees a closed-beta note instead.
async fn index(State(app): State<App>, headers: HeaderMap) -> Html<String> {
    const PAGE: &str = include_str!("../static/index.html");
    let (open, _) = play_open(&app, &headers).await;
    Html(render_index(PAGE, open))
}

fn render_index(page: &str, play_open: bool) -> String {
    const START: &str = "<!--PLAY_CARD_START-->";
    const END: &str = "<!--PLAY_CARD_END-->";
    if play_open {
        return page.replace(START, "").replace(END, "");
    }
    match (page.find(START), page.find(END)) {
        (Some(a), Some(b)) if b > a => {
            let note = r#"<div class="card">
      <h2>Browser version: closed beta</h2>
      <p>The in-browser build is being finished and is open to testers only for now. <a href="/play/">Testers log in here</a>.</p>
    </div>"#;
            format!("{}{}{}", &page[..a], note, &page[b + END.len()..])
        }
        _ => page.to_string(),
    }
}

fn web_dir() -> String {
    std::env::var("WEB_DIR").unwrap_or_else(|_| "web".into())
}

/// The browser client page. `__BUILD__` becomes a per-process stamp so browsers never
/// run a cached module against a newer server.
async fn play_index(State(app): State<App>, headers: HeaderMap) -> Response {
    let (open, v) = play_open(&app, &headers).await;
    if !open {
        return beta_page(&app, &v);
    }
    static STAMP: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    let stamp = STAMP.get_or_init(|| {
        let t = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        format!("{}-{t}", env!("CARGO_PKG_VERSION"))
    });
    match tokio::fs::read_to_string(format!("{}/index.html", web_dir())).await {
        Ok(html) => Html(html.replace("__BUILD__", stamp)).into_response(),
        Err(_) => (
            StatusCode::NOT_FOUND,
            "The browser client is not installed on this server",
        )
            .into_response(),
    }
}

/// The closed-beta page: explains the state and offers the login that may unlock it.
fn beta_page(app: &App, v: &crate::flags::Viewer) -> Response {
    const PAGE: &str = include_str!("../static/beta.html");
    let return_to = format!("{}/play/", app.cfg.public_url.trim_end_matches('/'));
    let login = format!(
        "{}/login?return_to={}",
        app.cfg.accounts_url.trim_end_matches('/'),
        percent_encode(&return_to)
    );
    let (message, actions) = match &v.handle {
        Some(h) => (
            format!(
                "Hi <b>{}</b>. The browser version is not open to everyone yet, and your account is not on the testers list. \
                 Ask to be added, or come back when the beta opens.",
                html_escape(h)
            ),
            r#"<a class="btn" href="/">Back to the downloads</a>"#.to_string(),
        ),
        None => (
            "The browser version of Wars is in a closed beta while the last rough edges are filed off. \
             Testers can log in with their 3m4 account to play; everyone else can use the desktop, Android or console versions today."
                .to_string(),
            format!(r#"<a class="btn" href="{login}">Log in as a tester</a> <a class="btn" href="/">Back to the downloads</a>"#),
        ),
    };
    (
        StatusCode::OK,
        [(axum::http::header::CACHE_CONTROL, "no-store")],
        Html(
            PAGE.replace("__MESSAGE__", &message)
                .replace("__ACTIONS__", &actions),
        ),
    )
        .into_response()
}

/// Minimal percent-encoding for a query value (RFC 3986 unreserved characters kept).
fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

async fn og_image() -> Response {
    (
        [
            (axum::http::header::CONTENT_TYPE, "image/png"),
            (axum::http::header::CACHE_CONTROL, "public, max-age=86400"),
        ],
        include_bytes!("../static/og-image.png").as_slice(),
    )
        .into_response()
}

async fn healthz() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}

async fn guest(
    State(app): State<App>,
    Json(body): Json<GuestRequest>,
) -> Result<Json<TokenResponse>, ApiError> {
    let (token, id) = app.auth.guest_token(&body.nickname);
    let _ =
        crate::db::upsert_player(&app.db, &id.id, &id.handle, id.name.as_deref(), None, true).await;
    Ok(Json(TokenResponse {
        token,
        player: id.info(None),
    }))
}

async fn me(
    State(app): State<App>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    let id = identity(&app, &headers).await?;
    let row = sqlx::query("SELECT nickname, elo, wins, losses FROM players WHERE id = $1")
        .bind(&id.id)
        .fetch_one(&app.db)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "db", e.to_string()))?;
    let nickname: Option<String> = row.get("nickname");
    Ok(Json(json!({
        "player": id.info(nickname.clone()),
        "nickname": nickname,
        "elo": row.get::<i32, _>("elo"),
        "wins": row.get::<i32, _>("wins"),
        "losses": row.get::<i32, _>("losses"),
        "email_verified": id.email_verified,
    })))
}

#[derive(Deserialize)]
struct PatchMe {
    nickname: Option<String>,
}

async fn patch_me(
    State(app): State<App>,
    headers: HeaderMap,
    Json(body): Json<PatchMe>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let id = identity(&app, &headers).await?;
    let nick = body
        .nickname
        .map(|n| crate::auth::sanitize_nick(&n))
        .filter(|n| n != "Guest");
    sqlx::query("UPDATE players SET nickname = $2, updated_at = now() WHERE id = $1")
        .bind(&id.id)
        .bind(&nick)
        .execute(&app.db)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "db", e.to_string()))?;
    Ok(Json(json!({ "player": id.info(nick) })))
}

async fn lobbies(State(app): State<App>) -> Json<Vec<LobbySummary>> {
    Json(app.lobbies.lock().await.public_list())
}

async fn civs(State(app): State<App>) -> Json<serde_json::Value> {
    Json(json!(app.data.civs.iter().map(|c| json!({ "key": c.key, "name": c.name, "description": c.description, "bonuses": c.bonus_text, "team_bonus": c.team_bonus_text, "unique_unit": c.unique_unit })).collect::<Vec<_>>()))
}

#[derive(Deserialize)]
struct NameQuery {
    name: Option<String>,
    public: Option<bool>,
}

async fn list_saves(
    State(app): State<App>,
    headers: HeaderMap,
) -> Result<Json<Vec<SaveSummary>>, ApiError> {
    let id = identity(&app, &headers).await?;
    let rows = sqlx::query("SELECT id, name, tick, lobby, created_at FROM saves WHERE owner_id = $1 ORDER BY created_at DESC LIMIT 100").bind(&id.id).fetch_all(&app.db).await.map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "db", e.to_string()))?;
    let v = rows
        .iter()
        .map(|r| {
            let lobby: Option<serde_json::Value> = r.get("lobby");
            let players = lobby
                .and_then(|l| serde_json::from_value::<Lobby>(l).ok())
                .map(|l| {
                    l.slots
                        .iter()
                        .filter(|s| s.open && (s.player.is_some() || s.is_ai))
                        .map(|s| {
                            s.player
                                .as_ref()
                                .map(|p| p.display.clone())
                                .unwrap_or_else(|| "Computer".into())
                        })
                        .collect()
                })
                .unwrap_or_default();
            SaveSummary {
                id: r.get::<uuid::Uuid, _>("id").to_string(),
                name: r.get("name"),
                tick: r.get::<i32, _>("tick") as u32,
                players,
                created_at: r
                    .get::<chrono::DateTime<chrono::Utc>, _>("created_at")
                    .to_rfc3339(),
            }
        })
        .collect();
    Ok(Json(v))
}

/// Upload a local save (body = postcard world snapshot) so it can be resumed online.
async fn upload_save(
    State(app): State<App>,
    headers: HeaderMap,
    Query(q): Query<NameQuery>,
    body: axum::body::Bytes,
) -> Result<Json<serde_json::Value>, ApiError> {
    let id = identity(&app, &headers).await?;
    if id.guest {
        return Err(err(
            StatusCode::FORBIDDEN,
            "guest",
            "Guests cannot store saves",
        ));
    }
    if body.len() > 16 * 1024 * 1024 {
        return Err(err(
            StatusCode::PAYLOAD_TOO_LARGE,
            "too_large",
            "Save too large",
        ));
    }
    let world = wars_sim::world::World::from_snapshot(&body)
        .map_err(|_| err(StatusCode::BAD_REQUEST, "bad_save", "Not a Wars save"))?;
    let sid = uuid::Uuid::new_v4();
    let name = q.name.unwrap_or_else(|| "Uploaded save".into());
    sqlx::query(
        "INSERT INTO saves (id, owner_id, name, tick, snapshot) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(sid)
    .bind(&id.id)
    .bind(name.chars().take(60).collect::<String>())
    .bind(world.tick as i32)
    .bind(body.to_vec())
    .execute(&app.db)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "db", e.to_string()))?;
    Ok(Json(json!({ "id": sid.to_string() })))
}

async fn get_save(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<uuid::Uuid>,
) -> Result<Response, ApiError> {
    let me = identity(&app, &headers).await?;
    let bytes: Vec<u8> =
        sqlx::query_scalar("SELECT snapshot FROM saves WHERE id = $1 AND owner_id = $2")
            .bind(id)
            .bind(&me.id)
            .fetch_optional(&app.db)
            .await
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "db", e.to_string()))?
            .ok_or_else(|| err(StatusCode::NOT_FOUND, "not_found", "No such save"))?;
    Ok((
        [(axum::http::header::CONTENT_TYPE, "application/octet-stream")],
        bytes,
    )
        .into_response())
}

async fn delete_save(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<uuid::Uuid>,
) -> Result<StatusCode, ApiError> {
    let me = identity(&app, &headers).await?;
    sqlx::query("DELETE FROM saves WHERE id = $1 AND owner_id = $2")
        .bind(id)
        .bind(&me.id)
        .execute(&app.db)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "db", e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_scenarios(
    State(app): State<App>,
    headers: HeaderMap,
) -> Result<Json<Vec<ScenarioSummary>>, ApiError> {
    let mine = identity(&app, &headers)
        .await
        .ok()
        .map(|i| i.id)
        .unwrap_or_default();
    let rows = sqlx::query("SELECT id, name, author, description, public, players, size, updated_at FROM scenarios WHERE public OR owner_id = $1 ORDER BY updated_at DESC LIMIT 200").bind(&mine).fetch_all(&app.db).await.map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "db", e.to_string()))?;
    Ok(Json(
        rows.iter()
            .map(|r| ScenarioSummary {
                id: r.get::<uuid::Uuid, _>("id").to_string(),
                name: r.get("name"),
                author: r.get("author"),
                description: r.get("description"),
                public: r.get("public"),
                players: r.get::<i32, _>("players") as usize,
                size: r.get("size"),
                updated_at: r
                    .get::<chrono::DateTime<chrono::Utc>, _>("updated_at")
                    .to_rfc3339(),
            })
            .collect(),
    ))
}

async fn upload_scenario(
    State(app): State<App>,
    headers: HeaderMap,
    Query(q): Query<NameQuery>,
    body: axum::body::Bytes,
) -> Result<Json<serde_json::Value>, ApiError> {
    let me = identity(&app, &headers).await?;
    if me.guest {
        return Err(err(
            StatusCode::FORBIDDEN,
            "guest",
            "Guests cannot store scenarios",
        ));
    }
    let sc = wars_mapgen::scenario::Scenario::from_bytes(&body).map_err(|_| {
        err(
            StatusCode::BAD_REQUEST,
            "bad_scenario",
            "Not a Wars scenario",
        )
    })?;
    let sid = uuid::Uuid::new_v4();
    let name = q.name.unwrap_or_else(|| sc.name.clone());
    let players = sc.players.iter().filter(|p| p.active).count() as i32;
    sqlx::query("INSERT INTO scenarios (id, owner_id, name, author, description, public, players, size, data) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)")
        .bind(sid)
        .bind(&me.id)
        .bind(name.chars().take(60).collect::<String>())
        .bind(if sc.author.is_empty() { me.handle.clone() } else { sc.author.clone() })
        .bind(sc.description.chars().take(500).collect::<String>())
        .bind(q.public.unwrap_or(false))
        .bind(players)
        .bind(sc.map.width)
        .bind(body.to_vec())
        .execute(&app.db)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "db", e.to_string()))?;
    Ok(Json(json!({ "id": sid.to_string() })))
}

async fn get_scenario(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<uuid::Uuid>,
) -> Result<Response, ApiError> {
    let mine = identity(&app, &headers)
        .await
        .ok()
        .map(|i| i.id)
        .unwrap_or_default();
    let bytes: Vec<u8> = sqlx::query_scalar(
        "SELECT data FROM scenarios WHERE id = $1 AND (public OR owner_id = $2)",
    )
    .bind(id)
    .bind(&mine)
    .fetch_optional(&app.db)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "db", e.to_string()))?
    .ok_or_else(|| err(StatusCode::NOT_FOUND, "not_found", "No such scenario"))?;
    Ok((
        [(axum::http::header::CONTENT_TYPE, "application/octet-stream")],
        bytes,
    )
        .into_response())
}

async fn delete_scenario(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<uuid::Uuid>,
) -> Result<StatusCode, ApiError> {
    let me = identity(&app, &headers).await?;
    sqlx::query("DELETE FROM scenarios WHERE id = $1 AND owner_id = $2")
        .bind(id)
        .bind(&me.id)
        .execute(&app.db)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "db", e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

async fn leaderboard(State(app): State<App>) -> Result<Json<Vec<LeaderboardEntry>>, ApiError> {
    let rows = sqlx::query("SELECT handle, COALESCE(nickname, handle) AS display, elo, wins, losses FROM players WHERE NOT guest AND wins + losses > 0 ORDER BY elo DESC LIMIT 100").fetch_all(&app.db).await.map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "db", e.to_string()))?;
    Ok(Json(
        rows.iter()
            .map(|r| LeaderboardEntry {
                handle: r.get("handle"),
                display: r.get("display"),
                elo: r.get("elo"),
                wins: r.get("wins"),
                losses: r.get("losses"),
            })
            .collect(),
    ))
}

async fn my_matches(
    State(app): State<App>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    let me = identity(&app, &headers).await?;
    let rows = sqlx::query("SELECT m.id, m.name, m.status, m.winner_team, m.ticks, m.started_at, mp.result, mp.civ, mp.team FROM matches m JOIN match_players mp ON mp.match_id = m.id WHERE mp.player_id = $1 ORDER BY m.started_at DESC LIMIT 50").bind(&me.id).fetch_all(&app.db).await.map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "db", e.to_string()))?;
    Ok(Json(json!(rows
        .iter()
        .map(|r| json!({
            "id": r.get::<uuid::Uuid, _>("id").to_string(),
            "name": r.get::<String, _>("name"),
            "status": r.get::<String, _>("status"),
            "winner_team": r.get::<Option<i32>, _>("winner_team"),
            "ticks": r.get::<i32, _>("ticks"),
            "started_at": r.get::<chrono::DateTime<chrono::Utc>, _>("started_at").to_rfc3339(),
            "result": r.get::<Option<String>, _>("result"),
            "civ": r.get::<String, _>("civ"),
            "team": r.get::<i32, _>("team"),
        }))
        .collect::<Vec<_>>())))
}

async fn get_replay(
    State(app): State<App>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Response, ApiError> {
    let bytes: Vec<u8> = sqlx::query_scalar("SELECT data FROM replays WHERE match_id = $1")
        .bind(id)
        .fetch_optional(&app.db)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "db", e.to_string()))?
        .ok_or_else(|| {
            err(
                StatusCode::NOT_FOUND,
                "not_found",
                "No replay for that match",
            )
        })?;
    Ok((
        [(axum::http::header::CONTENT_TYPE, "application/octet-stream")],
        bytes,
    )
        .into_response())
}

#[cfg(test)]
mod tests {
    use super::render_index;

    #[test]
    fn play_card_is_shown_or_replaced() {
        let page = "a<!--PLAY_CARD_START--><div>card</div><!--PLAY_CARD_END-->z";
        assert_eq!(render_index(page, true), "a<div>card</div>z");
        let closed = render_index(page, false);
        assert!(closed.starts_with('a') && closed.ends_with('z'));
        assert!(!closed.contains("<div>card</div>") && closed.contains("closed beta"));
    }
}
