# Requirements: Movie Search on Spin/wasmtime vs OCI

## 1. Assigned Topic

> **11. WebAssembly Microservices with Spin/wasmtime**
>
> Deploy a microservice application using the Spin framework (WASI components) and compare cold-start latency, memory isolation, and throughput against equivalent OCI containers, evaluating Wasm as a practical serverless isolation substrate.

This project evaluates the same movie-search HTTP microservice in Spin/wasmtime and in an OCI container.

## 2. Updated Project Goal

The project now uses one shared Rust application core:

- `movie-search-core` owns fixture loading, deduplication, tokenization, ranking, pagination, and response types;
- the Spin adapter exposes that core as a WASI HTTP component;
- the OCI adapter exposes that core as a native Rust HTTP server;
- both runtimes embed the same canonical `fixtures/movies.json` fixture.

The previous Meilisearch port attempt is retained only as feasibility evidence. It showed that a full same-version Meilisearch-on-Spin benchmark was not practical within this project because upstream Meilisearch depends on native runtime layers and LMDB/memory-mapped storage.

## 3. Research Questions

The final report must answer:

- Can the same Rust HTTP microservice run in Spin/wasmtime and OCI with identical functional results?
- How does Spin/wasmtime cold-start latency compare with the OCI container?
- How much memory overhead and isolation cost does each runtime model show?
- How do throughput and tail latency compare on the same fixture and selected API routes?
- Is Wasm a practical serverless isolation substrate for this class of read-heavy search microservice?
- What did the abandoned Meilisearch feasibility path reveal about porting native storage-heavy services to WASI?

## 4. Implementation Scope

### In scope

- Shared Rust core under `spin-meili/crates/search-core`.
- Spin HTTP adapter under `spin-meili/crates/spin-http-adapter`.
- OCI HTTP adapter under `spin-meili/crates/oci-http-adapter`.
- OCI Docker Compose runner under `oci-movie-search`.
- Canonical movie fixture under `fixtures/movies.json`.
- Smoke and parity scripts for both runtimes.
- Raw CSVs, processed summaries, plots, and final interpretation.
- Historical Meilisearch/WASI blocker evidence.

### Out of scope

- Claiming Meilisearch result parity.
- Reusing or converting native Meilisearch `data.ms` / LMDB storage.
- Meilisearch-compatible API coverage.
- Typo tolerance, fuzzy ranking, proximity ranking, synonyms, filters, facets, or Meilisearch ranking rules.
- Production persistence guarantees.

## 5. Shared API Surface

| Endpoint | Purpose |
|---|---|
| `GET /health` | Service readiness and cold-start success marker. |
| `GET /version` | Shared engine identity and dataset count. |
| `GET /stats` | Document count. |
| `GET /movies?offset=&limit=` | Deterministic movie listing. |
| `POST /search` | Deterministic movie search. |

Search request body:

```json
{"q":"space","offset":0,"limit":20}
```

Search response fields:

- `hits`
- `query`
- `offset`
- `limit`
- `estimatedTotalHits`
- `processingTimeMs`

## 6. Fixture Requirements

The shared fixture uses:

- file: `fixtures/movies.json`;
- source: `fixtures/movies_metadata.csv`;
- document count: `44,471`;
- primary key: `id`;
- fields: `id`, `title`, `overview`, `genre`, `year`;
- duplicate policy: last-write-wins by `id`.

## 7. Benchmark Requirements

All measurements must run on the same machine, using the same fixture and the same client tooling where possible.

### Cold start

Required:

- repeated stop/start runs, at least 20 repetitions per system;
- first successful `/health` as the basic readiness marker;
- optional start-plus-first-search scenario;
- CSV output with min, max, mean, median, p95, failures, and raw samples.

### Throughput and latency

Required:

- fixed concurrency levels: `10`, `50`, `100`, `200`;
- scenarios for `{"q":"space"}` and `{"q":""}`;
- latency percentiles p50, p95, p99;
- request rate, error count, and response validation;
- raw CSV plus processed tables and plots.

### Memory and isolation

Required:

- idle memory sampling;
- under-load sampling;
- peak and post-load memory where possible;
- container memory through Docker/cgroup metrics;
- Spin host/runtime memory through process metrics available on the host;
- qualitative isolation comparison between WASI capabilities and OCI container isolation.

Memory metrics are not perfectly symmetric across Spin and OCI. The report must state this clearly.

## 8. Required Checks

The repository must provide and keep passing:

- `cargo test --manifest-path spin-meili/Cargo.toml`;
- `spin build` from `spin-meili`;
- `benchmarks/smoke_spin.sh`;
- `benchmarks/smoke_oci.sh`;
- `benchmarks/compare_results.sh` when both services are running.

## 9. Deliverables

- Working Spin movie-search service.
- Working OCI movie-search service.
- Shared `movie-search-core` crate.
- Canonical fixture and source data notes.
- Smoke and parity scripts.
- Benchmark scripts and raw CSV outputs.
- Processed tables, plots, and final report.
- Reproducibility instructions in `README.md` and `docs/METHODOLOGY.md`.
- Historical Meilisearch feasibility notes explaining why the service changed.
