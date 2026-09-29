#!/usr/bin/env bash
set -euo pipefail

[[ $# -eq 2 && ( $1 == bundle || $1 == check ) ]] || { echo "usage: $0 <bundle|check> <AppDir>" >&2; exit 2; }
mode=$1
appdir=$(realpath "$2")
binary=$appdir/usr/bin/jayjay-gpui
lib=$appdir/usr/lib

declare -A excluded
while read -r name; do
  [[ $name == \#* ]] || excluded[$name]=1
done < "$(dirname "$(realpath "$0")")/excludelist"

# aarch64 kernels may use 16K or 64K pages, and patchelf 0.10 aligns the segments it adds to the build machine's 4K.
page_size=4096
[[ $(readelf -h "$binary") == *AArch64* ]] && page_size=65536

check_elf() {
  # Debian 11 and Raspberry Pi OS bullseye ship glibc 2.31, including for bundled libraries.
  local ceiling=2.31 needed type offset vaddr rest
  needed=$(objdump -T "$1" | sed -nE 's/.*GLIBC_([0-9.]+).*/\1/p' | sort -uV | tail -1)
  if [[ "$(printf '%s\n' "${needed:-0}" "$ceiling" | sort -V | tail -1)" != "$ceiling" ]]; then
    echo "error: $1 needs glibc $needed, above the $ceiling ceiling" >&2
    exit 1
  fi
  while read -r type offset vaddr rest; do
    [[ $type == LOAD ]] || continue
    if (( ${rest##* } % page_size || (vaddr - offset) % page_size )); then
      echo "error: $1 has a load segment not aligned to $page_size-byte pages" >&2
      exit 1
    fi
  done < <(readelf -lW "$1")
}

dependencies=$(LC_ALL=C ldd "$binary")
if grep -q '=> not found' <<<"$dependencies"; then
  echo "error: unresolved AppImage dependencies" >&2
  echo "$dependencies" >&2
  exit 1
fi
declare -A resolved
while read -r name arrow path _; do
  [[ $arrow == '=>' ]] && resolved[$name]=$(realpath "$path")
done <<<"$dependencies"

declare -A seen
queue=("$binary")
for ((i = 0; i < ${#queue[@]}; i++)); do
  file=${queue[i]}
  if [[ $mode == check ]]; then
    check_elf "$file"
  fi
  for name in $(readelf -d "$file" | sed -nE 's/.*\(NEEDED\).*\[(.*)\]/\1/p'); do
    case "$name" in
      # GPU and display backends must load from the host's driver stack at runtime.
      libvulkan.so.*|libwayland-*.so.*|libX11*.so.*|libGL*.so.*|libEGL.so.*)
        echo "error: $file links the host GPU or display backend $name" >&2
        exit 1 ;;
      # The excludelist names only the x86 loaders.
      ld-linux-*) continue ;;
    esac
    [[ -n ${excluded[$name]:-} || -n ${seen[$name]:-} ]] && continue
    seen[$name]=1
    if [[ $mode == bundle ]]; then
      install -Dm755 "${resolved[$name]}" "$lib/$name"
      patchelf --page-size "$page_size" --set-rpath "\$ORIGIN" "$lib/$name"
    elif [[ ${resolved[$name]} != "$lib/$name" ]]; then
      echo "error: $name loads from the host instead of the AppImage: ${resolved[$name]}" >&2
      exit 1
    fi
    queue+=("$lib/$name")
  done
done

if [[ $mode == bundle ]]; then
  patchelf --page-size "$page_size" --set-rpath "\$ORIGIN/../lib" "$binary"
  "$0" check "$appdir"
else
  for file in "$lib"/*; do
    if [[ -n ${excluded[$(basename "$file")]:-} ]]; then
      echo "error: $file is on the AppImage excludelist and must come from the host" >&2
      exit 1
    fi
  done
fi
