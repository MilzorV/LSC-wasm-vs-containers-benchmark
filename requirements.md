# Requirements: Meilisearch-Compatible Microservice on Spin/wasmtime vs OCI

## 1. Assigned topic

> **11. WebAssembly Microservices with Spin/wasmtime**
>
> Deploy a microservice application using the Spin framework (WASI components) and compare cold-start latency, memory isolation, and throughput against equivalent OCI containers, evaluating Wasm as a practical serverless isolation substrate.

This project evaluates whether a WebAssembly service running through **Spin/wasmtime** can be a practical alternative to an equivalent **OCI container** workload for serverless-style microservices.

## 2. Concrete project

The project will build and benchmark a **subset-first Meilisearch-compatible HTTP microservice**:

- the Wasm implementation runs as a Spin HTTP component on wasmtime;
- the API surface mimics a small, useful subset of Meilisearch;
- the OCI baseline is the official `getmeili/meilisearch` Docker image, pinned to a specific version;
- both systems use the same fixture data and the same benchmarked API paths wherever possible.

The goal is not to port all of Meilisearch to WebAssembly. The goal is to create a realistic search-service workload with a familiar HTTP contract, then compare Wasm/Spin and OCI containers on the dimensions required by the course topic.

## 3. Research questions

The final report must answer:

- Does the Spin/wasmtime service cold-start faster than the OCI Meilisearch container?
- How much memory does each isolation model use at idle and under load?
- Which system provides better throughput and tail latency for the selected API surface?
- How do WASI capability-based sandboxing and container isolation differ operationally?
- Is Wasm a practical serverless isolation substrate for this class of microservice?

## 4. Scope and success criteria

### In scope

- A Rust-based Spin HTTP component targeting `wasm32-wasip2`.
- A minimal Meilisearch-compatible search API that keeps request logic in memory and snapshots state through Spin's default key-value store for local smoke-test continuity.
- A pinned official Meilisearch OCI baseline using Docker or Podman.
- Reproducible benchmark scripts for cold start, memory, throughput, and latency.
- Raw CSV results, processed summaries, plots, and a final written interpretation.
- Documentation that explains setup, methodology, known limitations, and reproduction steps.

### Success criteria

The project is successful when:

- the Spin service builds with `spin build` and runs with `spin up`;
- the OCI baseline runs locally from the pinned `getmeili/meilisearch` image;
- the same benchmark fixture can be loaded into both systems;
- the required MVP endpoints work on the Spin service;
- cold-start latency, memory usage, throughput, latency percentiles, and errors are measured for both systems;
- results are presented as tables and plots;
- the report gives a clear conclusion about Wasm as a serverless-style isolation substrate.

### Out of scope for the three-week MVP

- Full Meilisearch API compatibility.
- LMDB, memory-mapped storage, or compatibility with native `data.ms` files.
- Durable storage in the Spin MVP.
- Binary-compatible Meilisearch dumps or snapshots.
- Vector search, hybrid search, facet search, federated search, tenant tokens, webhooks, or full key management.
- Kubernetes or production SpinKube deployment.
- Writing a new WebAssembly runtime or serverless platform.

SQLite persistence and external stores may be discussed as future work, but they are not required for the benchmark MVP.

## 5. Required API surface

The Spin service must implement the following MVP endpoints.

| Endpoint | Required behavior |
|---|---|
| `GET /health` | Return `200 OK` with a JSON status object. |
| `GET /version` | Return implementation name, project version, and optional Git/build metadata. |
| `GET /indexes` | Return the list of known indexes. |
| `POST /indexes/{uid}/documents` | Add or replace documents in an index; create the index if missing. |
| `POST /indexes/{uid}/search` | Run basic search against the selected index. |
| `GET /stats` | Return instance-level and per-index counts useful for benchmarking. |
| `GET /tasks` | Return accepted document-ingestion tasks or a simplified synchronous task facade. |

### Search behavior

The MVP search endpoint must support at least:

