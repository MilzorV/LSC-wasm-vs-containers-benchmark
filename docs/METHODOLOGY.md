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

## Benchmark Surface

The benchmark scripts should measure:

- cold start to first successful `/health`;
- optional cold start plus first successful `/search`;
- search throughput for `POST /search` with `{"q":"space"}`;
- placeholder search throughput for `POST /search` with `{"q":""}`;
- idle and under-load memory for both systems;
- latency percentiles p50, p95, p99;
- error counts and response validation.

Required concurrency levels are `10`, `50`, `100`, and `200`.

Memory metrics are not perfectly symmetric across Spin and OCI. The report must state this clearly and identify whether a metric comes from Docker/cgroup data or host process sampling.

## Benchmark Scripts

Run a short pilot:

```bash
benchmarks/run_all.sh --pilot
```

Run the full benchmark:

```bash
benchmarks/run_all.sh
```

Raw outputs are written to `results/raw`, processed summaries to `results/processed`, and plots to `results/plots`.

## Result File Contracts

Cold start raw CSV:

```text
system,scenario,iteration,success,ready_ms,first_search_ms,error
```

Load raw CSV:

```text
system,query,concurrency,request_id,success,status,latency_ms,error
```

Memory raw CSV:

```text
system,phase,timestamp_ms,memory_bytes,source
```

Processed summaries include sample counts, success/error counts, min, mean, median, p50, p95, p99, max, and request rate where applicable.
