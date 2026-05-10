#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OCI_URL="${OCI_URL:-http://127.0.0.1:8081}"
MEILI_MASTER_KEY="${MEILI_MASTER_KEY:-MASTER_KEY}"
INDEX_UID="${INDEX_UID:-movies}"
FIXTURE="${FIXTURE:-$ROOT_DIR/fixtures/documents.json}"

auth_header="Authorization: Bearer $MEILI_MASTER_KEY"

echo "Checking OCI Meilisearch health at $OCI_URL/health"
curl -fsS "$OCI_URL/health"
echo

echo "Loading fixture into OCI Meilisearch index '$INDEX_UID'"
task_response="$(
  curl -fsS \
    -X POST "$OCI_URL/indexes/$INDEX_UID/documents" \
    -H "$auth_header" \
    -H "content-type: application/json" \
    --data-binary "@$FIXTURE"
)"
echo "$task_response"

task_uid="$(
  printf '%s' "$task_response" | python3 -c 'import json, sys; print(json.load(sys.stdin).get("taskUid", ""))'
)"

if [ -n "$task_uid" ]; then
  echo "Waiting for OCI task $task_uid"
  for _ in $(seq 1 40); do
    task_status="$(
      curl -fsS \
        -H "$auth_header" \
        "$OCI_URL/tasks/$task_uid" \
        | python3 -c 'import json, sys; print(json.load(sys.stdin).get("status", ""))'
    )"

    if [ "$task_status" = "succeeded" ]; then
      break
    fi

    if [ "$task_status" = "failed" ]; then
      echo "Task $task_uid failed" >&2
      exit 1
    fi

    sleep 0.25
  done
fi

echo "Searching OCI Meilisearch index '$INDEX_UID' for 'space'"
curl -fsS \
  -X POST "$OCI_URL/indexes/$INDEX_UID/search" \
  -H "$auth_header" \
  -H "content-type: application/json" \
  --data '{"q":"space","limit":3}'
echo
