#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
UPSTREAM_DIR="${MEILI_UPSTREAM_DIR:-$ROOT_DIR/vendor/meilisearch}"
OUT_FILE="${MEILI_WASI_REPORT:-$ROOT_DIR/docs/upstream-wasi-blockers.md}"
TARGET="${MEILI_WASI_TARGET:-wasm32-wasip2}"
PACKAGES="${MEILI_WASI_PACKAGES:-flatten-serde-json filter-parser milli meilisearch-types routes meilisearch}"
REQUESTED_TAG="${MEILI_TAG:-v1.43.0}"

if [ ! -d "$UPSTREAM_DIR" ]; then
  echo "Missing upstream checkout at $UPSTREAM_DIR" >&2
  echo "Run scripts/fetch-meilisearch.sh first." >&2
  exit 2
fi

commit="$(git -C "$UPSTREAM_DIR" rev-parse HEAD)"
tag="$(git -C "$UPSTREAM_DIR" describe --tags --exact-match 2>/dev/null || true)"
overall_status=0
summary=""
details=""

for package in $PACKAGES; do
  safe_package="$(printf '%s' "$package" | tr -c 'A-Za-z0-9_.-' '_')"
  log_file="$ROOT_DIR/docs/upstream-wasi-check-$safe_package.log"

  set +e
  (
    cd "$UPSTREAM_DIR"
    cargo check --target "$TARGET" -p "$package"
  ) >"$log_file" 2>&1
  status=$?
  set -e

  if [ "$status" -eq 0 ]; then
    result="PASS"
  else
    result="FAIL"
    overall_status=1
  fi

  summary="$summary| \`$package\` | $result | \`$status\` | \`docs/upstream-wasi-check-$safe_package.log\` |
"

  details="$details
## \`$package\` last 80 log lines

\`\`\`text
$(tail -80 "$log_file")
\`\`\`
"
done

{
  echo "# Upstream Meilisearch WASI Feasibility Blockers"
  echo
  echo "- Date: $(date -u '+%Y-%m-%dT%H:%M:%SZ')"
  echo "- Requested upstream tag: \`$REQUESTED_TAG\`"
  echo "- Git exact-match tag: ${tag:-unknown}"
  echo "- Upstream commit: \`$commit\`"
  echo "- Checked target: \`$TARGET\`"
  echo "- Checked packages: \`$PACKAGES\`"
  echo "- Overall status: \`$overall_status\`"
  echo
  echo "## Summary"
  echo
  echo "| Package | Result | Exit | Log |"
  echo "|---|---:|---:|---|"
  printf '%s' "$summary"
  echo
  echo "## Interpretation"
  echo
  echo "This layered check identifies the highest upstream crates that compile for Spin's \`$TARGET\` target before native-runtime dependencies fail. Failures are expected for the full Meilisearch-on-Spin path because upstream Meilisearch depends on native HTTP/runtime crates, crypto/C dependencies, LMDB/heed, memory-mapped storage, filesystem behavior, and background task scheduling."
  echo
  printf '%s' "$details"
} >"$OUT_FILE"

cat "$OUT_FILE"
exit "$overall_status"
