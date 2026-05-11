#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION="${MEILI_DASHBOARD_VERSION:-v0.4.1}"
SHA1="${MEILI_DASHBOARD_SHA1:-c4de9bfd4bd4ddb87a62cbe00a4150bdf0b6f9d1}"
URL="${MEILI_DASHBOARD_URL:-https://github.com/meilisearch/mini-dashboard/releases/download/$VERSION/build.zip}"
TARGET_DIR="$ROOT_DIR/spin-meili/static/mini-dashboard"
TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/meili-dashboard.XXXXXX")"

cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

curl -fsSL "$URL" -o "$TMP_DIR/build.zip"
actual_sha1="$(shasum -a 1 "$TMP_DIR/build.zip" | awk '{print $1}')"
if [ "$actual_sha1" != "$SHA1" ]; then
  echo "mini-dashboard checksum mismatch" >&2
  echo "expected: $SHA1" >&2
  echo "actual:   $actual_sha1" >&2
  exit 1
fi

rm -rf "$TARGET_DIR"
mkdir -p "$TARGET_DIR"
unzip -q "$TMP_DIR/build.zip" -d "$TARGET_DIR"
echo "Fetched Meilisearch mini-dashboard $VERSION into $TARGET_DIR"
