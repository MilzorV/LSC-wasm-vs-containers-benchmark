# Final Report: Movie Search on Spin/WASM vs OCI

The authoritative final report is the LaTeX version:

- source: `report/final-report.tex`;
- PDF for submission: `report/final-report.pdf`.

This Markdown file is a short index and summary of the package.

## Scope

The project compares the same HTTP microservice `movie-search` under two isolation models:

- Spin/wasmtime as a WASI HTTP component;
- a native Rust server running in an OCI container.

We abandoned a direct comparison with Meilisearch because official Meilisearch relies on LMDB,
memory-mapped storage, and native runtime assumptions that could not be ported 1:1 to Spin/WASI
within the project.

## Final artifacts

- Report: `report/final-report.pdf`.
- Editable presentation: `presentation/movie-search-spin-vs-oci.pptx`.
- Presentation PDF: `presentation/movie-search-spin-vs-oci.pdf`.
- Demo: `demo/demo-script.md`.
- Raw results: `results/raw/`.
- Processed results: `results/processed/`.
- Charts: `results/plots/`.

## Key results

- Functional parity: Spin and OCI return the same `hit.id` for queries `space`, `toy story`,
  `dark knight`, `romance`, and the empty query.
- Cold start `/health` p95: OCI `28.420 ms`, Spin `144.220 ms`.
- Cold start + first `/search` p95: OCI `376.983 ms`, Spin `217.504 ms`.
- Empty query, concurrency 10: OCI `3314.9 req/s`, Spin `132.8 req/s`.
- Query `space`, concurrency 10: OCI `54.0 req/s`, Spin `77.2 req/s`.
- Memory load max: OCI `31.9 MiB` via `docker stats`, Spin `493.5 MiB` as host process RSS.

## Reproduction

Full benchmark:

```bash
benchmarks/run_all.sh
```

Build the report:

```bash
latexmk -pdf -outdir=report -interaction=nonstopmode -halt-on-error report/final-report.tex
```

The presentation demo is described in `demo/demo-script.md`. The demo does not run the full
benchmark live; it shows functional parity and the final result artifacts.
