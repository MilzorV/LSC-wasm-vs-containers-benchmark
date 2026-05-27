#!/usr/bin/env bash
set -euo pipefail

SPIN_URL="${SPIN_URL:-http://127.0.0.1:8080}"

echo "Checking Spin movie-search health at $SPIN_URL/health"
curl -fsS "$SPIN_URL/health"
echo

echo "Checking Spin dashboard at $SPIN_URL/"
curl -fsS "$SPIN_URL/" | python3 -c '
import sys
body = sys.stdin.read()
assert "movie-search-dashboard" in body, "dashboard marker missing"
print("dashboard ok")
'
echo

echo "Checking Spin movie-search version"
curl -fsS "$SPIN_URL/version"
echo

echo "Checking Spin movie-search stats"
stats_response="$(curl -fsS "$SPIN_URL/stats")"
echo "$stats_response"
printf '%s' "$stats_response" | python3 -c '
import json, sys
stats = json.load(sys.stdin)
assert stats["documentCount"] == 44471, stats
'

echo "Searching Spin movie-search for 'space'"
search_response="$(
  curl -fsS \
    -X POST "$SPIN_URL/search" \
    -H "content-type: application/json" \
    --data '{"q":"space","limit":3}'
)"
echo "$search_response"
printf '%s' "$search_response" | python3 -c '
import json, sys
response = json.load(sys.stdin)
assert response["query"] == "space", response
assert len(response["hits"]) == 3, response
print("hit ids:", ",".join(str(hit["id"]) for hit in response["hits"]))
'

echo "Listing Spin movies"
curl -fsS "$SPIN_URL/movies?limit=2"
echo
