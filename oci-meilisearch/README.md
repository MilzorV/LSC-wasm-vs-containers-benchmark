# OCI Meilisearch Baseline

This directory runs the native Meilisearch baseline for the Spin/wasmtime comparison.

Pinned image:

```text
getmeili/meilisearch:v1.43.0
```

This image tag intentionally matches the upstream source fetched by `scripts/fetch-meilisearch.sh`:

```text
meilisearch/meilisearch tag v1.43.0
observed commit 475ed56e5612df0dbb826748add5f93e0e7d5500
```

The OCI service is the native reference implementation. The Spin side must either reuse the same pinned source where possible or document any porting deviation.

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
