# Three-week project plan - Meilisearch-compatible Spin/wasmtime vs OCI

This plan implements [requirements.md](requirements.md). Work is organized by week and keeps the course topic central: deploy a Spin/wasmtime microservice, compare it against an equivalent OCI container workload, and evaluate Wasm as a practical serverless isolation substrate.

## Stack fixed for the project

| Layer | Choices |
|---|---|
| Wasm workload | Rust Spin HTTP component targeting `wasm32-wasip2`; subset-first Meilisearch-compatible API; in-memory storage for the MVP |
| Runtime | Spin CLI pinned in docs; wasmtime through Spin |
| OCI baseline | Docker or Podman; Docker Compose; pinned official `getmeili/meilisearch:<version>` image |
| Benchmark surface | `GET /health`, document ingestion, `POST /indexes/{uid}/search` with non-empty query and empty placeholder search |
| Benchmarks | Bash scripts for orchestration; Python 3.11+ stdlib load generator or optional `wrk` / `hey` / `bombardier`; CSV outputs |
| Analysis | Python with `pandas` and `matplotlib`; raw CSV to summaries, plots, and report figures |
| Docs | Markdown methodology, environment record, final report, and demo notes |

**Ports:** Spin on `127.0.0.1:8080`; OCI Meilisearch on `127.0.0.1:8081`. Only one stack should be measured at a time unless a script explicitly needs both for setup validation.

**Version pinning:** Record Rust, Spin, Docker/Podman, Python, Git SHA, and the exact `getmeili/meilisearch` image tag before collecting final results.

## Target repository layout

```text
.
|-- README.md
|-- requirements.md
|-- PROJECT_PLAN_3W.md
|-- spin-meili/
|-- oci-meilisearch/
|-- fixtures/
|-- benchmarks/
|-- analysis/
|-- results/
|   |-- raw/
|   |-- processed/
|   `-- plots/
|-- docs/
`-- report/
```

The exact internal Rust crate layout can be adjusted while implementing, but the service should keep search logic, in-memory storage, and Spin HTTP adapter code separated.

## Week 1 - Workloads, API surface, and smoke tests

- Finalize repo layout: `spin-meili/`, `oci-meilisearch/`, `fixtures/`, `benchmarks/`, `analysis/`, `results/raw|processed|plots/`, `docs/`, and `report/`.
- Scaffold the Rust Spin component:
  - target `wasm32-wasip2`;
  - checked-in `spin.toml`;
  - documented `spin build` and `spin up --listen 127.0.0.1:8080`.
- Implement the MVP Spin API:
  - `GET /health`;
  - `GET /version`;
  - `GET /indexes`;
  - `POST /indexes/{uid}/documents`;
  - `POST /indexes/{uid}/search`;
  - `GET /stats`;
  - `GET /tasks`.
- Implement in-memory storage only:
  - index metadata;
  - document storage keyed by documented primary key;
  - simple deterministic token/search behavior;
  - synchronous Meilisearch-like task facade for ingestion.
- Add JSON error responses with stable codes and clear messages.
- Add fixture data in `fixtures/` and document the schema, primary key, and example queries.
- Add OCI baseline under `oci-meilisearch/`:
  - Docker Compose or equivalent run script;
  - pinned `getmeili/meilisearch:<version>`;
  - documented master key and clean-data/reset command;
  - listening on `127.0.0.1:8081`.
- Create initial smoke scripts or commands:
  - start Spin and call `/health`;
  - load fixtures into Spin and run one search;
  - start OCI Meilisearch, load the same fixture, and run the comparable search.
- Draft `docs/ENVIRONMENT.md` and `docs/METHODOLOGY.md` with the measurement definitions from `requirements.md`.

**Exit:** Spin and OCI services both run locally, the fixture can be loaded into both, MVP endpoints work on Spin, and smoke checks prove comparable benchmark routes exist.

## Week 2 - Benchmark harness and measurement campaign

