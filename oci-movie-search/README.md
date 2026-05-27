# OCI Movie Search

This directory builds and runs the native OCI baseline for the Spin/wasmtime comparison.

The container runs `spin-meili/crates/oci-http-adapter`, which uses the same `movie-search-core` crate and embedded `fixtures/movies.json` data as the Spin adapter.

Start the baseline:

```bash
docker compose up --build
```

Health check:

```bash
curl -fsS http://127.0.0.1:8081/health
```

Smoke check from the repository root:

```bash
benchmarks/smoke_oci.sh
```

Stop the baseline:

```bash
docker compose down
```
