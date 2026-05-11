# LSC - Upstream Meilisearch on Spin/wasmtime vs OCI

This project investigates whether pinned upstream Meilisearch can be adapted to run inside **Fermyon Spin/wasmtime** and compared against the official **Meilisearch OCI container** on cold start, memory isolation/overhead, and HTTP throughput.

The current pivot is deliberately honest: the OCI side runs real `getmeili/meilisearch:v1.43.0`, while the Spin side first proves which parts of upstream `meilisearch/meilisearch:v1.43.0` compile for `wasm32-wasip2`. The existing custom Spin subset remains as a fallback benchmark workload until the upstream porting boundary is resolved.

Full specification: [requirements.md](requirements.md).

Three-week schedule: [PROJECT_PLAN_3W.md](PROJECT_PLAN_3W.md).

## Current status

- OCI baseline: `getmeili/meilisearch:v1.43.0`.
- Upstream source: fetched by script into `vendor/meilisearch/`.
- Observed upstream commit: `475ed56e5612df0dbb826748add5f93e0e7d5500`.
- Native upstream check: passes for package `meilisearch`.
- WASI layered check: small crates pass, `milli` and higher layers currently fail; see [docs/upstream-wasi-blockers.md](docs/upstream-wasi-blockers.md).
- Spin service: still builds and runs through the renamed legacy subset fallback.

## Prerequisites

Required tools:

- Spin CLI 3.x, currently recorded as `3.6.3`;
- Rust with `wasm32-wasip2`;
- Docker with Compose;
- Bash, curl, and Python 3.

Install the Rust target for the active toolchain:

```bash
rustup target add wasm32-wasip2
```

Meilisearch upstream uses its own pinned Rust toolchain. The first upstream cargo check may ask `rustup` to install that toolchain.

## Fetch and check upstream Meilisearch

Fetch pinned source:

```bash
scripts/fetch-meilisearch.sh
```

Run the native upstream check:

```bash
scripts/check-upstream-native.sh
```

Run the layered WASI feasibility check:

```bash
scripts/check-upstream-wasi.sh
```

The WASI check writes its tracked summary to:

```text
docs/upstream-wasi-blockers.md
```

Per-package logs are ignored by Git because they are large and machine-specific.

## Run the Spin service

The current Spin app is the legacy subset fallback while the upstream port is investigated.

```bash
cd spin-meili
spin build
spin up --listen 127.0.0.1:8080
```

From another shell:

```bash
benchmarks/smoke_spin.sh
```

The fallback service exposes the benchmark API surface:

- `GET /health`
- `GET /version`
- `GET /indexes`
- `POST /indexes/{uid}/documents`
- `POST /indexes/{uid}/search`
- `GET /stats`
- `GET /tasks`

The local Spin runtime stores fallback state in `spin-meili/.spin/sqlite_key_value.db`. Remove that file before a clean cold-start run.

## Run the OCI baseline

```bash
cd oci-meilisearch
docker compose up -d
```

From the repository root:

```bash
benchmarks/smoke_oci.sh
```

Reset the OCI baseline state:

```bash
cd oci-meilisearch
docker compose down -v
```

The OCI baseline uses `MEILI_MASTER_KEY=MASTER_KEY` and listens on `127.0.0.1:8081`.

## Shared fixture

The shared fixture is `fixtures/documents.json`.

- Index UID: `movies`
- Primary key: `id`
- Smoke query: `space`

Week 2 benchmark scripts should keep using this same fixture and API surface unless the methodology explicitly records a larger generated dataset.
