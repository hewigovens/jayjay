#!/usr/bin/env bash
set -euo pipefail

identity="$(security find-identity -v -p codesigning 2>/dev/null | awk '/"Apple Development: /{print $2; exit}' || true)"
codesign --force --sign "${identity:--}" "$@"
