#!/usr/bin/env bash
set -euo pipefail

worker_dir="$1"
database="$2"
query="$3"
value="${4:-}"

parameter=limit
case "$query" in
  dau)
    parameter=days
    sql="
      SELECT date, channel, platform, active_installs, pings
      FROM daily_usage
      WHERE day >= (unixepoch() / 86400) - $value
      ORDER BY day DESC, channel, platform
      LIMIT 200"
    ;;
  mau)
    parameter=months
    sql="
      SELECT month, channel, platform, active_installs, exact_client_installs, network_estimates
      FROM monthly_usage
      WHERE month >= strftime('%Y-%m', 'now', '-' || $value || ' months')
      ORDER BY month DESC, active_installs DESC, channel, platform
      LIMIT 200"
    ;;
  versions)
    sql="
      SELECT month, version, build, channel, platform, active_installs, exact_client_installs, network_estimates
      FROM version_adoption
      WHERE month = strftime('%Y-%m', 'now')
      ORDER BY active_installs DESC, version, build
      LIMIT $value"
    ;;
  platforms)
    parameter=
    sql="
      SELECT month, channel, platform, active_installs, exact_client_installs, network_estimates
      FROM platform_breakdown
      ORDER BY active_installs DESC, channel, platform"
    ;;
  systems)
    sql="
      SELECT month, channel, os, os_version, arch, active_installs, exact_client_installs, network_estimates
      FROM os_arch_breakdown
      ORDER BY active_installs DESC, os, os_version, arch
      LIMIT $value"
    ;;
  recent)
    sql="
      SELECT seen_at, channel, platform, os, os_version, arch, version, build, identity_kind
      FROM recent_pings
      ORDER BY seen_at DESC
      LIMIT $value"
    ;;
  *) echo "usage: $0 <worker-dir> <database> {dau|mau|versions|platforms|systems|recent} [count]" >&2; exit 2 ;;
esac
if [[ -n "$parameter" ]]; then
  case "$value" in ''|*[!0-9]*) echo "$parameter must be a non-negative integer" >&2; exit 2 ;; esac
fi
cd "$worker_dir"
wrangler d1 execute "$database" --remote --command "$sql"
