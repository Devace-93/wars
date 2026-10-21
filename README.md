# Wars

**An open-source real-time strategy game in the spirit of the classic age-advancing RTS.**
Advance from the Dark Age to the Imperial Age, gather food, wood, gold and stone, raise a
town, research technologies and lead one of thirteen historical civilisations against
other players or the computer.

Built by Enrique Magallón. MIT. Hosted edition at `wars.3m4.net`; the server is free to self-host.

| Surface | Where | Status |
|---|---|---|
| Server | `crates/server` (Rust, axum, Postgres) | building |
| Desktop (Linux / Windows / macOS) | `crates/desktop` (winit + wgpu, in-house UI) | building |
| Console | `crates/tui` (ratatui) | building |
| Android | `apps/android` (Kotlin + Compose over the Rust core) | building |
| iOS | `apps/ios` (SwiftUI over the Rust core, built on GitHub Actions) | building |

One deterministic simulation (`crates/sim`) runs everywhere; clients play in lockstep
through the server or fully offline against the AI.

See `docs/DESIGN.md` for the rules and `docs/CONTRACT.md` for the protocol.
