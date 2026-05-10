# Teacher Consultation Notes

Date: 2026-05-11

## Project topic

We are working on:

> WebAssembly Microservices with Spin/wasmtime: deploy a microservice application using Spin/WASI components and compare cold-start latency, memory isolation, and throughput against equivalent OCI containers.

Our concrete project is a **Meilisearch-compatible search microservice benchmark**:

- **Wasm side:** a subset-first search service implemented in Rust and running as a Spin/wasmtime HTTP component.
- **OCI side:** the official native `getmeili/meilisearch:v1.43.0` container.
- **Goal:** compare Wasm/Spin and OCI containers on cold start, memory behavior/isolation, throughput, and latency.

## What we have done

- Rewrote the project requirements and three-week plan around the Meilisearch-compatible benchmark scope.
- Built a Week 1 scaffold with:
  - `spin-meili/` Rust workspace for the Spin service;
  - `oci-meilisearch/` Docker Compose baseline;
  - `fixtures/documents.json` shared test fixture;
  - `benchmarks/smoke_spin.sh` and `benchmarks/smoke_oci.sh`;
  - `docs/ENVIRONMENT.md` and `docs/METHODOLOGY.md`.
- Implemented the Spin MVP endpoints:
  - `GET /health`
  - `GET /version`
  - `GET /indexes`
  - `POST /indexes/{uid}/documents`
  - `POST /indexes/{uid}/search`
  - `GET /stats`
  - `GET /tasks`
- Implemented basic deterministic search:
  - shared index UID: `movies`;
  - primary key: `id`;
  - supports `q`, `offset`, `limit`;
  - empty `q` works as placeholder search;
  - non-empty `q` matches case-insensitively over document text fields.
- Added Spin default key-value persistence for request-to-request continuity.
  - Local state is stored by Spin in `spin-meili/.spin/sqlite_key_value.db`.
  - Benchmark scripts should remove this file when they need a clean cold-start state.

## What already works

The Spin service builds and passes checks:

```bash
cd spin-meili
cargo fmt --check
cargo test --workspace
spin build
spin doctor
```

The Spin smoke test works:

```bash
spin up --listen 127.0.0.1:8080
benchmarks/smoke_spin.sh
```

The OCI baseline also works:

```bash
cd oci-meilisearch
docker compose up -d
cd ..
benchmarks/smoke_oci.sh
```

Both implementations accept the same fixture and return matching hits for the smoke query `space`.

## Current comparison surface

For Week 2 benchmarking, we plan to compare:

- cold start to first successful `/health`;
- search throughput for `POST /indexes/movies/search` with `{"q":"space"}`;
- placeholder search throughput for `POST /indexes/movies/search` with `{"q":""}`;
- memory at idle and under load;
- latency percentiles: p50, p95, p99;
- error rate under concurrency.

Planned concurrency levels:

```text
10, 50, 100, 200
```

## Important limitations

- The Spin service is **Meilisearch-compatible only for a small subset**, not a full Meilisearch port.
- Ranking behavior is intentionally simpler than native Meilisearch.
- The comparison is API-level on selected routes, not binary/runtime equivalence.
- Spin state currently uses local Spin key-value storage for practical smoke testing.
- Memory measurements will not be perfectly symmetric:
  - OCI has container/cgroup memory;
  - Spin is measured as a host process plus runtime behavior.

## Questions to ask the teacher

1. Is the chosen scope acceptable?
   - We are not porting all of Meilisearch; we are building a comparable subset workload and benchmarking it against native Meilisearch in Docker.

2. Is API-level equivalence enough for the OCI comparison?
   - The Wasm service implements selected Meilisearch-like endpoints, while OCI runs real Meilisearch.

3. Should we keep native Meilisearch as the OCI baseline, or add a simpler container baseline too?
   - Native Meilisearch is realistic but much more feature-complete than our subset.

4. For cold-start measurement, should startup include fixture loading?
   - Option A: measure pure service start to `/health`.
   - Option B: measure start plus fixture load plus first successful search.
   - These answer different questions.

5. How should we present the Spin key-value state in the report?
   - It is useful for request-to-request continuity, but clean benchmark runs need the local state file removed.

6. What is the expected depth of the memory isolation discussion?
   - We can compare WASI capabilities and wasmtime sandboxing against namespaces/cgroups/seccomp at a conceptual level, then connect it to measured memory overhead.

7. How large should the benchmark fixture be?
   - Current fixture is small for smoke tests.
   - Week 2 likely needs either a larger deterministic generated fixture or a real public dataset.

8. How many benchmark repetitions are expected?
   - Our draft says at least 20 cold starts and repeated throughput runs, but we should confirm if that is enough for the course.

9. Should we benchmark only locally, or is a Kubernetes/SpinKube deployment expected?
   - Current plan keeps Kubernetes out of scope for the MVP.

10. What final deliverable format is preferred?
    - Markdown report, PDF report, presentation slides, or all of these.

## Suggested next steps after consultation

- Confirm benchmark definition for cold start.
- Choose fixture size and dataset source.
- Implement Week 2 scripts:
  - `cold_start.sh`
  - `throughput.sh`
  - `memory.sh`
  - `run_all.sh`
  - optional Python load generator.
- Add result CSV schema and analysis script.
- Run the first small benchmark campaign and bring early numbers to the next consultation.
