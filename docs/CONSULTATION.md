# Teacher Consultation Notes

Date: 2026-05-11

## Project topic

We are working on:

> WebAssembly Microservices with Spin/wasmtime: deploy a microservice application using Spin/WASI components and compare cold-start latency, memory isolation, and throughput against equivalent OCI containers.

The project has pivoted from a custom Meilisearch-compatible subset to an attempted **upstream Meilisearch-on-Spin port**:

- **OCI side:** official `getmeili/meilisearch:v1.43.0`.
- **Source side:** upstream `meilisearch/meilisearch` tag `v1.43.0`, observed commit `475ed56e5612df0dbb826748add5f93e0e7d5500`.
- **Wasm side:** Spin/wasmtime HTTP component that should reuse as much upstream Meilisearch code as technically possible.
- **Fallback:** the existing custom Spin subset remains available only as a labeled fallback if upstream crates cannot be adapted within the project.

## What we have done

- Built the original Week 1 smoke scaffold:
  - `spin-meili/` Rust Spin workspace;
  - `oci-meilisearch/` Docker Compose baseline;
  - `fixtures/documents.json`;
  - `benchmarks/smoke_spin.sh` and `benchmarks/smoke_oci.sh`;
  - environment and methodology docs.
- Verified that both the Spin fallback and OCI baseline can accept the same fixture and search for `space`.
- Added upstream-port scaffolding:
  - `scripts/fetch-meilisearch.sh`;
  - `scripts/check-upstream-native.sh`;
  - `scripts/check-upstream-wasi.sh`;
  - `spin-meili/crates/meili-wasi-compat`;
  - renamed the custom search crate to `legacy-subset-core` so it is no longer presented as the main project core.
- Fetched upstream Meilisearch `v1.43.0` into `vendor/meilisearch/`.
- Ran a native upstream check for package `meilisearch`; it succeeds.
- Ran layered `wasm32-wasip2` checks; small crates pass, but `milli` and higher layers currently fail.
- Added browser-level mirror work:
  - Spin serves the same pinned Meilisearch mini-dashboard assets as OCI;
  - Spin and OCI both use `MASTER_KEY`;
  - Spin now implements dashboard-compatible document browsing, index stats, and settings routes for the selected benchmark surface.

## Current technical evidence

Native upstream check:

```bash
scripts/check-upstream-native.sh
```

Result: pass for pinned upstream `meilisearch`.

Layered WASI check:

```bash
scripts/check-upstream-wasi.sh
```

Current result:

- `flatten-serde-json`: pass;
- `filter-parser`: pass;
- `milli`: fail;
- `meilisearch-types`: fail;
- `routes`: fail;
- `meilisearch`: fail.

The first observed failures are in dependency/runtime layers such as `ring` C compilation for `wasm32-wasip2` and Tokio wasm feature constraints. LMDB/heed and memory-mapped storage are still expected deeper blockers because Meilisearch storage relies on LMDB and mmap.

Detailed evidence is in:

```text
docs/upstream-wasi-blockers.md
```

## Current benchmark surface

We now compare the same browser entrypoint plus the same benchmark API surface:

- browser dashboard at `/` on both Spin and OCI;
- cold start to first successful `/health`;
- search throughput for `POST /indexes/movies/search` with `{"q":"space"}`;
- placeholder search throughput for `POST /indexes/movies/search` with `{"q":""}`;
- document browsing via `GET /indexes/movies/documents`;
- memory at idle and under load;
- latency percentiles p50, p95, p99;
- error rate under concurrency.

Planned concurrency levels:

```text
10, 50, 100, 200
```

## Important limitations

- Full unmodified Meilisearch does not currently compile for `wasm32-wasip2` in our first feasibility check.
- The current Spin service serves the same mini-dashboard, but its backend logic is still the legacy subset fallback, not the upstream daemon.
- If we replace LMDB/mmap storage with a WASI-compatible layer, that must be treated as a porting deviation.
- Only a full or partial upstream reuse tier can support a same-source comparison.
- If we fall back to the subset, the result is still useful for Wasm-vs-container benchmarking, but not a same-application benchmark.

## Questions to ask the teacher

1. Is the new goal acceptable: attempt same pinned Meilisearch source/version first, then document any WASI blockers and deviations?

2. If full upstream Meilisearch cannot compile because of LMDB/mmap/runtime dependencies, is a partial port acceptable if it reuses upstream API/query/search layers and replaces storage?

3. Is the current blocker evidence enough to justify a fallback benchmark if the full port is not feasible within the course timeline?

4. Should the final comparison require same binary/application behavior, or is same API surface plus documented porting deviations acceptable?

5. Is browser-level parity with the same mini-dashboard and selected API workflows enough for the project, given the upstream daemon cannot currently compile to WASI?

6. Should cold start measure only first `/health`, or should it include fixture load and first successful search?

7. How large should the benchmark fixture be for the final run?

8. What level of memory isolation discussion is expected: conceptual WASI/container comparison, measured overhead, or both?

9. What final deliverable format is preferred: Markdown report, PDF, presentation slides, or all of these?

## Suggested next steps after consultation

- Decide whether to spend Week 2 on fixing WASI blockers or on the fallback benchmark harness.
- Try to reuse the highest passing upstream crates in the Spin adapter.
- If `milli` remains blocked, document the exact reason and decide whether a WASI-compatible storage abstraction is acceptable.
- Implement cold-start, throughput, and memory scripts.
- Run a small pilot benchmark and bring early numbers to the next consultation.
