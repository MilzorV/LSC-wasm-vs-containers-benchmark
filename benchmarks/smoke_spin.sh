#!/usr/bin/env bash
set -euo pipefail

SPIN_URL="${SPIN_URL:-http://127.0.0.1:8080}"

echo "Checking Spin movie-search health at $SPIN_URL/health"
curl -fsS "$SPIN_URL/health"
echo

echo "Checking Spin app UI at $SPIN_URL/spin"
curl -fsS "$SPIN_URL/spin" | python3 -c '
import sys
assert "movie-search-app" in sys.stdin.read()
print("spin app ok")
'
echo

echo "Checking Spin compare UI at $SPIN_URL/"
root_body="$(curl -fsS "$SPIN_URL/")"
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
echo "Checking Spin static asset at $SPIN_URL/assets/$asset_path"
curl -fsS -o /dev/null -w "HTTP %{http_code}\n" "$SPIN_URL/assets/$asset_path"
echo

echo "Checking Spin /demo alias"
curl -fsS "$SPIN_URL/demo" | python3 -c '
import sys
assert "movie-search-compare" in sys.stdin.read()
print("demo alias ok")
'
echo

echo "Checking Spin /benchmarks page"
curl -fsS "$SPIN_URL/benchmarks" | python3 -c '
import sys
assert "movie-search-benchmarks" in sys.stdin.read()
print("benchmarks page ok")
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
