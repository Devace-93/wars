//! A value that arrives later: produced on a std thread (native) or by a browser
//! future (wasm). The UI polls `try_take` once per frame, so screens never block.

#[cfg(not(target_arch = "wasm32"))]
pub struct Deferred<T> {
    rx: std::sync::mpsc::Receiver<T>,
}

#[cfg(not(target_arch = "wasm32"))]
impl<T: Send + 'static> Deferred<T> {
    /// Run `f` on a background thread.
    pub fn spawn(f: impl FnOnce() -> T + Send + 'static) -> Deferred<T> {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(f());
        });
        Deferred { rx }
    }
    pub fn ready(v: T) -> Deferred<T> {
        let (tx, rx) = std::sync::mpsc::channel();
        let _ = tx.send(v);
        Deferred { rx }
    }
    /// The value, once; `None` while still pending (or after it was taken).
    pub fn try_take(&mut self) -> Option<T> {
        self.rx.try_recv().ok()
    }
}

#[cfg(target_arch = "wasm32")]
pub struct Deferred<T> {
    slot: std::rc::Rc<std::cell::RefCell<Option<T>>>,
}

#[cfg(target_arch = "wasm32")]
impl<T: 'static> Deferred<T> {
    /// Drive a browser future to completion on the current thread.
    pub fn spawn_local(fut: impl std::future::Future<Output = T> + 'static) -> Deferred<T> {
        let slot = std::rc::Rc::new(std::cell::RefCell::new(None));
        let s2 = slot.clone();
        wasm_bindgen_futures::spawn_local(async move {
            *s2.borrow_mut() = Some(fut.await);
        });
        Deferred { slot }
    }
    pub fn ready(v: T) -> Deferred<T> {
        Deferred {
            slot: std::rc::Rc::new(std::cell::RefCell::new(Some(v))),
        }
    }
    pub fn try_take(&mut self) -> Option<T> {
        self.slot.borrow_mut().take()
    }
}
