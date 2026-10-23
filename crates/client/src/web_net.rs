//! Browser networking: a WebSocket transport and fetch helpers, all non-blocking.
//! Logging in with a 3m4 account is the standard relying-app flow: redirect to the
//! hosted login page and come back with the `3m4_session` cookie, which the server
//! accepts on the WebSocket upgrade.

use crate::deferred::Deferred;
use crate::session::Transport;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use wars_proto::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

pub const WEB: bool = true;

struct Shared {
    incoming: VecDeque<Vec<u8>>,
    outgoing: VecDeque<Vec<u8>>,
    open: bool,
    closed: bool,
}

pub struct WebTransport {
    ws: web_sys::WebSocket,
    shared: Rc<RefCell<Shared>>,
    _callbacks: Vec<Closure<dyn FnMut(JsValue)>>,
}

impl WebTransport {
    pub fn connect(ws_url: &str, token: &str) -> Result<WebTransport, String> {
        let ws = web_sys::WebSocket::new(ws_url).map_err(|e| format!("{e:?}"))?;
        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);
        let shared = Rc::new(RefCell::new(Shared {
            incoming: VecDeque::new(),
            outgoing: VecDeque::new(),
            open: false,
            closed: false,
        }));
        let mut callbacks = Vec::new();
        {
            let sh = shared.clone();
            let ws2 = ws.clone();
            let on_open = Closure::wrap(Box::new(move |_e: JsValue| {
                let mut s = sh.borrow_mut();
                s.open = true;
                while let Some(b) = s.outgoing.pop_front() {
                    let _ = ws2.send_with_u8_array(&b);
                }
            }) as Box<dyn FnMut(JsValue)>);
            ws.set_onopen(Some(on_open.as_ref().unchecked_ref()));
            callbacks.push(on_open);
        }
        {
            let sh = shared.clone();
            let on_msg = Closure::wrap(Box::new(move |e: JsValue| {
                if let Ok(me) = e.dyn_into::<web_sys::MessageEvent>() {
                    if let Ok(buf) = me.data().dyn_into::<js_sys::ArrayBuffer>() {
                        let arr = js_sys::Uint8Array::new(&buf);
                        sh.borrow_mut().incoming.push_back(arr.to_vec());
                    }
                }
            }) as Box<dyn FnMut(JsValue)>);
            ws.set_onmessage(Some(on_msg.as_ref().unchecked_ref()));
            callbacks.push(on_msg);
        }
        {
            let sh = shared.clone();
            let on_close = Closure::wrap(Box::new(move |_e: JsValue| {
                sh.borrow_mut().closed = true;
            }) as Box<dyn FnMut(JsValue)>);
            ws.set_onclose(Some(on_close.as_ref().unchecked_ref()));
            ws.set_onerror(Some(on_close.as_ref().unchecked_ref()));
            callbacks.push(on_close);
        }
        let mut t = WebTransport {
            ws,
            shared,
            _callbacks: callbacks,
        };
        t.send(encode(&ClientMsg::Hello {
            token: token.to_string(),
            version: PROTOCOL_VERSION,
        }));
        Ok(t)
    }
}

impl Transport for WebTransport {
    fn send(&mut self, bytes: Vec<u8>) {
        let open = self.shared.borrow().open;
        if open {
            let _ = self.ws.send_with_u8_array(&bytes);
        } else {
            self.shared.borrow_mut().outgoing.push_back(bytes);
        }
    }
    fn try_recv(&mut self) -> Option<Vec<u8>> {
        self.shared.borrow_mut().incoming.pop_front()
    }
    fn is_closed(&self) -> bool {
        self.shared.borrow().closed
    }
}

impl Drop for WebTransport {
    fn drop(&mut self) {
        let _ = self.ws.close();
    }
}

pub fn ws_url(base: &str) -> String {
    let b = base.trim_end_matches('/');
    let b = b
        .replacen("https://", "wss://", 1)
        .replacen("http://", "ws://", 1);
    format!("{b}/ws")
}

/// The page's own origin: the game is served by the same server it talks to.
pub fn same_origin_server() -> Option<String> {
    web_sys::window().and_then(|w| w.location().origin().ok())
}

