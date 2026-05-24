# Teacher Consultation Notes

> **Current status (2026-05-24):** the project completed the shared
> `movie-search-core` benchmark across Spin/WASM and OCI. The Meilisearch notes
> below are retained as the technical reason for the pivot and as consultation
> history.

Date: 2026-05-11 (original consultation), updated 2026-05-24

## Project topic

We are working on:

> WebAssembly Microservices with Spin/wasmtime: deploy a microservice application using Spin/WASI components and compare cold-start latency, memory isolation, and throughput against equivalent OCI containers.

## Final benchmark model

The submitted project compares one shared Rust HTTP service in two isolation models:

| Runtime | URL | Adapter |
|---|---|---|
| Spin/wasmtime | `http://127.0.0.1:8080` | `spin-meili/crates/spin-http-adapter` |
| OCI/Docker | `http://127.0.0.1:8081` | `spin-meili/crates/oci-http-adapter` |

Shared core: `spin-meili/crates/search-core` over `fixtures/movies.json`
(44,471 documents).

API:

- `GET /health`, `/version`, `/stats`, `/movies`
- `POST /search`

Parity is verified with `benchmarks/compare_results.sh` before performance runs.

## What we did after consultation

- Implemented deterministic search, pagination, and deduplication in `movie-search-core`.
- Built matching Spin and OCI HTTP adapters on the same fixture and API.
- Added cold-start, load, memory, analysis, plots, report, presentation, and demo script.
- Documented the abandoned Meilisearch path in `docs/upstream-wasi-blockers.md`.
- Moved historical Meilisearch-era crates to `archive/spin-meili/crates/`.

## Meilisearch feasibility history (background only)

Original plan before pivot:

- **OCI side:** official `getmeili/meilisearch:v1.43.0`
- **Wasm side:** upstream Meilisearch-on-Spin port attempt

Evidence collected:

```bash
scripts/check-upstream-native.sh   # pass
scripts/check-upstream-wasi.sh     # fail from milli upward
```

Detailed blockers: `docs/upstream-wasi-blockers.md` (LMDB/mmap, native crypto/C deps, Tokio wasm constraints).

## Current benchmark surface

Measured metrics:

- cold start to `/health` (`ready_ms`);
- post-ready first search (`search_after_ready_ms`);
- total cold path through first search (`total_cold_path_ms`);
- search throughput for `POST /search` with `{"q":"space"}`;
- empty-query throughput for `POST /search` with `{"q":""}`;
- idle and under-load memory with explicit source labels;
- latency percentiles p50, p95, p99;
- response validation and error counts under concurrency.

Concurrency levels: `10`, `50`, `100`, `200`.

Full runs repeat load scenarios three times and analyze only the latest raw run
id by default.

## Important limitations

- Search is a deterministic linear scan, not a production inverted index.
- Memory comparison uses multiple sources; host RSS is the closest apples-to-apples metric.
- Benchmarks run on a single documented host (`docs/ENVIRONMENT.md`).
- The Meilisearch path is feasibility evidence only, not the main comparison.

## Deliverables

- Report: `report/final-report.pdf`
- Presentation: `presentation/movie-search-spin-vs-oci.pdf`
- Demo script: `demo/demo-script.md`
- Reproducible benchmark: `benchmarks/run_all.sh` or `make benchmark`

## Questions resolved by the final implementation

1. **Same application behavior?** Yes — shared core, parity script, identical hit IDs.
2. **Cold start definition?** We report `/health`, post-ready search, and total cold path separately.
3. **Fixture size:** 44,471 deduplicated movies from TMDB metadata CSV.
4. **Memory discussion:** Both conceptual isolation comparison and measured samples, with source labels and caveats.
5. **Deliverable format:** Report PDF, presentation, demo script, raw/processed results, and plots.

## Suggested follow-up if extending the project

- Add inverted-index parity while keeping the same API.
- Run the benchmark on Linux CI hardware in addition to the documented M4 host.
- Compare Spin host RSS against OCI host RSS as the primary memory chart in the report narrative.
