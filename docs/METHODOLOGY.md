# Methodology

## Current Comparison Model

The project compares one application implemented once and exposed through two runtime adapters.

| System | URL | Purpose |
|---|---|---|
| Spin/wasmtime | `http://127.0.0.1:8080` | WASI HTTP component using `movie-search-core` |
| OCI | `http://127.0.0.1:8081` | Native Rust HTTP server using `movie-search-core` |

This replaces the earlier Meilisearch comparison. That path remains documented in [upstream-wasi-blockers.md](upstream-wasi-blockers.md), but it is not the main benchmark because the Spin fallback and official Meilisearch did not share storage or ranking semantics.

## Fixture

The shared fixture is `fixtures/movies.json`.

- Documents: `44,471`
- Primary key: `id`
- Fields: `id`, `title`, `overview`, `genre`, `year`
- Deduplication: last-write-wins by `id`

The raw CSV source remains in `fixtures/movies_metadata.csv`.

## Shared API Surface

Both runtimes expose the same endpoints:

- `GET /` — runtime comparison UI (Spin vs OCI; presentation only, not benchmarked)
- `GET /benchmarks` — benchmark dashboard (processed summaries + plots)
- `GET /demo` — alias for `/`
- `GET /assets/*`, `GET /benchmark-data/*` — frontend static assets
- `GET /health`
- `GET /version`
- `GET /stats`
- `GET /movies?offset=&limit=`
- `POST /search`

Search request body:

```json
{"q":"space","offset":0,"limit":20}
```

Search responses include:

```json
{
  "hits": [],
  "query": "space",
  "offset": 0,
  "limit": 20,
  "estimatedTotalHits": 0,
  "processingTimeMs": 0
}
```

`processingTimeMs` is not used for parity checks because it naturally varies by runtime and request.

## Search Semantics

The ranking is intentionally simple and deterministic:

1. tokenize the query on non-alphanumeric characters;
2. match tokens by case-insensitive substring search;
3. sort by number of matched query tokens descending;
4. break ties by field weight, with `title > genre > overview`;
5. break remaining ties by ascending `id`.

Empty queries return all documents by ascending `id`.

## Acceptance Checks

Spin:

```bash
cd spin-meili
spin build
spin up --listen 127.0.0.1:8080
```

From the repository root:

```bash
benchmarks/smoke_spin.sh
```

OCI:

```bash
cd oci-movie-search
docker compose up --build
```

From the repository root:

```bash
benchmarks/smoke_oci.sh
```

With both services running:

```bash
benchmarks/compare_results.sh
```

For live presentation, `http://127.0.0.1:8080/` compares Spin and OCI side by side.
`http://127.0.0.1:8080/benchmarks` shows processed benchmark artifacts.
Neither page is part of benchmark metrics.

## Benchmark Surface

The benchmark scripts measure:

- cold start to first successful `/health` (`ready_ms`);
- optional post-ready first search latency (`search_after_ready_ms`);
- optional total cold path from process start through first successful search (`total_cold_path_ms`);
- search throughput for `POST /search` with `{"q":"space"}`;
- empty-query throughput for `POST /search` with `{"q":""}`;
- idle and under-load memory for both systems;
- latency percentiles p50, p95, p99;
- error counts and response validation;
- load repeat variance when `--repeats` is greater than 1.

Required concurrency levels are `10`, `50`, `100`, and `200`.

Load harness details: [LOAD_HARNESS.md](LOAD_HARNESS.md).

## Memory Measurement

Memory metrics are **not perfectly symmetric** across Spin and OCI. Treat them as operational estimates and always read the `source` column.

| System | Primary source | Comparable secondary source |
|---|---|---|
| Spin | `host_process_rss` (Spin/wasmtime process tree on the host) | none |
| OCI | `docker_stats` (container usage reported by Docker) | `host_process_rss` (container PID tree on the host) |

For cross-runtime comparison, prefer `host_process_rss` samples and the plot
`results/plots/memory_peak_host_rss.png`. Docker-reported OCI memory remains in
the dataset because it reflects common container operations tooling.

## Benchmark Scripts

Run a short pilot:

```bash
benchmarks/run_all.sh --pilot
```

Run the full benchmark:

```bash
benchmarks/run_all.sh
```

Equivalent shortcuts:

```bash
make benchmark-pilot
make benchmark
make analyze
```

Standalone runners rebuild artifacts when passed `--build-spin` and/or
`--build-oci`. The orchestrator always builds before benchmarking.

Analysis uses only the latest run id per raw prefix by default. Pass
`--all-runs` to `analyze_results.py` to aggregate every historical CSV.

Raw outputs are written to `results/raw`, processed summaries to
`results/processed`, and plots to `results/plots`.

## Result File Contracts

Cold start raw CSV:

```text
system,scenario,iteration,success,ready_ms,search_after_ready_ms,total_cold_path_ms,first_search_ms,error
```

- `ready_ms`: startup until `/health` succeeds
- `search_after_ready_ms`: first successful `/search` after `/health`
- `total_cold_path_ms`: startup until first successful `/search`
- `first_search_ms`: deprecated alias of `total_cold_path_ms` for older runs

Load raw CSV:

```text
run_id,repeat,system,query,concurrency,request_id,success,status,latency_ms,error
```

Reference `hey` CSV:

```text
run_id,tool,system,query,concurrency,duration_seconds,request_rate,latency_p50_ms,latency_p95_ms,latency_p99_ms,success_count,error_count
```

Memory raw CSV:

```text
system,phase,timestamp_ms,memory_bytes,source
```

Processed summaries include sample counts, success/error counts, min, mean,
median, p50, p95, p99, max, standard deviation where applicable, and request
rate plus repeat throughput variance for load runs.