async fn fetch(
    method: &str,
    url: &str,
    body: Option<String>,
    bearer: Option<String>,
) -> Result<(u16, String), String> {
    let opts = web_sys::RequestInit::new();
    opts.set_method(method);
    opts.set_credentials(web_sys::RequestCredentials::Include);
    if let Some(b) = body {
        opts.set_body(&JsValue::from_str(&b));
    }
    let req = web_sys::Request::new_with_str_and_init(url, &opts).map_err(|e| format!("{e:?}"))?;
    let headers = req.headers();
    let _ = headers.set("Content-Type", "application/json");
    if let Some(t) = bearer {
        if !t.is_empty() {
            let _ = headers.set("Authorization", &format!("Bearer {t}"));
        }
    }
    let window = web_sys::window().ok_or("no window")?;
    let resp = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&req))
        .await
        .map_err(|e| format!("{e:?}"))?;
    let resp: web_sys::Response = resp.dyn_into().map_err(|_| "bad response")?;
    let status = resp.status();
    let text = wasm_bindgen_futures::JsFuture::from(resp.text().map_err(|e| format!("{e:?}"))?)
        .await
        .map_err(|e| format!("{e:?}"))?;
    Ok((status, text.as_string().unwrap_or_default()))
}

fn ok_json<T: serde::de::DeserializeOwned>(r: Result<(u16, String), String>) -> Result<T, String> {
    let (status, text) = r?;
    if status >= 400 {
        return Err(format!(
            "HTTP {status}: {}",
            text.chars().take(200).collect::<String>()
        ));
    }
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

pub fn connect_deferred(
    ws_url: String,
    token: String,
) -> Deferred<Result<Box<dyn Transport>, String>> {
    Deferred::ready(
        WebTransport::connect(&ws_url, &token).map(|t| Box::new(t) as Box<dyn Transport>),
    )
}
pub fn guest_token_deferred(
    server_url: String,
    nickname: String,
) -> Deferred<Result<TokenResponse, String>> {
    Deferred::spawn_local(async move {
        let body = serde_json::json!({ "nickname": nickname }).to_string();
        ok_json(
            fetch(
                "POST",
                &format!("{}/api/guest", server_url.trim_end_matches('/')),
                Some(body),
                None,
            )
            .await,
        )
    })
}
/// Not available in the browser (passwords never go through the page); use `login_redirect`.
pub fn login_deferred(
    _accounts_url: String,
    _email: String,
    _password: String,
) -> Deferred<Result<(String, String), String>> {
    Deferred::ready(Err(
        "Use the Log in button: it opens the 3m4 login page".into()
    ))
}
pub fn refresh_deferred(
    _accounts_url: String,
    _refresh: String,
) -> Deferred<Result<(String, String), String>> {
    Deferred::ready(Err("not used in the browser".into()))
}
pub fn lobbies_deferred(
    server_url: String,
    token: String,
) -> Deferred<Result<Vec<LobbySummary>, String>> {
    Deferred::spawn_local(async move {
        ok_json(
            fetch(
                "GET",
                &format!("{}/api/lobbies", server_url.trim_end_matches('/')),
                None,
                Some(token),
            )
            .await,
        )
    })
}
/// The cookie session, if the browser has one for this origin.
pub fn me_deferred(
    server_url: String,
    token: String,
) -> Deferred<Result<Option<PlayerInfo>, String>> {
    Deferred::spawn_local(async move {
        #[derive(serde::Deserialize)]
        struct Me {
            player: PlayerInfo,
        }
        match fetch(
            "GET",
            &format!("{}/api/me", server_url.trim_end_matches('/')),
            None,
            Some(token),
        )
        .await
        {
            Ok((401, _)) => Ok(None),
            other => ok_json::<Me>(other).map(|m| Some(m.player)),
        }
    })
}
/// Send the browser to the hosted login page; it returns to `return_to` with the cookie set.
pub fn login_redirect(accounts_url: &str, return_to: &str) {
    if let Some(w) = web_sys::window() {
        let url = format!(
            "{}/login?return_to={}",
            accounts_url.trim_end_matches('/'),
            js_sys::encode_uri_component(return_to)
        );
        let _ = w.location().set_href(&url);
    }
}
