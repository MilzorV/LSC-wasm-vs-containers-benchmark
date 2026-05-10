# Methodology

## Week 1 smoke scope

Week 1 validates that the two services expose a comparable search surface before benchmark automation begins.

| System | URL | Purpose |
|---|---|---|
| Spin/wasmtime subset | `http://127.0.0.1:8080` | Wasm implementation under test |
| OCI Meilisearch | `http://127.0.0.1:8081` | Native container baseline |

Only one service should be benchmarked at a time in Week 2, but both can be run during Week 1 smoke checks if ports do not conflict.

## Fixture

The shared fixture is `fixtures/documents.json`.

- Index UID: `movies`
- Primary key: `id`
- Fields: `id`, `title`, `overview`, `genre`, `year`
- Smoke query: `space`

The Spin MVP keeps its search engine in memory while handling a request and snapshots that state into Spin's default key-value store between requests. With the local Spin runtime this appears under `spin-meili/.spin/sqlite_key_value.db`, so smoke scripts always load the fixture before searching and benchmark scripts should remove that file when they need a clean cold-start state.

## API surface

The Week 1 Spin MVP implements:

- `GET /health`
- `GET /version`
- `GET /indexes`
- `POST /indexes/{uid}/documents`
- `POST /indexes/{uid}/search`
- `GET /stats`
- `GET /tasks`

The OCI baseline uses the native Meilisearch API for comparable fixture ingestion and search. The comparison is API-level on the selected benchmark routes, not full implementation identity.

## Week 1 acceptance checks

Spin:

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
- search throughput for `POST /indexes/movies/search` with `{"q":"space"}`;
- placeholder search throughput for `POST /indexes/movies/search` with `{"q":""}`;
- idle and under-load memory for both systems.

Required concurrency levels for Week 2 are `10`, `50`, `100`, and `200`.
