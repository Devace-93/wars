//! wars-render: a tiny immediate-mode 2D renderer (solid shapes + text) on wgpu, the
//! isometric camera and the procedural art that draws the world.

pub mod art;
pub mod draw;
pub mod gpu;
pub mod iso;
pub mod text;

pub use draw::{Color, DrawList, Vertex};
pub use gpu::Renderer;
pub use iso::Iso;
pub use text::{Font, TextSystem};
