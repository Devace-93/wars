//! Feature flags from exp (exp.3m4.net): the browser client is a closed beta until
//! the `wars-play` flag is on. Evaluations go through exp's evaluation API with the
//! environment client key and are cached for a short while per viewer.
//!
//! Decision for a viewer:
//! - no `EXP_CLIENT_KEY` configured → open (local development, self-hosters);
//! - the viewer's handle is listed in the json flag `<flag>-testers` → open;
//! - otherwise the boolean flag `<flag>` decides (exp applies its rollout
//!   percentage to the viewer's account id when there is one);
//! - exp unreachable or answering an error → closed (fail safe), retried soon.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const TTL_OK: Duration = Duration::from_secs(30);
const TTL_ERR: Duration = Duration::from_secs(10);

/// Who is asking, as far as the flag is concerned.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Viewer {
    /// Stable id for rollout buckets (account id). Guests and anonymous visitors
    /// have none and are only let in at 100 % rollout.
    pub distinct_id: Option<String>,
    /// Account handle, matched against the testers list.
    pub handle: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Access {
    Open,
    Closed,
}

pub struct Flags {
    url: String,
    key: String,
    flag: String,
    http: reqwest::Client,
    cache: Mutex<HashMap<String, (Instant, Access)>>,
}

#[derive(serde::Deserialize)]
struct Evaluated {
    enabled: bool,
    #[serde(default)]
    value: serde_json::Value,
}

impl Flags {
    pub fn new(url: &str, key: &str, flag: &str) -> Flags {
        Flags {
            url: url.trim_end_matches('/').to_string(),
            key: key.trim().to_string(),
            flag: flag.to_string(),
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(4))
                .build()
                .expect("http client"),
            cache: Mutex::new(HashMap::new()),
        }
    }

    /// True when no exp key is configured: everything is open.
    pub fn disabled(&self) -> bool {
        self.key.is_empty()
    }

    pub fn flag_name(&self) -> &str {
        &self.flag
    }

    /// May the viewer use the browser client?
    pub async fn play_access(&self, viewer: &Viewer) -> Access {
        if self.disabled() {
            return Access::Open;
        }
        let cache_key = format!(
            "{}|{}",
            viewer.distinct_id.as_deref().unwrap_or(""),
            viewer.handle.as_deref().unwrap_or("")
        );
        if let Some((at, access)) = self.cache.lock().unwrap().get(&cache_key) {
            let ttl = if *access == Access::Open {
                TTL_OK
            } else {
                TTL_ERR.max(TTL_OK)
            };
            if at.elapsed() < ttl {
                return access.clone();
            }
        }
        let (access, ttl) = match self.fetch(viewer.distinct_id.as_deref()).await {
            Ok(flags) => (decide(&flags, &self.flag, viewer), TTL_OK),
            Err(e) => {
                tracing::warn!("exp evaluation failed: {e}");
                (Access::Closed, TTL_ERR)
            }
        };
        // Errors are cached for a shorter time than real answers.
        let at = if ttl == TTL_ERR {
            Instant::now() - (TTL_OK - TTL_ERR)
        } else {
            Instant::now()
        };
        self.cache
            .lock()
            .unwrap()
            .insert(cache_key, (at, access.clone()));
        access
    }

    async fn fetch(&self, distinct_id: Option<&str>) -> Result<HashMap<String, Evaluated>, String> {
        let mut req = self
            .http
            .get(format!("{}/api/eval/flags", self.url))
            .header("X-Environment-Key", &self.key);
        if let Some(id) = distinct_id {
            req = req.query(&[("distinct_id", id)]);
        }
        let resp = req.send().await.map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("HTTP {}", resp.status()));
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        parse_eval(&body)
    }
}

/// exp answers `{ "flags": { "<key>": { "kind", "enabled", "value" } } }`; accept the
/// bare map too, so a self-hosted exp of another version still works.
fn parse_eval(body: &serde_json::Value) -> Result<HashMap<String, Evaluated>, String> {
    let map = body.get("flags").unwrap_or(body);
    serde_json::from_value(map.clone()).map_err(|e| format!("unexpected evaluation body: {e}"))
}

fn decide(flags: &HashMap<String, Evaluated>, flag: &str, viewer: &Viewer) -> Access {
    if let (Some(handle), Some(testers)) = (
        viewer.handle.as_deref(),
        flags.get(&format!("{flag}-testers")),
    ) {
        let listed = testers.enabled
            && testers
                .value
                .as_array()
                .map(|a| {
                    a.iter().any(|v| {
                        v.as_str()
                            .map(|s| s.eq_ignore_ascii_case(handle))
                            .unwrap_or(false)
                    })
                })
                .unwrap_or(false);
        if listed {
            return Access::Open;
        }
    }
    match flags.get(flag) {
        Some(f) if f.enabled && f.value.as_bool().unwrap_or(true) => Access::Open,
        _ => Access::Closed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flags(json: &str) -> HashMap<String, Evaluated> {
        parse_eval(&serde_json::from_str(json).unwrap()).unwrap()
    }

    #[test]
    fn boolean_flag_decides_for_everyone() {
        let on = flags(r#"{"flags":{"wars-play":{"kind":"boolean","enabled":true,"value":true}}}"#);
        let off =
            flags(r#"{"flags":{"wars-play":{"kind":"boolean","enabled":false,"value":false}}}"#);
        let anon = Viewer::default();
        assert_eq!(decide(&on, "wars-play", &anon), Access::Open);
        assert_eq!(decide(&off, "wars-play", &anon), Access::Closed);
        assert_eq!(decide(&HashMap::new(), "wars-play", &anon), Access::Closed);
    }

    #[test]
    fn testers_list_opens_the_door_while_the_flag_is_off() {
        let f = flags(
            r#"{"flags":{
                "wars-play":{"kind":"boolean","enabled":false,"value":false},
                "wars-play-testers":{"kind":"json","enabled":true,"value":["Devace","friend"]}
            }}"#,
        );
        let me = Viewer {
            distinct_id: Some("u1".into()),
            handle: Some("devace".into()),
        };
        let other = Viewer {
            distinct_id: Some("u2".into()),
            handle: Some("stranger".into()),
        };
        assert_eq!(decide(&f, "wars-play", &me), Access::Open);
        assert_eq!(decide(&f, "wars-play", &other), Access::Closed);
        assert_eq!(decide(&f, "wars-play", &Viewer::default()), Access::Closed);
    }

    #[test]
    fn bare_map_is_accepted_too() {
        let f = flags(r#"{"wars-play":{"kind":"boolean","enabled":true,"value":true}}"#);
        assert_eq!(decide(&f, "wars-play", &Viewer::default()), Access::Open);
    }

    #[tokio::test]
    async fn without_a_key_everything_is_open() {
        let f = Flags::new("https://exp.invalid", "", "wars-play");
        assert_eq!(f.play_access(&Viewer::default()).await, Access::Open);
    }

    #[tokio::test]
    async fn unreachable_exp_closes_the_beta() {
        let f = Flags::new("http://127.0.0.1:9", "exp_env_test", "wars-play");
        assert_eq!(f.play_access(&Viewer::default()).await, Access::Closed);
    }
}
