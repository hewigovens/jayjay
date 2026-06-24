-- D1 schema for jayjay_stats. Apply with:
--   just worker::apply-schema
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

DROP VIEW IF EXISTS daily_usage;
DROP VIEW IF EXISTS version_adoption;
DROP VIEW IF EXISTS platform_breakdown;
DROP VIEW IF EXISTS os_arch_breakdown;
DROP VIEW IF EXISTS recent_pings;
DROP VIEW IF EXISTS release_pings;

-- Read-only stats views for Wrangler queries. Aggregates intentionally ignore
-- probe/test/dev rows and rows without a release-looking version.
CREATE VIEW release_pings AS
SELECT
  ts,
  day,
  unique_key,
  COALESCE(NULLIF(channel, ''), 'unknown') AS channel,
  COALESCE(NULLIF(platform, ''), 'unknown') AS platform,
  COALESCE(NULLIF(os, ''), 'unknown') AS os,
  COALESCE(NULLIF(os_version, ''), 'unknown') AS os_version,
  COALESCE(NULLIF(arch, ''), 'unknown') AS arch,
  version,
  COALESCE(NULLIF(model, ''), '') AS model
FROM pings
WHERE version IS NOT NULL
  AND version GLOB '[0-9]*.[0-9]*.[0-9]*'
  AND version NOT GLOB '*[^0-9.]*'
  AND length(version) - length(replace(version, '.', '')) = 2;

CREATE VIEW daily_usage AS
SELECT
  day,
  date(day * 86400, 'unixepoch') AS date,
  channel,
  platform,
  COUNT(*) AS pings,
  COUNT(DISTINCT unique_key) AS unique_users
FROM release_pings
GROUP BY day, channel, platform;

CREATE VIEW version_adoption AS
SELECT
  version,
  channel,
  platform,
  COUNT(*) AS pings,
  COUNT(DISTINCT unique_key) AS unique_users,
  date(MIN(ts), 'unixepoch') AS first_seen,
  date(MAX(ts), 'unixepoch') AS last_seen
FROM release_pings
WHERE day >= (unixepoch() / 86400) - 30
GROUP BY version, channel, platform;

CREATE VIEW platform_breakdown AS
SELECT
  channel,
  platform,
  COUNT(*) AS pings,
  COUNT(DISTINCT unique_key) AS unique_users
FROM release_pings
WHERE day >= (unixepoch() / 86400) - 30
GROUP BY channel, platform;

CREATE VIEW os_arch_breakdown AS
SELECT
  channel,
  os,
  os_version,
  arch,
  COUNT(*) AS pings,
  COUNT(DISTINCT unique_key) AS unique_users
FROM release_pings
WHERE day >= (unixepoch() / 86400) - 30
GROUP BY channel, os, os_version, arch;

CREATE VIEW recent_pings AS
SELECT
  datetime(ts, 'unixepoch') AS seen_at,
  date(day * 86400, 'unixepoch') AS date,
  channel,
  platform,
  os,
  os_version,
  arch,
  version,
  model
FROM release_pings;

-- Daily active users by channel/os/arch/version:
--   SELECT day, channel, os, arch, version, COUNT(DISTINCT unique_key) AS dau
--   FROM pings GROUP BY day, channel, os, arch, version ORDER BY day DESC;
-- Retention (keep ~1 year):  DELETE FROM pings WHERE day < (unixepoch()/86400) - 365;
