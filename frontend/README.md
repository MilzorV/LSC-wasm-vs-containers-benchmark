# Movie Search Frontend

Vite multi-page frontend for the movie-search HTTP service.

## Scripts

```bash
npm install
npm run dev      # http://127.0.0.1:5173 — proxies API to Spin :8080
npm run build    # outputs to dist/ (embedded by Rust adapters)
```

## Pages

| Path | Purpose |
|------|---------|
| `/` | Product UI — search, browse, movie details |
| `/demo` | Benchmark demo — side-by-side Spin vs OCI |

Build `dist/` before `cargo build`, `spin build`, or Docker image build:

```bash
make frontend-build
```
