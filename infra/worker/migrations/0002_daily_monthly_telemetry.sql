ALTER TABLE pings ADD COLUMN monthly_key TEXT;
ALTER TABLE pings ADD COLUMN identity_kind TEXT;
ALTER TABLE pings ADD COLUMN build TEXT;

DROP VIEW IF EXISTS recent_pings;
DROP VIEW IF EXISTS os_arch_breakdown;
DROP VIEW IF EXISTS platform_breakdown;
DROP VIEW IF EXISTS version_adoption;
DROP VIEW IF EXISTS monthly_usage;
DROP VIEW IF EXISTS monthly_install_latest;
DROP VIEW IF EXISTS daily_usage;
DROP VIEW IF EXISTS release_pings;

CREATE VIEW release_pings AS
SELECT
  id,
  ts,
  day,
  unique_key,
  monthly_key,
  COALESCE(NULLIF(identity_kind, ''), 'legacy') AS identity_kind,
  COALESCE(NULLIF(channel, ''), 'unknown') AS channel,
  COALESCE(NULLIF(platform, ''), 'unknown') AS platform,
  COALESCE(NULLIF(os, ''), 'unknown') AS os,
  COALESCE(NULLIF(os_version, ''), 'unknown') AS os_version,
  COALESCE(NULLIF(arch, ''), 'unknown') AS arch,
  version,
  COALESCE(NULLIF(build, ''), 'unknown') AS build,
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
  COUNT(DISTINCT unique_key) AS active_installs
FROM release_pings
GROUP BY day, channel, platform;

CREATE VIEW monthly_install_latest AS
SELECT
  id,
  ts,
  day,
  month,
  unique_key,
  monthly_key,
  identity_kind,
  channel,
  platform,
  os,
  os_version,
  arch,
  version,
  build,
  model
FROM (
  SELECT
    release_pings.*,
    strftime('%Y-%m', ts, 'unixepoch') AS month,
    ROW_NUMBER() OVER (
      PARTITION BY strftime('%Y-%m', ts, 'unixepoch'), channel, monthly_key
      ORDER BY ts DESC, id DESC
    ) AS row_number
  FROM release_pings
  WHERE monthly_key IS NOT NULL AND monthly_key <> ''
)
WHERE row_number = 1;

CREATE VIEW monthly_usage AS
SELECT
  month,
  channel,
  platform,
  COUNT(*) AS active_installs,
  SUM(CASE WHEN identity_kind = 'client' THEN 1 ELSE 0 END) AS exact_client_installs,
  SUM(CASE WHEN identity_kind = 'network' THEN 1 ELSE 0 END) AS network_estimates
FROM monthly_install_latest
GROUP BY month, channel, platform;

CREATE VIEW version_adoption AS
SELECT
  month,
  version,
  build,
  channel,
  platform,
  COUNT(*) AS active_installs,
  SUM(CASE WHEN identity_kind = 'client' THEN 1 ELSE 0 END) AS exact_client_installs,
  SUM(CASE WHEN identity_kind = 'network' THEN 1 ELSE 0 END) AS network_estimates
FROM monthly_install_latest
GROUP BY month, version, build, channel, platform;

CREATE VIEW platform_breakdown AS
SELECT
  month,
  channel,
  platform,
  COUNT(*) AS active_installs,
  SUM(CASE WHEN identity_kind = 'client' THEN 1 ELSE 0 END) AS exact_client_installs,
  SUM(CASE WHEN identity_kind = 'network' THEN 1 ELSE 0 END) AS network_estimates
FROM monthly_install_latest
WHERE month = strftime('%Y-%m', 'now')
GROUP BY month, channel, platform;

CREATE VIEW os_arch_breakdown AS
SELECT
  month,
  channel,
  os,
  os_version,
  arch,
  COUNT(*) AS active_installs,
  SUM(CASE WHEN identity_kind = 'client' THEN 1 ELSE 0 END) AS exact_client_installs,
  SUM(CASE WHEN identity_kind = 'network' THEN 1 ELSE 0 END) AS network_estimates
FROM monthly_install_latest
WHERE month = strftime('%Y-%m', 'now')
GROUP BY month, channel, os, os_version, arch;

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
  build,
  identity_kind
FROM release_pings;
