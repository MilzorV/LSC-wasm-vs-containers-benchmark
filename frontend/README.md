# Movie Search Frontend

Vite multi-page frontend embedded by the Spin and OCI HTTP adapters.

## Scripts

```bash
npm install
npm run dev      # http://127.0.0.1:5173 — proxies API to Spin :8080
npm run build    # prebuild copies benchmark data; outputs dist/
```

## Pages

| Path | Purpose |
|------|---------|
| `/spin` | **Spin app** — search, browse, movie details (`:8080` API) |
| `/oci` | **OCI app** — search, browse, movie details (`:8081` API) |
| `/` | Runtime comparison — side-by-side Spin vs OCI search |
| `/benchmarks` | Benchmark dashboard — metrics, plots, run pilot/full via `make bench-ui` |
| `/demo` | Alias for `/` (legacy URL) |

Open the Spin app on the Spin host and the OCI app on the Docker host:

```text
http://127.0.0.1:8080/spin
http://127.0.0.1:8081/oci
```

Build `dist/` before `cargo build`, `spin build`, or Docker image build:

```bash
make analyze          # optional: refresh results/processed/dashboard.json
make frontend-build
```
