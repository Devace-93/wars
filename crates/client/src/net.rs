//! Blocking networking for desktop and console: a WebSocket transport on a thread,
//! REST helpers and accounts login.

use crate::session::Transport;
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use wars_proto::*;

pub struct WsTransport {
    tx: Sender<Vec<u8>>,
    rx: Receiver<Vec<u8>>,
    closed: Arc<Mutex<bool>>,
}

impl WsTransport {
    /// Connect and send the Hello frame. Blocks until the socket is open.
    pub fn connect(ws_url: &str, token: &str) -> Result<WsTransport, String> {
        let (socket, _) = tungstenite::connect(ws_url).map_err(|e| e.to_string())?;
        let socket = Arc::new(Mutex::new(socket));
        if let Ok(mut s) = socket.lock() {
            if let tungstenite::stream::MaybeTlsStream::Plain(t) = s.get_mut() {
                let _ = t.set_nonblocking(false);
            }
        }
        let (out_tx, out_rx) = channel::<Vec<u8>>();
        let (in_tx, in_rx) = channel::<Vec<u8>>();
        let closed = Arc::new(Mutex::new(false));
        // Writer thread.
        {
            let socket = socket.clone();
            let closed = closed.clone();
            std::thread::spawn(move || {
                while let Ok(bytes) = out_rx.recv() {
                    let mut s = socket.lock().unwrap();
                    if s.send(tungstenite::Message::Binary(bytes)).is_err() {
                        *closed.lock().unwrap() = true;
                        break;
                    }
                }
            });
        }
        // Reader thread: the socket is shared, so reads use a short timeout loop.
        {
            let socket = socket.clone();
            let closed = closed.clone();
            std::thread::spawn(move || loop {
                let msg = {
                    let mut s = socket.lock().unwrap();
                    // Set a read timeout on the underlying TCP stream so we release the lock.
                    match s.get_mut() {
                        tungstenite::stream::MaybeTlsStream::Plain(t) => {
                            let _ = t.set_read_timeout(Some(std::time::Duration::from_millis(5)));
                        }
                        tungstenite::stream::MaybeTlsStream::Rustls(t) => {
                            let _ = t
                                .get_mut()
                                .set_read_timeout(Some(std::time::Duration::from_millis(5)));
                        }
                        _ => {}
                    }
                    s.read()
                };
                match msg {
                    Ok(tungstenite::Message::Binary(b)) => {
                        if in_tx.send(b).is_err() {
                            break;
                        }
                    }
                    Ok(tungstenite::Message::Close(_)) => {
                        *closed.lock().unwrap() = true;
                        break;
                    }
                    Ok(_) => {}
                    Err(tungstenite::Error::Io(e))
                        if e.kind() == std::io::ErrorKind::WouldBlock
                            || e.kind() == std::io::ErrorKind::TimedOut =>
                    {
                        std::thread::sleep(std::time::Duration::from_millis(2));
                    }
                    Err(_) => {
                        *closed.lock().unwrap() = true;
                        break;
                    }
                }
            });
        }
        let t = WsTransport {
            tx: out_tx,
            rx: in_rx,
            closed,
        };
        t.tx.send(encode(&ClientMsg::Hello {
            token: token.to_string(),
            version: PROTOCOL_VERSION,
        }))
        .map_err(|e| e.to_string())?;
        Ok(t)
    }

    /// Block (with timeout) for the next frame; used by the lobby screens.
    pub fn recv_timeout(&self, ms: u64) -> Option<Vec<u8>> {
        self.rx
            .recv_timeout(std::time::Duration::from_millis(ms))
            .ok()
    }
}

impl Transport for WsTransport {
    fn send(&mut self, bytes: Vec<u8>) {
        let _ = self.tx.send(bytes);
    }
    fn try_recv(&mut self) -> Option<Vec<u8>> {
        match self.rx.try_recv() {
            Ok(b) => Some(b),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                *self.closed.lock().unwrap() = true;
                None
            }
        }
    }
    fn is_closed(&self) -> bool {
        *self.closed.lock().unwrap()
    }
}

/// Server base URL (https://wars.3m4.net or http://localhost:8086) -> ws URL.
pub fn ws_url(base: &str) -> String {
    let b = base.trim_end_matches('/');
    let b = b
        .replacen("https://", "wss://", 1)
        .replacen("http://", "ws://", 1);
    format!("{b}/ws")
}