- `q`;
- `offset`;
- `limit`.

The Spin implementation should provide deterministic full-text search good enough for benchmarking, not perfect Meilisearch ranking parity. Empty `q` should behave as placeholder search and return paginated documents. Search results should include enough metadata to compare basic shape and counts with the OCI baseline.

### Document ingestion behavior

`POST /indexes/{uid}/documents` must:

- accept a JSON array of documents;
- require a stable primary key field, documented in the fixture schema;
- return a Meilisearch-like task object;
- make documents visible to later searches in the benchmark flow.

The task object may represent synchronous completion internally, as long as this limitation is documented.

### Error behavior

The service must return JSON errors with:

- an HTTP status code appropriate to the failure;
- a stable machine-readable code;
- a human-readable message.

## 6. Architecture requirements

The implementation should separate portable search logic from Spin-specific HTTP glue.

Recommended workspace layout:

```text
.
|-- README.md
|-- requirements.md
|-- PROJECT_PLAN_3W.md
|-- spin-meili/
|   |-- Cargo.toml
|   |-- spin.toml
|   `-- crates/
|       |-- core/
|       |-- storage-memory/
|       `-- spin-http-adapter/
|-- oci-meilisearch/
|   |-- docker-compose.yml
|   `-- README.md
|-- fixtures/
|   `-- documents.json
|-- benchmarks/
|   |-- common.sh
|   |-- cold_start.sh
|   |-- throughput.sh
|   |-- memory.sh
|   |-- loadgen.py
|   `-- run_all.sh
|-- analysis/
|   `-- analyze_results.py
|-- results/
|   |-- raw/
|   |-- processed/
|   `-- plots/
|-- docs/
|   |-- ENVIRONMENT.md
|   `-- METHODOLOGY.md
`-- report/
    |-- report.md
    `-- figures/
```

Recommended crate responsibilities:

- `core`: request/response models, indexing, tokenization, ranking, pagination, and stats.
- `storage-memory`: portable in-memory index and task model for tests and benchmark logic.
- `spin-http-adapter`: Spin HTTP trigger integration, routing, auth hook if used, and JSON error mapping.

The MVP must not depend on LMDB, mmap, background threads, host filesystem persistence, or native Meilisearch storage internals.

## 7. Toolchain and runtime requirements

### Wasm/Spin side

Required:

- Rust with `rustup`;
- target `wasm32-wasip2`;
- Spin CLI pinned in documentation;
- wasmtime through Spin;
- `spin.toml` checked into the repository;
- `rust-toolchain.toml` or a documented Rust version.

Reference commands:

```bash
rustup target add wasm32-wasip2
spin build
spin up --listen 127.0.0.1:8080
```

### OCI side

Required:

- Docker or Podman;
- Docker Compose or equivalent start script;
- pinned `getmeili/meilisearch:<version>`;
- documented master key and data reset behavior;
- service listening on a separate port, for example `127.0.0.1:8081`.

The report must state the exact image tag used for the baseline.

## 8. Benchmark methodology

All measurements must be run on the same machine, with the same fixture data, the same client tool, and the same benchmark routes where possible.

### Environment recording

`docs/ENVIRONMENT.md` or generated metadata must include:

- OS and kernel version;
- CPU model and core count;
- RAM;
- power/performance mode if known;
- Rust version;
- Spin version;
- Docker/Podman version;
- Meilisearch image tag;
- Git commit SHA for the benchmark run.

### Fixture requirements

The fixture must:

- be stored in the repository or generated by a deterministic script;
- include enough documents to make search and pagination meaningful;
- use the same document schema for both Spin and OCI;
- document the primary key field;
- be loaded before throughput and memory benchmarks.

### Cold-start benchmark

Cold start is defined as:

1. ensure the service is stopped and previous service state is reset where required;
2. start the service;
3. send requests until the first successful `GET /health` or equivalent benchmark route response;
4. record elapsed time;
5. stop the service;
6. repeat.

