#!/usr/bin/env bash
set -euo pipefail

OCI_URL="${OCI_URL:-http://127.0.0.1:8081}"

echo "Checking OCI movie-search health at $OCI_URL/health"
curl -fsS "$OCI_URL/health"
echo

echo "Checking OCI app UI at $OCI_URL/oci"
curl -fsS "$OCI_URL/oci" | python3 -c '
import sys
assert "movie-search-app" in sys.stdin.read()
print("oci app ok")
'
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

echo "Checking OCI enhanced search features"
enhanced_response="$(
  curl -fsS \
    -X POST "$OCI_URL/search" \
    -H "content-type: application/json" \
    --data '{"q":"spce","limit":5,"filter":{"genre":["Science Fiction"],"year":{"gte":1970,"lte":2026}},"facets":["genre","year"],"highlight":["title","overview"],"typoTolerance":true,"debugRanking":true}'
)"
echo "$enhanced_response"
printf '%s' "$enhanced_response" | python3 -c '
import json, sys
response = json.load(sys.stdin)
assert response["query"] == "spce", response
assert response["hits"], response
assert "facetDistribution" in response, response
assert "facetStats" in response, response
assert "rankingInfo" in response, response
print("enhanced search ok")
'

echo "Checking OCI suggestions"
suggest_response="$(
  curl -fsS \
    -X POST "$OCI_URL/suggest" \
    -H "content-type: application/json" \
    --data '{"q":"dark kn","limit":5}'
)"
echo "$suggest_response"
printf '%s' "$suggest_response" | python3 -c '
import json, sys
response = json.load(sys.stdin)
assert response["suggestions"], response
print("suggestions:", ", ".join(response["suggestions"]))
'

echo "Listing OCI movies"
curl -fsS "$OCI_URL/movies?limit=2"
echo
