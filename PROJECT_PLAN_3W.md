# Three-week Project Plan - Movie Search on Spin/wasmtime vs OCI

This plan implements [requirements.md](requirements.md). The project now compares one shared Rust movie-search service across Spin/wasmtime and OCI, with historical Meilisearch porting evidence kept as background.

## Architecture

| Area | Choice |
|---|---|
| Shared core | `spin-meili/crates/search-core` |
| Spin runtime | Spin HTTP component on `127.0.0.1:8080` |
| OCI runtime | Native Rust HTTP server in Docker on `127.0.0.1:8081` |
| Fixture | `fixtures/movies.json`, 44,471 deduplicated movies |
| Raw source data | `fixtures/movies_metadata.csv` |
| Benchmarks | Bash orchestration, Python validation/analysis, raw CSV outputs |

## Week 1 - Functional Parity

- Implement `movie-search-core` with deterministic loading, deduplication, tokenization, ranking, and pagination.
- Expose the same API from Spin and OCI:
  - `GET /health`
  - `GET /version`
  - `GET /stats`
  - `GET /movies?offset=&limit=`
  - `POST /search`
- Replace Meilisearch-specific smoke scripts with movie-search smoke scripts.
- Add a result-parity script that compares Spin and OCI hit IDs for representative queries.
- Preserve Meilisearch/WASI blocker docs as the explanation for the pivot.

**Exit:** `cargo test`, `spin build`, both smoke scripts, and parity comparison pass locally.

## Week 2 - Benchmark Harness

- Add cold-start measurement scripts:
  - Spin start to first `/health`;
  - OCI start to first `/health`;
  - optional start plus first `/search`.
- Add throughput/latency load generation for `POST /search`.
- Record raw CSVs under `results/raw`.
- Sample memory:
  - Docker/cgroup metrics for OCI;
  - host process metrics for Spin/wasmtime.
- Use concurrency levels `10`, `50`, `100`, and `200`.

**Exit:** pilot benchmark data exists for both runtimes and response validation is built into the load scripts.

## Week 3 - Analysis and Report

- Process CSVs into tables and plots under `results/processed` and `results/plots`.
- Write the final report:
  - methodology and hardware/software environment;
  - functional parity result;
  - cold-start comparison;
  - throughput and tail-latency comparison;
  - memory/isolation discussion;
  - Meilisearch feasibility appendix.
- Re-run final benchmark set from a clean state.

**Exit:** final data, plots, and report are reproducible from repository instructions.

## Acceptance Checklist

- [ ] Spin service runs locally.
- [ ] OCI service runs locally.
- [ ] Both use the same `movie-search-core`.
- [ ] Both embed the same `fixtures/movies.json`.
- [ ] `/stats.documentCount` is `44,471` in both runtimes.
- [ ] Smoke scripts pass.
- [ ] Result-parity script passes.
- [ ] Benchmark CSVs and plots are produced.
- [ ] Final report distinguishes the movie-search benchmark from historical Meilisearch feasibility evidence.
