# Methodology

## Current comparison model

The project now separates **porting feasibility** from **runtime benchmarking**.

| System | URL | Purpose |
|---|---|---|
| Spin/wasmtime | `http://127.0.0.1:8080` | Wasm implementation under test; currently backed by the legacy subset fallback until upstream reuse is resolved |
| OCI Meilisearch | `http://127.0.0.1:8081` | Official native baseline, `getmeili/meilisearch:v1.43.0` |

The goal is to move as much Spin behavior as possible onto upstream Meilisearch `v1.43.0` source. If that is blocked, the report must state the exact blocker and label any fallback benchmark results accordingly.

## Upstream feasibility checks

Fetch pinned source:

```bash
scripts/fetch-meilisearch.sh
```

Check native upstream buildability:

```bash
scripts/check-upstream-native.sh
```

Check selected upstream crates for `wasm32-wasip2`:

```bash
scripts/check-upstream-wasi.sh
```

The WASI report is [upstream-wasi-blockers.md](upstream-wasi-blockers.md). The current result is:

- `flatten-serde-json`: passes for `wasm32-wasip2`;
- `filter-parser`: passes for `wasm32-wasip2`;
- `milli`: fails before storage integration because dependencies such as `ring` do not build cleanly for this target in the current setup;
- `meilisearch-types`, `routes`, and `meilisearch`: fail through runtime/dependency layers including Tokio wasm feature constraints.

LMDB/heed and memory-mapped storage remain expected deeper blockers even after the current crypto/runtime build blockers are addressed.

## Fixture

The shared fixture is `fixtures/documents.json`.

- Index UID: `movies`
- Primary key: `id`
- Fields: `id`, `title`, `overview`, `genre`, `year`
- Smoke query: `space`

The OCI baseline uses native Meilisearch task processing. The current Spin fallback stores state through Spin's default key-value store under `spin-meili/.spin/sqlite_key_value.db`.

## Benchmark API surface

The benchmark surface is intentionally narrow:

- `GET /health`
- `GET /version`
- `GET /indexes`
- `POST /indexes/{uid}/documents`
- `POST /indexes/{uid}/search`
- `GET /stats`
- `GET /tasks`

Benchmark scripts should use the same fixture, index UID, primary key, and search request JSON for both systems. If Spin cannot exactly match native Meilisearch response fields, scripts should validate stable comparable fields such as hit IDs, hit count, status, and error rate.

## Week 1 acceptance checks

Spin fallback:

```bash
cd spin-meili
spin build
spin up --listen 127.0.0.1:8080
```

From another shell:

```bash
benchmarks/smoke_spin.sh
```

OCI:

```bash
cd oci-meilisearch
docker compose up -d
```

From the repository root:

```bash
benchmarks/smoke_oci.sh
```

Reset OCI state:

```bash
cd oci-meilisearch
docker compose down -v
```

## Benchmark surface for Week 2

The benchmark scripts should measure:

- cold start to first successful `/health`;
- optional cold start plus fixture load plus first successful search;
- search throughput for `POST /indexes/movies/search` with `{"q":"space"}`;
- placeholder search throughput for `POST /indexes/movies/search` with `{"q":""}`;
- idle and under-load memory for both systems;
- latency percentiles p50, p95, p99;
- error counts and response validation.

Required concurrency levels are `10`, `50`, `100`, and `200`.

## Interpretation rule

Results must be interpreted according to the Spin implementation tier:

- **full upstream:** Spin reuses the relevant pinned Meilisearch source layers;
- **partial upstream:** Spin reuses some upstream layers and replaces blocked storage/runtime boundaries;
- **legacy fallback:** Spin uses the custom subset only because upstream compilation was blocked.

Only the first tier supports a strong same-application comparison. The second and third tiers are still useful, but the report must frame them as porting evidence plus benchmark evidence.
