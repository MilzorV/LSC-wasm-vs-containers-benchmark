#!/usr/bin/env bash
set -euo pipefail

SPIN_URL="${SPIN_URL:-http://127.0.0.1:8080}"
OCI_URL="${OCI_URL:-http://127.0.0.1:8081}"

python3 - "$SPIN_URL" "$OCI_URL" <<'PY'
import json
import sys
import urllib.request

spin_url, oci_url = sys.argv[1], sys.argv[2]
queries = ["space", "toy story", "dark knight", "romance", ""]


def post_json(base_url, path, payload):
    data = json.dumps(payload).encode()
    request = urllib.request.Request(
        f"{base_url}{path}",
        data=data,
        headers={"content-type": "application/json"},
        method="POST",
    )
    with urllib.request.urlopen(request, timeout=30) as response:
        return json.load(response)


def get_json(base_url, path):
    with urllib.request.urlopen(f"{base_url}{path}", timeout=30) as response:
        return json.load(response)


def stable_search_shape(response):
    return {
        "query": response["query"],
        "offset": response["offset"],
        "limit": response["limit"],
        "estimatedTotalHits": response["estimatedTotalHits"],
        "hitIds": [hit["id"] for hit in response["hits"]],
    }


spin_version = get_json(spin_url, "/version")
oci_version = get_json(oci_url, "/version")
if spin_version["engine"] != oci_version["engine"]:
    raise SystemExit(f"engine mismatch: spin={spin_version} oci={oci_version}")

spin_stats = get_json(spin_url, "/stats")
oci_stats = get_json(oci_url, "/stats")
if spin_stats != oci_stats:
    raise SystemExit(f"stats mismatch: spin={spin_stats} oci={oci_stats}")

for query in queries:
    payload = {"q": query, "limit": 10}
    spin = stable_search_shape(post_json(spin_url, "/search", payload))
    oci = stable_search_shape(post_json(oci_url, "/search", payload))
    if spin != oci:
        print("Mismatch for query:", repr(query), file=sys.stderr)
        print("Spin:", json.dumps(spin, indent=2), file=sys.stderr)
        print("OCI:", json.dumps(oci, indent=2), file=sys.stderr)
        raise SystemExit(1)
    print(f"{query!r}: {spin['hitIds']}")

print("Spin and OCI movie-search results match.")
PY
