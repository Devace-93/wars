//! Local files: saved games, scenarios and settings live under the platform data dir.

use std::path::PathBuf;
use wars_sim::world::World;

static OVERRIDE: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();

/// Mobile shells point the data dir at the app's private files directory.
pub fn set_data_dir(p: PathBuf) {
    let _ = OVERRIDE.set(p);
}

pub fn data_dir() -> PathBuf {
    if let Some(p) = OVERRIDE.get() {
        let _ = std::fs::create_dir_all(p.join("saves"));
        let _ = std::fs::create_dir_all(p.join("scenarios"));
        return p.clone();
    }
    if let Some(d) = directories::ProjectDirs::from("net", "3m4", "wars") {
        let p = d.data_dir().to_path_buf();
        let _ = std::fs::create_dir_all(p.join("saves"));
        let _ = std::fs::create_dir_all(p.join("scenarios"));
        return p;
    }
    PathBuf::from(".")
}

pub fn save_world(name: &str, world: &World) -> std::io::Result<PathBuf> {
    let path = data_dir()
        .join("saves")
        .join(format!("{}.wars", sanitize(name)));
    std::fs::write(&path, world.snapshot())?;
    Ok(path)
}

pub fn load_world(path: &std::path::Path) -> std::io::Result<World> {
    let bytes = std::fs::read(path)?;
    World::from_snapshot(&bytes)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
}

pub fn list_saves() -> Vec<(String, PathBuf)> {
    list(data_dir().join("saves"), "wars")
}

pub fn save_scenario(sc: &wars_mapgen::scenario::Scenario) -> std::io::Result<PathBuf> {
    let path = data_dir()
        .join("scenarios")
        .join(format!("{}.scn", sanitize(&sc.name)));
    std::fs::write(&path, sc.to_bytes())?;
    Ok(path)
}

pub fn load_scenario(path: &std::path::Path) -> std::io::Result<wars_mapgen::scenario::Scenario> {
    let bytes = std::fs::read(path)?;
    wars_mapgen::scenario::Scenario::from_bytes(&bytes)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
}

pub fn list_scenarios() -> Vec<(String, PathBuf)> {
    list(data_dir().join("scenarios"), "scn")
}

fn list(dir: PathBuf, ext: &str) -> Vec<(String, PathBuf)> {
    let mut v: Vec<(String, PathBuf)> = std::fs::read_dir(dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().map(|x| x == ext).unwrap_or(false))
                .map(|p| {
                    (
                        p.file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_default(),
                        p,
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}

pub fn sanitize(name: &str) -> String {
    let s: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let s = s.trim().to_string();
    if s.is_empty() {
        "unnamed".into()
    } else {
        s
    }
}

/// Simple key/value settings (JSON) for hotkeys, server URL, last login.
pub fn load_settings() -> serde_json::Value {
    let p = data_dir().join("settings.json");
    std::fs::read_to_string(p)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(serde_json::json!({}))
}

pub fn save_settings(v: &serde_json::Value) {
    let p = data_dir().join("settings.json");
    let _ = std::fs::write(p, serde_json::to_string_pretty(v).unwrap_or_default());
}
