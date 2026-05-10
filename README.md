# LSC - Meilisearch-compatible Spin/wasmtime vs OCI benchmark

Compare a **subset-first Meilisearch-compatible HTTP microservice** running on **Fermyon Spin/wasmtime** against the official **Meilisearch OCI container**. The project measures cold-start latency, memory isolation/overhead, and HTTP throughput to evaluate Wasm as a practical serverless isolation substrate.

Full specification: [requirements.md](requirements.md).

Three-week schedule: [PROJECT_PLAN_3W.md](PROJECT_PLAN_3W.md).

The implementation trees are not scaffolded yet. Week 1 of the plan adds the Spin service under `spin-meili/`, the OCI baseline under `oci-meilisearch/`, fixtures, benchmark scripts, analysis, and report folders.
