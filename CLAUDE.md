# wars — notes for agents

- Spec: `docs/CONTRACT.md` (protocol, REST, data model) and `docs/DESIGN.md` (game rules and numbers). Keep both in sync with the code.
- Research behind the design lives in `docs/research/`. Never use the "Age of Empires" trademark in code, UI strings or marketing copy; civilisation names are historical and fine.
- Everything in this repo (code, comments, commits, docs) is written in English.
- Cargo workspace; keep `cargo build -j 8` (the dev box has 14 GB RAM). `cargo test --workspace` must stay green.
- The `sim` crate is deterministic: no floats, no HashMap iteration, one seeded RNG. Every surface (server, desktop, tui, android, ios) runs the same sim through `client`.
- Identity: accounts.3m4.net (see `/home/enrique/accounts/CONTRACT.md`). Guests get a wars-issued token.
- Local server: `make up` → http://localhost:8086. Compose project `wars`. Brand colour: Tailwind **amber**.
- Build mode: Claude writes the whole codebase unless Enrique says "modo aprender".
- PR per feature; UI PRs include a GIF + webm demo.
