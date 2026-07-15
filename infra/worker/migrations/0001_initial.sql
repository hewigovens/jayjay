CREATE TABLE IF NOT EXISTS pings (
  id         INTEGER PRIMARY KEY,
  ts         INTEGER NOT NULL DEFAULT (unixepoch()),
  day        INTEGER NOT NULL,
  unique_key TEXT    NOT NULL,
  channel    TEXT,
  platform   TEXT,
  os         TEXT,
  os_version TEXT,
  arch       TEXT,
  version    TEXT,
  model      TEXT
);

CREATE INDEX IF NOT EXISTS idx_pings_day ON pings (day);
