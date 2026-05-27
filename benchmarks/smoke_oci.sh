#!/usr/bin/env bash
set -euo pipefail

OCI_URL="${OCI_URL:-http://127.0.0.1:8081}"

echo "Checking OCI movie-search health at $OCI_URL/health"
curl -fsS "$OCI_URL/health"
echo

echo "Checking OCI compare UI at $OCI_URL/"
root_body="$(curl -fsS "$OCI_URL/")"
printf '%s' "$root_body" | python3 -c '
import sys
body = sys.stdin.read()
assert "movie-search-compare" in body, "compare app marker missing"
assert "/assets/" in body, "built assets reference missing"
print("compare ui ok")
'
echo

asset_path="$(printf '%s' "$root_body" | python3 -c '
import sys
body = sys.stdin.read()
marker = "href=\"/assets/"
start = body.find(marker)
assert start != -1, "no /assets/ link in index"
rest = body[start + len(marker):]
print(rest.split("\"")[0])
')"
echo "Checking OCI static asset at $OCI_URL/assets/$asset_path"
curl -fsS -o /dev/null -w "HTTP %{http_code}\n" "$OCI_URL/assets/$asset_path"
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
