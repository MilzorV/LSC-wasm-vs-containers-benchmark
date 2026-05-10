# OCI Meilisearch Baseline

This directory runs the native Meilisearch baseline for the Spin/wasmtime comparison.

Pinned image:

```text
getmeili/meilisearch:v1.43.0
```

Start the baseline:

```bash
cd oci-meilisearch
docker compose up -d
```

Health check:

```bash
curl -fsS http://127.0.0.1:8081/health
```

Reset all container state:

```bash
cd oci-meilisearch
docker compose down -v
```

The configured master key is `MASTER_KEY`. Benchmark and smoke scripts use `Authorization: Bearer MASTER_KEY` for protected routes.
