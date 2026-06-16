-- D1 schema for jayjay_stats. Apply with:
--   wrangler d1 execute jayjay_stats --remote --file=schema.sql
CREATE TABLE IF NOT EXISTS pings (
  id         INTEGER PRIMARY KEY,
  ts         INTEGER NOT NULL DEFAULT (unixepoch()), -- insert time (seconds)
  day        INTEGER NOT NULL,                        -- days since epoch (unique bucket)
  unique_key TEXT    NOT NULL,                        -- salted hash(ip+day); never an IP
  channel    TEXT,                                    -- "swiftui" | "gpui"
  platform   TEXT,
  os         TEXT,
  os_version TEXT,
  arch       TEXT,
  version    TEXT,
  model      TEXT
);

CREATE INDEX IF NOT EXISTS idx_pings_day ON pings (day);

-- Daily active users by channel/os/arch/version:
--   SELECT day, channel, os, arch, version, COUNT(DISTINCT unique_key) AS dau
--   FROM pings GROUP BY day, channel, os, arch, version ORDER BY day DESC;
-- Retention (keep ~1 year):  DELETE FROM pings WHERE day < (unixepoch()/86400) - 365;
