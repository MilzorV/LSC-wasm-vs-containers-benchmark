# LSC - Movie Search on Spin/wasmtime vs OCI

This project compares the same small movie-search HTTP service running in two isolation models:

- **Spin/wasmtime** through a WASI HTTP component;
- **OCI container** through a native Rust HTTP server.

The earlier Meilisearch porting path is kept as feasibility evidence only. The official Meilisearch container used LMDB and native runtime layers, while the Spin side had a custom fallback with different search semantics. That made result parity impossible to claim honestly. The current benchmark therefore uses one shared Rust core, `movie-search-core`, in both runtimes.

Full specification: [requirements.md](requirements.md).

Methodology: [docs/METHODOLOGY.md](docs/METHODOLOGY.md).

Load harness notes: [docs/LOAD_HARNESS.md](docs/LOAD_HARNESS.md).

## Current Status

- Shared search core: `spin-meili/crates/search-core`.
- Spin adapter: `spin-meili/crates/spin-http-adapter`.
- OCI adapter: `spin-meili/crates/oci-http-adapter`.
- Canonical fixture: `fixtures/movies.json`.
- Raw source data: `fixtures/movies_metadata.csv`.
- Dataset size after last-write-wins deduplication by `id`: `44,471` movies.
- Historical Meilisearch/WASI blocker evidence: [docs/upstream-wasi-blockers.md](docs/upstream-wasi-blockers.md).

## Prerequisites

Required tools:

- Spin CLI 3.x;
- Rust with `wasm32-wasip2`;
- Docker with Compose;
- Node.js 22+ and npm (frontend build);
- Bash, curl, and Python 3.

Install the Rust target for the active toolchain:

```bash
rustup target add wasm32-wasip2
```

## Run the Spin Service

```bash
cd spin-meili
cd ../frontend && npm ci && npm run build
cd ../spin-meili && spin build
spin up --listen 127.0.0.1:8080
```

From another shell:

```bash
benchmarks/smoke_spin.sh
```

The Spin service exposes:

- `GET /` — runtime comparison UI (Spin vs OCI search)
- `GET /benchmarks` — benchmark dashboard (tables + plots)
- `GET /demo` — alias for `/` (legacy URL)
- `GET /assets/*`, `GET /benchmark-data/*` — frontend static assets
- `GET /health`
- `GET /version`
- `GET /stats`
- `GET /movies?offset=&limit=`
- `POST /search`

Example search:

```bash
curl -fsS \
  -X POST http://127.0.0.1:8080/search \
  -H 'content-type: application/json' \
  --data '{"q":"space","limit":3}'
```

## Run the OCI Service

```bash
cd oci-movie-search
docker compose up --build
```

From the repository root:

```bash
benchmarks/smoke_oci.sh
```

The OCI service listens on `127.0.0.1:8081` and serves the same API from the same `movie-search-core` crate.

Stop the OCI service:

```bash
cd oci-movie-search
docker compose down
```

## Compare Result Parity

With both services running:

```bash
benchmarks/compare_results.sh
```

The comparison script validates that Spin and OCI return the same engine name, stats, total hits, and hit IDs for representative queries:

- `space`
- `toy story`
- `dark knight`
- `romance`
- empty query

## Web app

Build the frontend once (copies benchmark data into `dist/`), then start backends:

```bash
make analyze          # optional: refresh dashboard.json and plots
make frontend-build
make build
```

Start **both** Spin (`:8080`) and OCI (`:8081`) for the compare UI:

```text
http://127.0.0.1:8080/
```

- **Search both** — same query against Spin and OCI; parity badge when rankings match
- **`/demo`** — same page (legacy link)

Benchmark dashboard (embedded summaries from last `make analyze`):

```text
http://127.0.0.1:8080/benchmarks
```

To run benchmarks from the browser, start the local helper in another terminal:

```bash
make bench-ui   # http://127.0.0.1:8092
```

Local frontend development (proxies API to Spin on `:8080`):

```bash
cd frontend && npm run dev
```

Optional macOS shortcut:

```bash
make demo-open
```

Source: [`frontend/`](frontend/).

## Run Benchmarks

For a quick end-to-end pilot:

```bash
benchmarks/run_all.sh --pilot
```

For the full benchmark plan:

```bash
benchmarks/run_all.sh
# or
make benchmark
```

The full run builds both runtimes, runs smoke/parity checks, collects cold-start, load, and memory samples, then generates processed summaries and plots. Load runs repeat three times by default; analysis uses only the latest raw run id unless `--all-runs` is passed to `analyze_results.py`.

Outputs:

- raw CSVs and service logs: `results/raw/`;
- summary CSVs and `dashboard.json`: `results/processed/`;
- plots: `results/plots/`;
- LaTeX report source/PDF: `report/final-report.tex`, `report/final-report.pdf`;
- presentation: `presentation/movie-search-spin-vs-oci.pptx`, `presentation/movie-search-spin-vs-oci.pdf`;
- demo script: `demo/demo-script.md`.

Individual runners are also available:

```bash
benchmarks/run_cold_start.py --include-first-search --build-spin --build-oci
benchmarks/run_load.py --repeats 3 --build-spin --build-oci
benchmarks/run_load_hey.py --build-spin --build-oci
benchmarks/run_memory.py --build-spin --build-oci
benchmarks/analyze_results.py
make analyze
```

## Historical Meilisearch Checks

The repository still contains scripts for the earlier feasibility investigation:

```bash
scripts/fetch-meilisearch.sh
scripts/check-upstream-native.sh
scripts/check-upstream-wasi.sh
```

Those scripts are not part of the main benchmark path anymore. Their purpose is to preserve evidence that a full same-version Meilisearch-on-Spin comparison was blocked by native runtime, crypto/C, LMDB/heed, and memory-mapped storage assumptions.
