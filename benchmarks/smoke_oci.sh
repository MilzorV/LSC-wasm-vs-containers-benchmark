#!/usr/bin/env bash
set -euo pipefail

OCI_URL="${OCI_URL:-http://127.0.0.1:8081}"

echo "Checking OCI movie-search health at $OCI_URL/health"
curl -fsS "$OCI_URL/health"
echo

echo "Checking OCI dashboard at $OCI_URL/"
curl -fsS "$OCI_URL/" | python3 -c '
import sys
body = sys.stdin.read()
assert "movie-search-dashboard" in body, "dashboard marker missing"
print("dashboard ok")
'
echo

echo "Checking OCI movie-search version"
curl -fsS "$OCI_URL/version"
echo

echo "Checking OCI movie-search stats"
stats_response="$(curl -fsS "$OCI_URL/stats")"
echo "$stats_response"
printf '%s' "$stats_response" | python3 -c '
import json, sys
stats = json.load(sys.stdin)
assert stats["documentCount"] == 44471, stats
'

echo "Searching OCI movie-search for 'space'"
search_response="$(
  curl -fsS \
    -X POST "$OCI_URL/search" \
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

echo "Listing OCI movies"
curl -fsS "$OCI_URL/movies?limit=2"
echo
