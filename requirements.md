# Requirements: Upstream Meilisearch on Spin/wasmtime vs OCI

## 1. Assigned topic

> **11. WebAssembly Microservices with Spin/wasmtime**
>
> Deploy a microservice application using the Spin framework (WASI components) and compare cold-start latency, memory isolation, and throughput against equivalent OCI containers, evaluating Wasm as a practical serverless isolation substrate.

This project evaluates whether a Meilisearch-style HTTP microservice can run as a WebAssembly component through **Spin/wasmtime**, and how that compares with an equivalent **OCI container** deployment.

## 2. Updated project goal

The primary goal is now an attempted **same-version Meilisearch port**:

- use the official upstream `meilisearch/meilisearch` source pinned to `v1.43.0`;
- keep the OCI baseline pinned to `getmeili/meilisearch:v1.43.0`;
- replace the native Meilisearch HTTP/server boundary with a Spin HTTP component where possible;
- route the benchmark API surface into the highest upstream Meilisearch layer that can realistically compile for `wasm32-wasip2`;
- document every storage, runtime, and API deviation needed for WASI/Spin.

This is a feasibility and benchmark project, not a claim that unmodified Meilisearch already runs in Spin. If full upstream compilation is blocked, the project must preserve the evidence and continue with the closest honest source-sharing port.

## 3. Research questions

The final report must answer:

- Can the pinned Meilisearch `v1.43.0` source compile for native host builds?
- Which upstream crates compile for `wasm32-wasip2`, and where does the Spin/WASI port fail?
- Can a Spin HTTP component reuse upstream Meilisearch API, parsing, search, or indexing layers?
- How does Spin/wasmtime cold-start latency compare with the official OCI container?
- How much memory overhead and isolation cost does each runtime model show?
- How do throughput and tail latency compare on the same fixture and selected API routes?
- Is Wasm a practical serverless isolation substrate for this class of search microservice?

## 4. Version pinning

| Item | Required value |
|---|---|
| Upstream source | `meilisearch/meilisearch` tag `v1.43.0` |
| Observed upstream commit | `475ed56e5612df0dbb826748add5f93e0e7d5500` |
| OCI baseline image | `getmeili/meilisearch:v1.43.0` |
| Spin target | Rust `wasm32-wasip2` component |
| Spin runtime | wasmtime through Spin |

The repository must include scripts that fetch or verify the pinned upstream source. The generated upstream checkout may live under `vendor/meilisearch/`, but it does not need to be committed if it is reproducibly fetched by script.

## 5. Porting scope

### In scope

- Native host verification of the pinned upstream Meilisearch source.
- Layered WASI feasibility checks for selected upstream crates.
- A Spin HTTP adapter under `spin-meili/crates/spin-http-adapter`.
- A compatibility crate under `spin-meili/crates/meili-wasi-compat` for documenting and isolating WASI-specific deviations.
- The same pinned Meilisearch mini-dashboard served by Spin and the OCI baseline for browser-level demo parity.
- Reuse of upstream Meilisearch code wherever it compiles cleanly or can be adapted with narrowly scoped shims.
- A legacy subset Spin implementation retained only as a fallback/reference path while the upstream port boundary is explored.
- The official Meilisearch OCI container as the native baseline.
- Reproducible fixture loading, smoke checks, benchmark scripts, raw CSVs, plots, and final interpretation.

### Expected blocker areas

The project must explicitly investigate and document:

- LMDB/heed storage dependencies;
- memory-mapped storage assumptions;
- filesystem and snapshot behavior;
- Tokio/native server runtime assumptions;
- background task scheduling;
- native process configuration;
- telemetry and other host integrations;
- C or crypto build dependencies that do not target `wasm32-wasip2` cleanly.

### Out of scope

- Pretending a custom subset is identical to upstream Meilisearch.
- Binary compatibility with native Meilisearch `data.ms` directories.
- Full Meilisearch API coverage in the Spin adapter.
- Kubernetes or SpinKube deployment for the MVP.
- Production persistence guarantees inside Spin.
- Rewriting Meilisearch from scratch.

## 6. Benchmark API surface

The benchmark remains intentionally narrow so both runtimes can be compared on the same HTTP surface:

| Endpoint | Purpose |
|---|---|
| `GET /health` | Service readiness and cold-start success marker. |
| `GET /version` | Runtime/source identity and build metadata. |
| `GET /indexes` | List benchmark indexes. |
| `POST /indexes/{uid}/documents` | Load the shared fixture. |
| `POST /indexes/{uid}/search` | Run search and placeholder search. |
| `GET /stats` | Report document/index counts. |
| `GET /tasks` | Report ingestion task state. |

For browser mirror demos, Spin must also serve the pinned mini-dashboard at `/` and implement the document browsing, index stats, and settings routes the dashboard uses for the selected `movies` workflow.

The shared fixture uses:

- index UID: `movies`;
- primary key: `id`;
- fields: `id`, `title`, `overview`, `genre`, `year`;
- smoke query: `space`.

The OCI baseline must use native Meilisearch endpoints. The Spin side must use upstream Meilisearch request/search/indexing logic where possible. If the Spin side falls back to the legacy subset implementation, that must be labeled as a porting deviation.

## 7. Required scripts and evidence

The repository must provide:

- `scripts/fetch-meilisearch.sh` to fetch the pinned upstream source;
- `scripts/check-upstream-native.sh` to run a native upstream cargo check;
- `scripts/check-upstream-wasi.sh` to run layered `wasm32-wasip2` checks and generate `docs/upstream-wasi-blockers.md`;
- `benchmarks/smoke_spin.sh` to check the Spin service;
- `benchmarks/smoke_oci.sh` to check the official OCI service.

`docs/upstream-wasi-blockers.md` must be treated as primary project evidence. It should identify the highest upstream crates that compile for WASI and the first blocker layers.

## 8. Benchmark requirements

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

## 9. Feasibility checkpoint

The project is successful if it honestly reaches one of these outcomes:

1. **Best case:** a Spin component reuses enough upstream Meilisearch code to serve the benchmark endpoints from the pinned source version.
2. **Acceptable porting outcome:** upstream API/parsing/search layers are reused, but storage or runtime boundaries require documented WASI compatibility shims.
3. **Acceptable feasibility outcome:** full upstream reuse is blocked by documented evidence, and the project benchmarks the existing Spin subset only as a clearly labeled fallback while explaining why exact equivalence is not currently practical.

The final conclusion must not hide the difference between these outcomes.

## 10. Deliverables

- Working official OCI Meilisearch baseline pinned to `v1.43.0`.
- Pinned upstream source fetch/check scripts.
- Native upstream build evidence.
- WASI feasibility blocker report.
- Spin HTTP adapter and compatibility crate.
- Legacy subset fallback kept clearly named and documented.
- Smoke scripts for Spin and OCI.
- Benchmark scripts and raw CSV outputs.
- Processed tables, plots, and final report.
- Reproducibility instructions in `README.md` and `docs/METHODOLOGY.md`.
