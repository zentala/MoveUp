-- 0001_init.sql — credential storage for the MoveUp relay (E022).
--
-- Hashes and metadata only. No ergonomics data ever reaches Cloudflare: the
-- newest snapshot lives in the Durable Object's memory and dies with it
-- (PLAN.md §Architecture, docs/PRIVACY.md).
--
-- Written by T02 so the shape is decided once; T03 owns everything that reads
-- or writes these tables, and may add indexes and columns here.

CREATE TABLE IF NOT EXISTS licenses (
  -- sha256(license_key), hex. The plaintext key exists only on the customer's
  -- machine and in the output of scripts/mint-license.mjs.
  key_hash   TEXT PRIMARY KEY,
  plan       TEXT    NOT NULL,
  max_desks  INTEGER NOT NULL DEFAULT 1,
  max_viewers INTEGER NOT NULL DEFAULT 5,
  -- Unix ms, or NULL for a licence that does not expire.
  expires_at INTEGER,
  created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS desks (
  desk_id          TEXT PRIMARY KEY,
  license_key_hash TEXT NOT NULL REFERENCES licenses(key_hash) ON DELETE CASCADE,
  -- sha256(desk_token), hex; compared in constant time, never selected by value.
  token_hash       TEXT NOT NULL,
  desk_name        TEXT NOT NULL,
  app_version      TEXT NOT NULL,
  created_at       INTEGER NOT NULL,
  last_seen        INTEGER
);

CREATE INDEX IF NOT EXISTS desks_by_license ON desks(license_key_hash);

CREATE TABLE IF NOT EXISTS viewers (
  viewer_id   TEXT PRIMARY KEY,
  desk_id     TEXT NOT NULL REFERENCES desks(desk_id) ON DELETE CASCADE,
  -- sha256(viewer_token), hex.
  token_hash  TEXT NOT NULL,
  device_name TEXT NOT NULL,
  paired_at   INTEGER NOT NULL,
  last_seen   INTEGER,
  -- Unix ms of revocation. A revoked row is kept so the Settings list can show
  -- what was removed and when; NULL means active.
  revoked_at  INTEGER
);

CREATE INDEX IF NOT EXISTS viewers_by_desk ON viewers(desk_id);
