set shell := ["bash", "-euo", "pipefail", "-c"]
set positional-arguments := true

infra := justfile_directory()
worker_dir := infra / "worker"
database := "jayjay_stats"

list:
    @echo "worker recipes:"
    @echo "  just worker::apply-schema     Apply table/index/view schema to remote D1"
    @echo "  just worker::daily [days]     Daily users and pings, default 30 days"
    @echo "  just worker::versions [limit] Version adoption, default top 20"
    @echo "  just worker::platforms        Platform split, last 30 days"
    @echo "  just worker::systems [limit]  OS/arch split, default top 30"
    @echo "  just worker::recent [limit]   Recent telemetry rows, default 20"

apply-schema:
    cd "{{ worker_dir }}" && wrangler d1 execute "{{ database }}" --remote --command="$(<schema.sql)"

daily days='30':
    #!/usr/bin/env bash
    set -euo pipefail
    days="{{ days }}"
    case "$days" in ''|*[!0-9]*) echo "days must be a non-negative integer" >&2; exit 2;; esac
    cd "{{ worker_dir }}"
    wrangler d1 execute "{{ database }}" --remote --command "
      SELECT date, channel, platform, unique_users AS users, pings
      FROM daily_usage
      WHERE day >= (unixepoch() / 86400) - $days
      ORDER BY day DESC, channel, platform
      LIMIT 200"

versions limit='20':
    #!/usr/bin/env bash
    set -euo pipefail
    limit="{{ limit }}"
    case "$limit" in ''|*[!0-9]*) echo "limit must be a non-negative integer" >&2; exit 2;; esac
    cd "{{ worker_dir }}"
    wrangler d1 execute "{{ database }}" --remote --command "
      SELECT version, channel, platform, unique_users AS users, pings, first_seen, last_seen
      FROM version_adoption
      ORDER BY unique_users DESC, pings DESC, version
      LIMIT $limit"

platforms:
    #!/usr/bin/env bash
    set -euo pipefail
    cd "{{ worker_dir }}"
    wrangler d1 execute "{{ database }}" --remote --command "
      SELECT channel, platform, unique_users AS users, pings
      FROM platform_breakdown
      ORDER BY unique_users DESC, pings DESC, channel, platform"

systems limit='30':
    #!/usr/bin/env bash
    set -euo pipefail
    limit="{{ limit }}"
    case "$limit" in ''|*[!0-9]*) echo "limit must be a non-negative integer" >&2; exit 2;; esac
    cd "{{ worker_dir }}"
    wrangler d1 execute "{{ database }}" --remote --command "
      SELECT channel, os, os_version, arch, unique_users AS users, pings
      FROM os_arch_breakdown
      ORDER BY unique_users DESC, pings DESC, os, os_version, arch
      LIMIT $limit"

recent limit='20':
    #!/usr/bin/env bash
    set -euo pipefail
    limit="{{ limit }}"
    case "$limit" in ''|*[!0-9]*) echo "limit must be a non-negative integer" >&2; exit 2;; esac
    cd "{{ worker_dir }}"
    wrangler d1 execute "{{ database }}" --remote --command "
      SELECT seen_at, channel, platform, os, os_version, arch, version, model
      FROM recent_pings
      ORDER BY seen_at DESC
      LIMIT $limit"
