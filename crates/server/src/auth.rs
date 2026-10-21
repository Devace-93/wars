//! Identity: accounts.3m4.net sessions (EdDSA JWT verified with the JWKS) and wars guest
//! tokens (HS256 signed by this server).

use crate::Config;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};
use wars_proto::PlayerInfo;

#[derive(Clone, Debug)]
pub struct Identity {
    pub id: String,
    pub handle: String,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub guest: bool,
    pub email_verified: bool,
}

impl Identity {
    pub fn info(&self, nickname: Option<String>) -> PlayerInfo {
        PlayerInfo {
            id: self.id.clone(),
            handle: self.handle.clone(),
            display: nickname.unwrap_or_else(|| self.handle.clone()),
            avatar_url: self.avatar_url.clone(),
            guest: self.guest,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct AccountsClaims {
    iss: String,
    aud: String,
    sub: String,
    email_verified: Option<bool>,
    handle: String,
    name: Option<String>,
    avatar_url: Option<String>,
    exp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct GuestClaims {
    sub: String,
    handle: String,
    guest: bool,
    exp: u64,
    iat: u64,
}

#[derive(Deserialize)]
struct Jwks {
    keys: Vec<Jwk>,
}
#[derive(Deserialize, Clone)]
struct Jwk {
    kid: String,
    x: String,
    #[allow(dead_code)]
    alg: Option<String>,
}

pub struct Auth {
    accounts_url: String,
    jwks_url: String,
    guest_key: Vec<u8>,
    keys: RwLock<(Instant, HashMap<String, DecodingKey>)>,
    http: reqwest::Client,
}

impl Auth {
    pub fn new(cfg: &Config) -> Auth {
        Auth {
            accounts_url: cfg.accounts_url.trim_end_matches('/').to_string(),
            jwks_url: format!(
                "{}/.well-known/jwks.json",
                cfg.accounts_internal_url.trim_end_matches('/')
            ),
            guest_key: cfg.guest_signing_key.as_bytes().to_vec(),
            keys: RwLock::new((Instant::now() - Duration::from_secs(3600), HashMap::new())),
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .expect("http client"),
        }
    }

    async fn refresh_keys(&self) {
        if let Ok(resp) = self.http.get(&self.jwks_url).send().await {
            if let Ok(jwks) = resp.json::<Jwks>().await {
                let mut map = HashMap::new();
                for k in jwks.keys {
                    if let Ok(dk) = DecodingKey::from_ed_components(&k.x) {
                        map.insert(k.kid, dk);
                    }
                }
                if let Ok(mut w) = self.keys.write() {
                    *w = (Instant::now(), map);
                }
                return;
            }
        }
        tracing::warn!("could not fetch JWKS from {}", self.jwks_url);
    }

    async fn key_for(&self, kid: &str) -> Option<DecodingKey> {
        let (stale, known) = {
            let r = self.keys.read().ok()?;
            (
                r.0.elapsed() > Duration::from_secs(600),
                r.1.get(kid).cloned(),
            )
        };
        if let Some(k) = known {
            if !stale {
                return Some(k);
            }
        }
        self.refresh_keys().await;
        self.keys.read().ok()?.1.get(kid).cloned()
    }

    /// Verify either kind of token.
    pub async fn verify(&self, token: &str) -> Result<Identity, &'static str> {
        let header = jsonwebtoken::decode_header(token).map_err(|_| "invalid_session")?;
        match header.alg {
            Algorithm::EdDSA => {
                let kid = header.kid.ok_or("invalid_session")?;
                let key = self.key_for(&kid).await.ok_or("invalid_session")?;
                let mut v = Validation::new(Algorithm::EdDSA);
                v.set_audience(&["3m4"]);
                v.set_issuer(&[self.accounts_url.as_str()]);
                v.leeway = 60;
                let data =
                    jsonwebtoken::decode::<AccountsClaims>(token, &key, &v).map_err(|e| {
                        if matches!(e.kind(), jsonwebtoken::errors::ErrorKind::ExpiredSignature) {
                            "session_expired"
                        } else {
                            "invalid_session"
                        }
                    })?;
                let c = data.claims;
                Ok(Identity {
                    id: c.sub,
                    handle: c.handle,
                    name: c.name,
                    avatar_url: c.avatar_url,
                    guest: false,
                    email_verified: c.email_verified.unwrap_or(false),
                })
            }
            Algorithm::HS256 => {
                let mut v = Validation::new(Algorithm::HS256);
                v.validate_aud = false;
                v.leeway = 60;
                let data = jsonwebtoken::decode::<GuestClaims>(
                    token,
                    &DecodingKey::from_secret(&self.guest_key),
                    &v,
                )
                .map_err(|_| "invalid_session")?;
                let c = data.claims;
                if !c.guest {
                    return Err("invalid_session");
                }
                Ok(Identity {
                    id: c.sub,
                    handle: c.handle.clone(),
                    name: Some(c.handle),
                    avatar_url: None,
                    guest: true,
                    email_verified: false,
                })
            }
            _ => Err("invalid_session"),
        }
    }

    pub fn guest_token(&self, nickname: &str) -> (String, Identity) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let id = format!("guest:{}", uuid::Uuid::new_v4());
        let handle = sanitize_nick(nickname);
        let claims = GuestClaims {
            sub: id.clone(),
            handle: handle.clone(),
            guest: true,
            exp: now + 30 * 24 * 3600,
            iat: now,
        };
        let token = jsonwebtoken::encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(&self.guest_key),
        )
        .expect("guest token");
        (
            token,
            Identity {
                id,
                handle: handle.clone(),
                name: Some(handle),
                avatar_url: None,
                guest: true,
                email_verified: false,
            },
        )
    }
}

pub fn sanitize_nick(s: &str) -> String {
    let t: String = s
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-' || *c == ' ')
        .take(24)
        .collect();
    let t = t.trim().to_string();
    if t.len() < 2 {
        "Guest".to_string()
    } else {
        t
    }
}

/// Pull a bearer token or the 3m4_session cookie out of request headers.
pub fn token_from_headers(headers: &axum::http::HeaderMap) -> Option<String> {
    if let Some(v) = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
    {
        if let Some(t) = v.strip_prefix("Bearer ") {
            return Some(t.trim().to_string());
        }
    }
    if let Some(c) = headers
        .get(axum::http::header::COOKIE)
        .and_then(|v| v.to_str().ok())
    {
        for part in c.split(';') {
            let part = part.trim();
            if let Some(v) = part.strip_prefix("3m4_session=") {
                return Some(v.to_string());
            }
        }
    }
    None
}
