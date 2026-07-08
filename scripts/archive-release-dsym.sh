#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "usage: $0 /path/to/JayJay.app.dSYM /path/to/JayJay-version.dSYM.zip" >&2
  exit 2
fi

dsym_path="$1"
zip_path="$2"

if [[ ! -d "$dsym_path" ]]; then
  exit 0
fi

mkdir -p "$(dirname "$zip_path")"
ditto -c -k --keepParent "$dsym_path" "$zip_path"
