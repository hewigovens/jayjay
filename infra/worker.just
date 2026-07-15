set shell := ["bash", "-euo", "pipefail", "-c"]
set positional-arguments := true

root := justfile_directory()
worker_dir := root / "infra" / "worker"
database := "jayjay_stats"

list:
    @echo "worker recipes:"
    @echo "  just worker::check-migrations Verify migration names and immutable checksums"
    @echo "  just worker::apply-schema     Apply pending migrations to remote D1"
    @echo "  just worker::deploy           Verify, migrate, verify remote D1, then deploy"
    @echo "  just worker::dau [days]       DAU and ping counts, default 30 days"
    @echo "  just worker::mau [months]     Monthly active installs, default 6 months"
    @echo "  just worker::versions [limit] Current-month version/build adoption, default top 20"
    @echo "  just worker::platforms        Current-month platform split"
    @echo "  just worker::systems [limit]  Current-month OS/arch split, default top 30"
    @echo "  just worker::recent [limit]   Recent telemetry rows, default 20"

check-migrations:
    cd "{{ worker_dir }}" && python3 scripts/verify_migrations.py

apply-schema: check-migrations
    cd "{{ worker_dir }}" && wrangler d1 migrations apply "{{ database }}" --remote

deploy: apply-schema
    cd "{{ worker_dir }}" && python3 scripts/verify_migrations.py --remote --database "{{ database }}"
    cd "{{ worker_dir }}" && wrangler deploy

dau days='30':
    #!/usr/bin/env bash
    set -euo pipefail
    days="{{ days }}"
    case "$days" in ''|*[!0-9]*) echo "days must be a non-negative integer" >&2; exit 2;; esac
    cd "{{ worker_dir }}"
    wrangler d1 execute "{{ database }}" --remote --command "
      SELECT date, channel, platform, active_installs, pings
      FROM daily_usage
      WHERE day >= (unixepoch() / 86400) - $days
      ORDER BY day DESC, channel, platform
      LIMIT 200"

mau months='6':
    #!/usr/bin/env bash
    set -euo pipefail
    months="{{ months }}"
    case "$months" in ''|*[!0-9]*) echo "months must be a non-negative integer" >&2; exit 2;; esac
    cd "{{ worker_dir }}"
    wrangler d1 execute "{{ database }}" --remote --command "
      SELECT month, channel, platform, active_installs, exact_client_installs, network_estimates
      FROM monthly_usage
      WHERE month >= strftime('%Y-%m', 'now', '-' || $months || ' months')
      ORDER BY month DESC, active_installs DESC, channel, platform
      LIMIT 200"

versions limit='20':
    #!/usr/bin/env bash
    set -euo pipefail
    limit="{{ limit }}"
    case "$limit" in ''|*[!0-9]*) echo "limit must be a non-negative integer" >&2; exit 2;; esac
    cd "{{ worker_dir }}"
    wrangler d1 execute "{{ database }}" --remote --command "
      SELECT month, version, build, channel, platform, active_installs, exact_client_installs, network_estimates
      FROM version_adoption
      WHERE month = strftime('%Y-%m', 'now')
      ORDER BY active_installs DESC, version, build
      LIMIT $limit"

platforms:
    #!/usr/bin/env bash
    set -euo pipefail
    cd "{{ worker_dir }}"
    wrangler d1 execute "{{ database }}" --remote --command "
      SELECT month, channel, platform, active_installs, exact_client_installs, network_estimates
      FROM platform_breakdown
      ORDER BY active_installs DESC, channel, platform"

systems limit='30':
    #!/usr/bin/env bash
    set -euo pipefail
    limit="{{ limit }}"
    case "$limit" in ''|*[!0-9]*) echo "limit must be a non-negative integer" >&2; exit 2;; esac
    cd "{{ worker_dir }}"
    wrangler d1 execute "{{ database }}" --remote --command "
      SELECT month, channel, os, os_version, arch, active_installs, exact_client_installs, network_estimates
      FROM os_arch_breakdown
      ORDER BY active_installs DESC, os, os_version, arch
      LIMIT $limit"

recent limit='20':
    #!/usr/bin/env bash
    set -euo pipefail
    limit="{{ limit }}"
    case "$limit" in ''|*[!0-9]*) echo "limit must be a non-negative integer" >&2; exit 2;; esac
    cd "{{ worker_dir }}"
    wrangler d1 execute "{{ database }}" --remote --command "
      SELECT seen_at, channel, platform, os, os_version, arch, version, build, identity_kind
      FROM recent_pings
      ORDER BY seen_at DESC
      LIMIT $limit"
