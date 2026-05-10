#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SPIN_URL="${SPIN_URL:-http://127.0.0.1:8080}"
INDEX_UID="${INDEX_UID:-movies}"
FIXTURE="${FIXTURE:-$ROOT_DIR/fixtures/documents.json}"

echo "Checking Spin health at $SPIN_URL/health"
curl -fsS "$SPIN_URL/health"
echo

echo "Loading fixture into Spin index '$INDEX_UID'"
curl -fsS \
  -X POST "$SPIN_URL/indexes/$INDEX_UID/documents" \
  -H "content-type: application/json" \
  --data-binary "@$FIXTURE"
echo

echo "Searching Spin index '$INDEX_UID' for 'space'"
curl -fsS \
  -X POST "$SPIN_URL/indexes/$INDEX_UID/search" \
  -H "content-type: application/json" \
  --data '{"q":"space","limit":3}'
echo
