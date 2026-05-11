#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
UPSTREAM_DIR="${MEILI_UPSTREAM_DIR:-$ROOT_DIR/vendor/meilisearch}"
LOG_FILE="${MEILI_NATIVE_LOG:-$ROOT_DIR/docs/upstream-native-check.log}"
PACKAGE="${MEILI_NATIVE_PACKAGE:-meilisearch}"

if [ ! -d "$UPSTREAM_DIR" ]; then
  echo "Missing upstream checkout at $UPSTREAM_DIR" >&2
  echo "Run scripts/fetch-meilisearch.sh first." >&2
  exit 2
fi

{
  echo "# Native upstream check"
  echo "Date: $(date -u '+%Y-%m-%dT%H:%M:%SZ')"
  echo "Upstream dir: $UPSTREAM_DIR"
  echo "Commit: $(git -C "$UPSTREAM_DIR" rev-parse HEAD)"
  echo "Package: $PACKAGE"
  echo
  cd "$UPSTREAM_DIR"
  cargo check -p "$PACKAGE"
} 2>&1 | tee "$LOG_FILE"