/// Log in against accounts.3m4.net with email + password; returns the session JWT and the
/// refresh cookie so the caller can persist it.
pub fn accounts_login(
    accounts_url: &str,
    email: &str,
    password: &str,
) -> Result<(String, String), String> {
    let agent = ureq::AgentBuilder::new().build();
    let resp = agent
        .post(&format!(
            "{}/api/auth/login",
            accounts_url.trim_end_matches('/')
        ))
        .set("Content-Type", "application/json")
        .send_json(serde_json::json!({ "email": email, "password": password }))
        .map_err(|e| match e {
            ureq::Error::Status(401, _) => "Wrong email or password".to_string(),
            other => other.to_string(),
        })?;
    let mut session = None;
    let mut refresh = None;
    for c in resp.all("set-cookie") {
        if let Some(v) = c.strip_prefix("3m4_session=") {
            session = Some(v.split(';').next().unwrap_or("").to_string());
        }
        if let Some(v) = c.strip_prefix("3m4_refresh=") {
            refresh = Some(v.split(';').next().unwrap_or("").to_string());
        }
    }
    match (session, refresh) {
        (Some(s), Some(r)) => Ok((s, r)),
        _ => Err("Login succeeded but no session cookie was returned".into()),
    }
}

/// Refresh the session JWT with the stored refresh cookie.
pub fn accounts_refresh(accounts_url: &str, refresh: &str) -> Result<(String, String), String> {
    let agent = ureq::AgentBuilder::new().build();
    let resp = agent
        .post(&format!(
            "{}/api/session/refresh",
            accounts_url.trim_end_matches('/')
        ))
        .set("Content-Type", "application/json")
        .set("Cookie", &format!("3m4_refresh={refresh}"))
        .send_string("{}")
        .map_err(|e| e.to_string())?;
    let mut session = None;
    let mut new_refresh = None;
    for c in resp.all("set-cookie") {
        if let Some(v) = c.strip_prefix("3m4_session=") {
            session = Some(v.split(';').next().unwrap_or("").to_string());
        }
        if let Some(v) = c.strip_prefix("3m4_refresh=") {
            new_refresh = Some(v.split(';').next().unwrap_or("").to_string());
        }
    }
    match (session, new_refresh) {
        (Some(s), Some(r)) => Ok((s, r)),
        _ => Err("Refresh did not return a session".into()),
    }
}

/// Ask the wars server for a guest token.
pub fn guest_token(server_url: &str, nickname: &str) -> Result<TokenResponse, String> {
    let resp = ureq::post(&format!("{}/api/guest", server_url.trim_end_matches('/')))
        .set("Content-Type", "application/json")
        .send_json(serde_json::json!({ "nickname": nickname }))
        .map_err(|e| e.to_string())?;
    resp.into_json::<TokenResponse>().map_err(|e| e.to_string())
}

/// Generic authenticated GET returning JSON.
pub fn get_json<T: serde::de::DeserializeOwned>(
    server_url: &str,
    path: &str,
    token: &str,
) -> Result<T, String> {
    let resp = ureq::get(&format!("{}{}", server_url.trim_end_matches('/'), path))
        .set("Authorization", &format!("Bearer {token}"))
        .call()
        .map_err(|e| e.to_string())?;
    resp.into_json::<T>().map_err(|e| e.to_string())
}

pub fn post_json<T: serde::de::DeserializeOwned>(
    server_url: &str,
    path: &str,
    token: &str,
    body: serde_json::Value,
) -> Result<T, String> {
    let resp = ureq::post(&format!("{}{}", server_url.trim_end_matches('/'), path))
        .set("Authorization", &format!("Bearer {token}"))
        .set("Content-Type", "application/json")
        .send_json(body)
        .map_err(|e| e.to_string())?;
    resp.into_json::<T>().map_err(|e| e.to_string())
}

pub fn post_bytes(
    server_url: &str,
    path: &str,
    token: &str,
    bytes: &[u8],
) -> Result<serde_json::Value, String> {
    let resp = ureq::post(&format!("{}{}", server_url.trim_end_matches('/'), path))
        .set("Authorization", &format!("Bearer {token}"))
        .set("Content-Type", "application/octet-stream")
        .send_bytes(bytes)
        .map_err(|e| e.to_string())?;
    resp.into_json::<serde_json::Value>()
        .map_err(|e| e.to_string())
}

pub fn get_bytes(server_url: &str, path: &str, token: &str) -> Result<Vec<u8>, String> {
    let resp = ureq::get(&format!("{}{}", server_url.trim_end_matches('/'), path))
        .set("Authorization", &format!("Bearer {token}"))
        .call()
        .map_err(|e| e.to_string())?;
    let mut buf = Vec::new();
    std::io::Read::read_to_end(&mut resp.into_reader(), &mut buf).map_err(|e| e.to_string())?;
    Ok(buf)
}
