# Wars — Interface Contract

`wars.3m4.net` is the Wars server: lobbies, authoritative lockstep matches, saves,
scenarios, replays and the ladder. Every client (desktop, console, Android, iOS)
speaks the same protocol. This file is the contract between the server
(`crates/server`), the shared client core (`crates/client`) and the front-ends.
Keep it in sync with the code.

## Hosts and ports

| Where | Host | Notes |
|---|---|---|
| Production | `https://wars.3m4.net` | Routed by the 3m4 edge Caddy on the Hetzner box (`wars-server-1:8086`) |
| Local | `http://localhost:8086` | `make up` (Postgres on host port 5486) |

Containers: `postgres` 5432, `server` 8086 (axum). Compose project name `wars`.
Brand colour: Tailwind **amber**.

## Identity

Identity is `accounts.3m4.net` (see `/home/enrique/accounts/CONTRACT.md`).

- The server accepts the accounts JWT (`3m4_session` cookie **or** `Authorization:
  Bearer <jwt>`), verified locally with the JWKS at
  `ACCOUNTS_INTERNAL_URL/.well-known/jwks.json` (Ed25519, cached 10 min, refetched on
  an unknown `kid`); `iss` = `ACCOUNTS_URL`, `aud` = `3m4`.
- Native clients log in with email + password against
  `POST {ACCOUNTS_URL}/api/auth/login` and keep the `3m4_refresh` cookie to renew the
  15-minute JWT through `POST /api/session/refresh`. OAuth-only accounts cannot log in
  natively yet. Guests: `POST /api/guest {nickname}` returns a 30-day HS256 token
  signed with `GUEST_SIGNING_KEY` (`sub` = `guest:<uuid>`).
- First time a `sub` is seen a `players` row is created from the claims and refreshed
  on every request. Display name: `nickname` (if set) else the accounts `handle`.
- Guests can play and host but cannot store saves or scenarios and never appear on the
  ladder. Verified e-mail is not required (no `403 email_unverified` gate).

Env: `DATABASE_URL`, `ACCOUNTS_URL`, `ACCOUNTS_INTERNAL_URL` (`http://accounts-api-1:5000`
on the `edge` docker network in production), `GUEST_SIGNING_KEY`, `PUBLIC_URL`, `PORT`.

## REST API (`/api`, JSON; errors `{"error": {"code", "message"}}`)

| Method | Path | Auth | Body / query | Response |
|---|---|---|---|---|
| GET | `/healthz`, `/api/healthz` | – | | `{"status":"ok"}` |
| POST | `/guest` | – | `{nickname}` | `{token, player}` |
| GET | `/me` | session | | `{player, nickname, elo, wins, losses, email_verified}` |
| PATCH | `/me` | session | `{nickname?}` | `{player}` |
| GET | `/lobbies` | – | | `[LobbySummary]` (public, not started) |
| GET | `/civs` | – | | civilisation cards |
| GET | `/saves` | session | | `[SaveSummary]` (own) |
| POST | `/saves?name=` | account | body = world snapshot (postcard) | `{id}` |
| GET | `/saves/{id}` | session | | snapshot bytes (own only) |
| DELETE | `/saves/{id}` | session | | 204 |
| GET | `/scenarios` | optional | | `[ScenarioSummary]` (public + own) |
| POST | `/scenarios?name=&public=` | account | body = scenario (postcard) | `{id}` |
| GET | `/scenarios/{id}` | optional | | scenario bytes |
| DELETE | `/scenarios/{id}` | session | | 204 |
| GET | `/leaderboard` | – | | `[LeaderboardEntry]` top 100 by ELO |
| GET | `/matches` | session | | own match history |
| GET | `/replays/{match_id}` | – | | replay bytes (postcard `Replay`) |
| GET | `/ws` | – | WebSocket upgrade | see below |

"account" = non-guest session. Objects are defined in `crates/proto/src/lib.rs`.

## WebSocket protocol (`/ws`)