- Implement benchmark scripts in `benchmarks/`:
  - `common.sh` for ports, URLs, fixture path, image tag, output paths, and cleanup helpers;
  - `cold_start.sh` for repeated stop/start/first-success timing;
  - `throughput.sh` for concurrency-grid load tests;
  - `memory.sh` for idle, under-load, peak, and post-load memory sampling;
  - `loadgen.py` if not using an external HTTP benchmark tool;
  - `run_all.sh` for a complete campaign.
- Use consistent CSV schemas with fields such as:
  - system (`spin` or `oci`);
  - route/scenario;
  - run id;
  - concurrency;
  - duration;
  - requests per second;
  - latency p50/p95/p99;
  - error count;
  - memory sample or peak;
  - Git SHA and timestamp.
- Run cold-start campaign:
  - target at least 20 iterations per system;
  - record min, max, mean, median, p95, and failures.
- Run throughput and latency campaign:
  - scenarios: non-empty search query and empty placeholder search;
  - concurrency levels: `10, 50, 100, 200`;
  - warmup before each timed window;
  - repeat failed or high-variance cells with notes.
- Run memory campaign:
  - Spin host process RSS and any observable component-level data;
  - OCI cgroup memory via `docker stats` or equivalent;
  - idle, low load, high load, peak, and post-load phases.
- Archive raw results under `results/raw/` with timestamp and Git SHA.
- Record any measurement caveats immediately in `docs/METHODOLOGY.md`.

**Exit:** Complete raw datasets exist for cold start, throughput/latency, and memory for both systems, with environment metadata and repeatable commands.

## Week 3 - Analysis, report, and demo

- Implement or finish `analysis/analyze_results.py`:
  - load raw CSVs;
  - validate required columns;
  - compute summary statistics;
  - generate processed tables in `results/processed/`;
  - generate plots in `results/plots/`.
- Produce required figures:
  - cold-start comparison;
  - throughput vs concurrency;
  - p95/p99 latency comparison;
  - memory by phase;
  - optional error-rate chart if errors appear.
- Write `report/report.md`:
  - project goal and benchmark surface;
  - implementation summary;
  - methodology and environment;
  - cold-start results;
  - memory and isolation results;
  - throughput and tail-latency results;
  - limitations;
  - final recommendation about Wasm as a serverless isolation substrate.
- Make the limitations explicit:
  - Spin service is a Meilisearch-compatible subset, not native Meilisearch;
  - ranking behavior may differ;
  - in-memory Spin storage resets between runs;
  - Spin and OCI memory are observed through different mechanisms;
  - local results are not a full production-platform benchmark.
- Reproducibility QA:
  - start from a clean shell;
  - follow README commands for Spin and OCI;
  - regenerate at least one plot from raw CSV;
  - confirm links to `requirements.md`, `PROJECT_PLAN_3W.md`, `docs/`, and `report/` work.
- Prepare demo notes:
  - run Spin service;
  - run OCI baseline;
  - load fixture;
  - execute one benchmark;
  - show one key plot and one-sentence conclusion.

**Exit:** Submission-ready report, plots, raw data, reproducibility instructions, and clear conclusion.

## Success checklist

- [ ] Spin/wasmtime Meilisearch-compatible MVP builds and runs locally.
- [ ] MVP endpoints are implemented on Spin.
- [ ] Official pinned `getmeili/meilisearch` OCI baseline runs locally.
- [ ] Same fixture data and query set are used for both systems.
- [ ] Cold-start latency measured for both systems.
- [ ] Memory usage and isolation characteristics measured or documented for both systems.
- [ ] Throughput, latency percentiles, and error rates measured for both systems.
- [ ] Raw CSVs, processed summaries, and plots are stored in the repo.
- [ ] Final report answers cold start, memory, throughput, isolation, practicality, and limitations.
- [ ] README and methodology docs provide enough commands to reproduce the main results.
