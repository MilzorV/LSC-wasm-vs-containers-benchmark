# LSC — Wasm/Spin vs OCI containers benchmark

Compare **Fermyon Spin + Bartholomew** to an **OCI (nginx)** baseline: cold start, memory, and HTTP throughput. Full specification: [requirements.md](requirements.md).

**Schedule:** [PROJECT_PLAN_3W.md](PROJECT_PLAN_3W.md) (three-week, week-by-week plan).

**Scaffold reference** (workloads + scripts you can copy or merge): `../v1/` in the parent `LSC` workspace.

## Run Spin (Wasm)

```bash
cd spin-bartholomew
spin up --listen 127.0.0.1:8080
```

Quick checks:

```bash
curl -i http://127.0.0.1:8080/
curl -i http://127.0.0.1:8080/blog/example
curl -i http://127.0.0.1:8080/static/style.css
```

## Run OCI baseline (nginx)

```bash
cd oci-baseline
docker compose up -d --build
```

Quick checks:

```bash
curl -i http://127.0.0.1:8081/
curl -i http://127.0.0.1:8081/blog/example
curl -i http://127.0.0.1:8081/static/style.css
docker compose down
```