Binary frames, each one `ClientMsg` or `ServerMsg` encoded with
[postcard](https://docs.rs/postcard). `PROTOCOL_VERSION` = 1.

1. The first client frame is `Hello { token, version }` (within 10 s). The server
   answers `Welcome { you, server_version }` or `Error { code }` and closes.
2. Lobby: `CreateLobby` / `JoinLobby { id | room_code }` → `LobbyState(Lobby)` on every
   change. `SetSlot { slot, change }` (civ/team/colour/ready by the seated player; AI,
   open/close, kick, by the host), `SetSource`, `SetSettings` (host), `Chat`.
   `LeaveLobby` → `LobbyClosed`. Room codes are 6 characters.
3. `StartMatch` (host): the server builds the world (random map from the lobby seed,
   a stored scenario or a save), inserts the `matches` row and sends
   `MatchStart { lobby, you, snapshot, tick_rate }` to every member (`you` = sim
   `PlayerId`, `None` for spectators).
4. Lockstep: the server ticks at `tick_rate` (20 Hz × speed). Each tick it takes the
   commands received since the previous tick (plus the AI players' commands), steps its
   own copy of the world and broadcasts `Tick { tick, commands }`. Clients apply the
   batches strictly in order with the same `wars_sim::step::step`, so every
   participant computes the same world. Clients send `Commands { commands }` without a
   tick; the server schedules them on the next batch (this is the command delay, about
   50–100 ms plus RTT/2).
5. Every `HASH_INTERVAL` (20) ticks a client sends `Hash { tick, hash }`. The server
   keeps the last 60 of its own hashes; on a mismatch it answers `Desync { tick }` and
   `Snapshot { tick, snapshot }`, which the client loads, dropping queued batches that
   are already included.
6. Late join / reconnect: joining a started room sends `MatchStart` with a fresh
   snapshot (the same `sub` gets its slot back; others spectate).
7. `Pause(bool)`, `Speed(pct)` (host) → `Paused`, `SpeedChanged`; `SaveGame { name }`
   (account) → `Saved { id, name }`; `RequestSnapshot` → `Snapshot`; `Ping` → `Pong`.
8. The match ends with `MatchEnded { winner_team }`. Results, ELO (K = 32, team
   average) and the replay (`Replay { lobby, seed, commands, ticks, winner_team }`)
   are persisted. If every member leaves, the match is abandoned.

Autosave: every 2 minutes the server stores an `Autosave: <lobby name>` save for each
seated account holder (replacing the previous one).

## Save, scenario and replay files

- Save = `World` snapshot (postcard). Local saves live under the platform data dir
  (`~/.local/share/wars/saves/*.wars` on Linux); online saves in the `saves` table.
- Scenario = `wars_mapgen::scenario::Scenario` (postcard `.scn`, JSON for humans).
- Replay = `Replay` (postcard); a replay is replayed by regenerating the world from
  `lobby` + `seed` and feeding `commands` tick by tick.

## Data model (Postgres, `crates/server/migrations`)

```
players        id (accounts sub | guest:<uuid>) pk, handle, name, avatar_url, nickname,
               guest, elo (1000), wins, losses, created_at, updated_at
matches        id uuid pk, name, host_id fk, lobby jsonb, seed, status
               (running|finished|abandoned), winner_team, ticks, started_at, ended_at
match_players  (match_id, slot) pk, player_id, sim_player, civ, team, is_ai, result, score
saves          id pk, owner_id fk, name, tick, lobby jsonb, snapshot bytea, created_at
scenarios      id pk, owner_id fk, name, author, description, public, players, size,
               data bytea, created_at, updated_at
replays        match_id pk fk, data bytea, created_at
```

## Determinism rules (every participant)

Fixed-point maths only, one seeded RNG in the world, `BTreeMap`/`Vec` iteration, 20
ticks per second, AI thinks every 20 ticks from the same world state. Any float in
`wars-sim`, `wars-ai` or `wars-mapgen` logic is a bug.
