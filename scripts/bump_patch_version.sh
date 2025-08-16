#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

if [[ ! -f Cargo.toml ]]; then
  echo "Cargo.toml not found; aborting version bump" >&2
  exit 1
fi

current_ver_line=$(grep -E '^[[:space:]]*version[[:space:]]*=' Cargo.toml | head -n1 || true)
if [[ -z "$current_ver_line" ]]; then
  echo "Could not find version in Cargo.toml" >&2
  exit 1
fi

read -r major minor patch < <(echo "$current_ver_line" | sed -E 's/.*"([0-9]+)\.([0-9]+)\.([0-9]+)".*/\1 \2 \3/')
if [[ -z "${major:-}" || -z "${minor:-}" || -z "${patch:-}" ]]; then
  echo "Failed to parse version from Cargo.toml line: $current_ver_line" >&2
  exit 1
fi

new_patch=$((patch + 1))
new_version="${major}.${minor}.${new_patch}"

echo "Bumping Cargo.toml version to ${new_version}" >&2

exit 0