Required metrics:

- minimum;
- maximum;
- mean;
- median;
- p95;
- number of failed runs.

Minimum repetitions: **20 cold starts per system**, unless the report explains why fewer were possible.

### Throughput and latency benchmark

Throughput tests must run against at least:

- `POST /indexes/{uid}/search` with a non-empty query;
- `POST /indexes/{uid}/search` with empty `q` placeholder search.

Required concurrency levels:

```text
10, 50, 100, 200
```

Required metrics:

- requests per second;
- p50 latency;
- p95 latency;
- p99 latency;
- average latency;
- error count and error rate.

Each measured cell should include a warmup period before the timed interval.

### Memory and isolation benchmark

Memory measurements must include:

- idle memory after startup;
- memory during low concurrency load;
- memory during high concurrency load;
- peak memory during the benchmark;
- post-load memory if practical.

The report must distinguish:

- Spin host process memory and any observable Wasm component memory;
- OCI container memory from cgroups or `docker stats`;
- qualitative isolation differences between WASI capabilities and container namespaces/cgroups/seccomp/AppArmor-style controls.

Spin memory and container memory may not be perfectly equivalent measurements. This limitation must be stated rather than hidden.

### Output requirements

Benchmark scripts must write raw machine-readable results, preferably CSV, into `results/raw/`. Analysis scripts must produce:

- processed summary tables in `results/processed/`;
- plots in `results/plots/`;
- report-ready figures copied or referenced from `report/figures/`.

## 9. Fairness rules

The comparison must be framed as **same benchmark surface**, not identical implementation.

Required fairness rules:

- use the same machine and network path, preferably localhost;
- run only one measured service at a time unless a benchmark explicitly requires otherwise;
- use the same fixture data and query set;
- pin tool versions and container image tags;
- separate cold-start measurements from warm throughput measurements;
- report failed runs and error rates;
- avoid changing benchmark scripts between Spin and OCI runs except for target URL and service-control command.

## 10. Security and isolation discussion

The project must include a qualitative comparison of the isolation models.

The Wasm/Spin discussion should cover:

- WASI capability-oriented access;
- host-controlled resources;
- absence of direct general OS access from the component;
- small startup footprint as a serverless advantage;
- maturity and ecosystem limitations.

The OCI discussion should cover:

- process, filesystem, and network namespace isolation;
- cgroups resource accounting and limits;
- mature deployment and observability tooling;
- larger image and runtime surface compared with a Wasm component.

The conclusion must connect these isolation properties back to the measured cold-start, memory, and throughput results.

## 11. Deliverables

The final repository should include:

1. Working Spin/wasmtime Meilisearch-compatible MVP.
2. Working official Meilisearch OCI baseline.
3. Fixture data and load/import scripts.
4. Benchmark scripts for cold start, memory, throughput, and latency.
5. Raw benchmark results.
6. Processed summaries and plots.
7. Methodology and environment documentation.
8. Final report with limitations and recommendation.
9. Demo instructions for running both systems and reproducing at least one benchmark.

## 12. Known limitations to document

The final report must explicitly state:

- the Spin service is a subset implementation, not Meilisearch itself;
- API-level equivalence is limited to selected endpoints and selected request shapes;
- ranking behavior may differ from native Meilisearch;
- the Spin MVP uses a local Spin key-value snapshot for request-to-request continuity and benchmark scripts must clear it when a clean cold-start state is required;
- memory measurements for Spin and OCI are collected through different runtime mechanisms;
- results from a local benchmark do not automatically generalize to a production serverless platform.

## 13. Optional stretch work

Stretch work is allowed only after the benchmark MVP is complete:

- SQLite-backed Spin storage;
- `GET /tasks/{id}`;
- `GET /indexes/{uid}/documents`;
- partial document updates;
- more realistic ranking/tokenization;
- a larger fixture generator;
- SpinKube deployment notes;
- comparison with another lightweight container baseline.
