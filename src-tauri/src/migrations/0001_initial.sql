-- Initial schema for Llama Recipe Manager.
-- New migrations live in this folder; never edit this one once it has shipped.

CREATE TABLE IF NOT EXISTS recipes (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    command     TEXT NOT NULL,
    model_path  TEXT NOT NULL DEFAULT '',
    mmproj_path TEXT NOT NULL DEFAULT '',
    gpu_info    TEXT NOT NULL DEFAULT '',
    tags        TEXT NOT NULL DEFAULT '',
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
