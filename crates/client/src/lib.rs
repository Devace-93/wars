//! wars-client: everything a client needs that is not rendering.
//!
//! - `Session`: an offline match (local sim + AI) or an online one (lockstep follower).
//! - `Controller`: selection, control groups and command building shared by desktop,
//!   mobile and console front-ends.
//! - `saves`: local save/load of worlds and scenarios.
//! - `net` (feature): blocking WebSocket/HTTP transport and accounts login.

pub mod controller;
#[cfg(feature = "net")]
pub mod net;
pub mod saves;
pub mod session;

pub use controller::Controller;
pub use session::{LocalMatch, RemoteMatch, Session, Transport};
pub use wars_ai;
pub use wars_data;
pub use wars_mapgen;
pub use wars_proto;
pub use wars_sim;
