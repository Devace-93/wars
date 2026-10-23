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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Inject verification keys (tests, or a pre-fetched JWKS).
    pub fn set_keys(&self, keys: HashMap<String, DecodingKey>) {
        if let Ok(mut w) = self.keys.write() {
            *w = (Instant::now(), keys);
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;
    use ring::signature::{Ed25519KeyPair, KeyPair};

    fn cfg() -> Config {
        Config {
            port: 0,
            database_url: String::new(),
            accounts_url: "https://accounts.3m4.net".into(),
            accounts_internal_url: "http://127.0.0.1:1".into(),
            guest_signing_key: "test-guest-key".into(),
            public_url: String::new(),
            version: "test".into(),
        }
    }

    fn now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// An accounts-style EdDSA session token signed by a fresh Ed25519 key, verified through
    /// the JWKS path (jsonwebtoken 10, `from_ed_components`).
    #[tokio::test]
    async fn verifies_accounts_eddsa_session() {
        let rng = ring::rand::SystemRandom::new();
        let pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
        let pair = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).unwrap();
        let x = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(pair.public_key().as_ref());
        let auth = Auth::new(&cfg());
        let mut keys = HashMap::new();
        keys.insert(
            "kid1".to_string(),
            DecodingKey::from_ed_components(&x).unwrap(),
        );
        auth.set_keys(keys);
        let claims = AccountsClaims {
            iss: "https://accounts.3m4.net".into(),
            aud: "3m4".into(),
            sub: "user-uuid".into(),
            email_verified: Some(true),
            handle: "ana".into(),
            name: Some("Ana".into()),
            avatar_url: None,
            exp: now() + 600,
        };
        let mut header = Header::new(Algorithm::EdDSA);
        header.kid = Some("kid1".into());
        let token =
            jsonwebtoken::encode(&header, &claims, &EncodingKey::from_ed_der(pkcs8.as_ref()))
                .unwrap();
        let id = auth.verify(&token).await.expect("valid session");
        assert_eq!(id.id, "user-uuid");
        assert_eq!(id.handle, "ana");
        assert!(!id.guest);
        assert!(id.email_verified);
        // Wrong audience is rejected.
        let bad = AccountsClaims {
            aud: "other".into(),
            ..claims
        };
        let token =
            jsonwebtoken::encode(&header, &bad, &EncodingKey::from_ed_der(pkcs8.as_ref())).unwrap();
        assert_eq!(auth.verify(&token).await.unwrap_err(), "invalid_session");
    }

    #[tokio::test]
    async fn guest_tokens_round_trip_and_reject_tampering() {
        let auth = Auth::new(&cfg());
        let (token, id) = auth.guest_token("Enrique");
        let back = auth.verify(&token).await.unwrap();
        assert_eq!(back.id, id.id);
        assert!(back.guest);
        let other = Auth::new(&Config {
            guest_signing_key: "different".into(),
            ..cfg()
        });
        assert!(other.verify(&token).await.is_err());
    }
}
