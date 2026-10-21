-- Players mirror accounts.3m4.net identities (sub) and keep wars-only fields.
CREATE TABLE IF NOT EXISTS players (
    id          TEXT PRIMARY KEY,            -- accounts sub (uuid) or guest:<uuid>
    handle      TEXT NOT NULL,
    name        TEXT,
    avatar_url  TEXT,
    nickname    TEXT,                        -- optional display override
    guest       BOOLEAN NOT NULL DEFAULT FALSE,
    elo         INTEGER NOT NULL DEFAULT 1000,
    wins        INTEGER NOT NULL DEFAULT 0,
    losses      INTEGER NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS matches (
    id          UUID PRIMARY KEY,
    name        TEXT NOT NULL,
    host_id     TEXT REFERENCES players(id),
    lobby       JSONB NOT NULL,              -- wars_proto::Lobby at start
    seed        BIGINT NOT NULL,
    status      TEXT NOT NULL DEFAULT 'running',  -- running | finished | abandoned
    winner_team INTEGER,
    ticks       INTEGER NOT NULL DEFAULT 0,
    started_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    ended_at    TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS match_players (
    match_id    UUID REFERENCES matches(id) ON DELETE CASCADE,
    slot        INTEGER NOT NULL,
    player_id   TEXT REFERENCES players(id),
    sim_player  INTEGER NOT NULL,
    civ         TEXT NOT NULL,
    team        INTEGER NOT NULL,
    is_ai       BOOLEAN NOT NULL DEFAULT FALSE,
    result      TEXT,                        -- won | lost | resigned | left
    score       INTEGER,
    PRIMARY KEY (match_id, slot)
);

CREATE TABLE IF NOT EXISTS saves (
    id          UUID PRIMARY KEY,
    owner_id    TEXT NOT NULL REFERENCES players(id),
    name        TEXT NOT NULL,
    tick        INTEGER NOT NULL,
    lobby       JSONB,                       -- slots at save time (for online resume)
    snapshot    BYTEA NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS saves_owner_idx ON saves(owner_id, created_at DESC);

CREATE TABLE IF NOT EXISTS scenarios (
    id          UUID PRIMARY KEY,
    owner_id    TEXT NOT NULL REFERENCES players(id),
    name        TEXT NOT NULL,
    author      TEXT NOT NULL DEFAULT '',
    description TEXT NOT NULL DEFAULT '',
    public      BOOLEAN NOT NULL DEFAULT FALSE,
    players     INTEGER NOT NULL DEFAULT 2,
    size        INTEGER NOT NULL DEFAULT 96,
    data        BYTEA NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS scenarios_public_idx ON scenarios(public, updated_at DESC);

CREATE TABLE IF NOT EXISTS replays (
    match_id    UUID PRIMARY KEY REFERENCES matches(id) ON DELETE CASCADE,
    data        BYTEA NOT NULL,              -- postcard: Replay { lobby, seed, commands }
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
