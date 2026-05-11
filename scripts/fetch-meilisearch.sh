#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TAG="${MEILI_TAG:-v1.43.0}"
UPSTREAM_URL="${MEILI_UPSTREAM_URL:-https://github.com/meilisearch/meilisearch.git}"
TARGET_DIR="$ROOT_DIR/vendor/meilisearch"

if [ -d "$TARGET_DIR/.git" ]; then
  echo "Upstream Meilisearch already fetched at $TARGET_DIR"
  git -C "$TARGET_DIR" fetch --tags --depth 1 origin "$TAG"
  git -C "$TARGET_DIR" checkout --detach "$TAG"
else
  mkdir -p "$(dirname "$TARGET_DIR")"
  git clone --depth 1 --branch "$TAG" "$UPSTREAM_URL" "$TARGET_DIR"
fi

commit="$(git -C "$TARGET_DIR" rev-parse HEAD)"
echo "Fetched Meilisearch $TAG at $commit"
