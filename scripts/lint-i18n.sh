#!/usr/bin/env bash
# Fails on catalog messages no shell uses and on hardcoded prose in shell sources.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

# Product, font, and key names that stay English in every locale.
names="JayJay|JayJay Feedback|Codex CLI|Claude CLI|Codex|Claude|Terminal|Finder|Frog|Homebrew|Seti|Octicons|Segoe Fluent Icons|Segoe MDL2 Assets|Ghostty|VSCodium|Visual Studio Code|Cursor|Zed|Xcode|Vim|Ctrl|Alt|Shift|Super|Enter|Esc|Space|Tab|Return"
# Generated bindings, CLI help, model prompts, and the GitHub issue template stay English.
english=(
  shell/gpui/src/startup/arguments.rs
  shell/mac/Sources/JayJayCore
  shell/mac/Sources/JayJay/Shared/AI/AppleIntelligence.swift
  shell/mac/Sources/JayJay/Shared/HelpBook.swift
)

status=0
camel() { awk -F'[-_]' '{ printf "%s", $1; for (i = 2; i <= NF; i++) printf "%s", toupper(substr($i, 1, 1)) substr($i, 2) }' <<< "$1"; }

used=$( { grep -rhoE --include='*.rs' '[a-z0-9_:]*[a-z_]+::[a-z0-9_]+' crates shell/gpui/src --exclude-dir=jayjay-i18n \
    | sed -n 's/^jayjay_i18n:://; /^[a-z_]*::[a-z0-9_]*$/p'
  grep -rhoE --include='*.swift' 'Localized\.[A-Za-z]+\.[A-Za-z0-9]+' shell/mac/Sources shell/mac/Packages; } | sort -u)
for file in crates/jayjay-i18n/locales/en-US/*.ftl; do
  stem=$(basename "$file" .ftl)
  swift_enum=$(camel "$stem")
  swift_enum=$(tr a-z A-Z <<< "${swift_enum:0:1}")${swift_enum:1}
  for id in $(sed -n "s/^\(${stem}-[a-z0-9-]*\) =.*/\1/p" "$file"); do
    name=${id#"$stem"-}
    if ! grep -qxF -e "${stem}::${name//-/_}" -e "Localized.${swift_enum}.$(camel "$name")" <<< "$used"; then
      echo "unused message: $id ($file)"
      status=1
    fi
  done
done

excluded=$(IFS='|'; echo "${english[*]}")
hits=$(find shell/gpui/src shell/mac/Sources shell/mac/Packages/*/Sources -type f \( -name '*.rs' -o -name '*.swift' \) ! -name 'tests.rs' ! -path '*/tests/*' -print0 \
  | grep -zv -E "^($excluded)" \
  | xargs -0 awk -v names="^\"($names)\"$" '
    FNR == 1 { skip = 0 }
    /#\[cfg\(test\)\]/ { skip = 1 }
    skip || /^[[:space:]]*\/\// || /key_context|KeyContext|assert|precondition|fatalError|\.contains\(|search: |=\\"|[=!]= *"|\.join\(|#\[/ { next }
    {
      rest = $0
      while (match(rest, /"([A-Z][a-z]+([ ,.…?!&:'\''][^"]*)?|\([a-z]+( [a-z]+)*\)|(\\\([^)"]*\)|\{[^}"]*\}) [A-Za-z][a-z]+[^"]*)"/)) {
        literal = substr(rest, RSTART, RLENGTH)
        if (literal !~ names) { print FILENAME ":" FNR ": " $0; break }
        rest = substr(rest, RSTART + RLENGTH)
      }
    }')
if [ -n "$hits" ]; then
  echo "$hits"
  echo "hardcoded user-facing text; add it to crates/jayjay-i18n/locales/en-US"
  status=1
fi
exit $status
