# LSC - Meilisearch-compatible Spin/wasmtime vs OCI benchmark

Compare a **subset-first Meilisearch-compatible HTTP microservice** running on **Fermyon Spin/wasmtime** against the official **Meilisearch OCI container**. The project measures cold-start latency, memory isolation/overhead, and HTTP throughput to evaluate Wasm as a practical serverless isolation substrate.

Full specification: [requirements.md](requirements.md).

Three-week schedule: [PROJECT_PLAN_3W.md](PROJECT_PLAN_3W.md).

## Week 1 scaffold

The Week 1 scaffold includes:

- `spin-meili/` - Rust Spin HTTP service with the Meilisearch-compatible MVP API.
- `oci-meilisearch/` - Docker Compose baseline using `getmeili/meilisearch:v1.43.0`.
- `fixtures/documents.json` - shared movie fixture using primary key `id`.
- `benchmarks/smoke_spin.sh` and `benchmarks/smoke_oci.sh` - smoke checks for fixture loading and search.
- `docs/ENVIRONMENT.md` and `docs/METHODOLOGY.md` - starter reproducibility and benchmark notes.

## Prerequisites

The current shell already has Spin and Docker, but Rust was not installed when the scaffold was created. Install Rust before building the Spin component:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-wasip2
```

Required tools:

- Spin CLI 3.x
- Rust stable with `wasm32-wasip2`
- Docker or Podman with Compose
- Python 3 for smoke-script JSON parsing

## Run the Spin service

```bash
cd spin-meili
spin build
spin up --listen 127.0.0.1:8080
```

From another shell:

```bash
benchmarks/smoke_spin.sh
```

The Spin MVP implements:

- `GET /health`
- `GET /version`
- `GET /indexes`
- `POST /indexes/{uid}/documents`
- `POST /indexes/{uid}/search`
- `GET /stats`
- `GET /tasks`

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

## Fixture

The shared fixture is `fixtures/documents.json`.

- Index UID: `movies`
- Primary key: `id`
- Smoke query: `space`

Week 2 benchmark scripts should build on the same fixture and routes documented in [docs/METHODOLOGY.md](docs/METHODOLOGY.md).
