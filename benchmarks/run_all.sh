#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PILOT=0

if [[ "${1:-}" == "--pilot" ]]; then
  PILOT=1
fi

mkdir -p "$ROOT_DIR/results/raw" "$ROOT_DIR/results/processed" "$ROOT_DIR/results/plots"

cleanup() {
  if [[ -n "${SPIN_PID:-}" ]]; then
    kill "$SPIN_PID" >/dev/null 2>&1 || true
    wait "$SPIN_PID" >/dev/null 2>&1 || true
  fi
  (cd "$ROOT_DIR/oci-movie-search" && docker compose down --remove-orphans >/dev/null 2>&1 || true)
}
trap cleanup EXIT

wait_health() {
  local url="$1"
  for _ in $(seq 1 120); do
    if curl -fsS "$url/health" >/dev/null 2>&1; then
      return 0
    fi
    sleep 0.25
  done
  echo "Timed out waiting for $url/health" >&2
  return 1
}

echo "== Build frontend and unit tests =="
(cd "$ROOT_DIR/frontend" && npm ci && npm run build)
cargo test --manifest-path "$ROOT_DIR/spin-meili/Cargo.toml"
(cd "$ROOT_DIR/spin-meili" && spin build)
(cd "$ROOT_DIR/oci-movie-search" && docker compose build)

echo "== Smoke and parity =="
(cd "$ROOT_DIR/spin-meili" && exec spin up --listen 127.0.0.1:8080 >"$ROOT_DIR/results/raw/run_all_spin.log" 2>&1) &
SPIN_PID="$!"
(cd "$ROOT_DIR/oci-movie-search" && docker compose up --no-build -d)
wait_health "http://127.0.0.1:8080"
wait_health "http://127.0.0.1:8081"
"$ROOT_DIR/benchmarks/smoke_spin.sh"
"$ROOT_DIR/benchmarks/smoke_oci.sh"
"$ROOT_DIR/benchmarks/compare_results.sh"
cleanup
trap cleanup EXIT

echo "== Benchmarks =="
if [[ "$PILOT" == "1" ]]; then
  python3 "$ROOT_DIR/benchmarks/run_cold_start.py" --iterations 2 --include-first-search --build-spin --build-oci
  python3 "$ROOT_DIR/benchmarks/run_load.py" --duration 3 --concurrency 2 --queries space,empty,enhanced --repeats 1 --build-spin --build-oci
  python3 "$ROOT_DIR/benchmarks/run_memory.py" --idle-seconds 2 --load-seconds 3 --load-concurrency 2 --build-spin --build-oci
else
  python3 "$ROOT_DIR/benchmarks/run_cold_start.py" --include-first-search --build-spin --build-oci
  python3 "$ROOT_DIR/benchmarks/run_load.py" --repeats 3 --build-spin --build-oci
  python3 "$ROOT_DIR/benchmarks/run_load_hey.py" --build-spin --build-oci || true
  python3 "$ROOT_DIR/benchmarks/run_memory.py" --build-spin --build-oci
fi

echo "== Analysis =="
python3 "$ROOT_DIR/benchmarks/analyze_results.py"

echo "Done. Raw CSVs: results/raw; summaries: results/processed; plots: results/plots."
