# Final Report: Movie Search on Spin/WASM vs OCI

The authoritative final report is the LaTeX version:

- source: `report/final-report.tex`;
- PDF for submission: `report/final-report.pdf`.

This Markdown file is a short index and summary of the package.

## Scope

The project compares the same HTTP microservice `movie-search` under two isolation models:

- Spin/wasmtime as a WASI HTTP component;
- a native Rust server running in an OCI container.

It also includes `spin-file-tools-sdk4` and `docker-file-tools` as a second workload that makes
the practical recommendation easier to explain.

We abandoned a direct comparison with Meilisearch because official Meilisearch relies on LMDB,
memory-mapped storage, and native runtime assumptions that could not be ported 1:1 to Spin/WASI
within the project.

To make the demo stronger without changing the fairness model, the shared core now includes
opt-in Meilisearch-inspired features: filters, facets, sorting, highlighting, suggestions, and
lightweight typo tolerance. Legacy benchmark payloads still work unchanged.

## Final artifacts

- Report: `report/final-report.pdf`.
- LaTeX presentation: `presentation/main.pdf`.
- Demo: `demo/demo-script.md`.
- Speaker plan: `presentation/speaker-plan.md`.
- Raw results: `results/raw/`.
- Processed results: `results/processed/`.
- Charts: `results/plots/`.

## Key results

- Functional parity: Spin and OCI return the same `hit.id` for queries `space`, `toy story`,
  `dark knight`, `romance`, and the empty query.
- Enhanced parity: both runtimes match for a filtered/faceted/highlighted typo-tolerant search
  payload and for `/suggest`.
- Cold start `/health` p95: OCI `38.700 ms`, Spin `202.689 ms`.
- Cold path through first `/search` p95: OCI `854.839 ms`, Spin `446.347 ms`.
- Empty query, concurrency 10: OCI `2224.6 req/s`, Spin `84.5 req/s`.
- Query `space`, concurrency 10: OCI `6.0 req/s`, Spin `16.8 req/s`.
- Enhanced query, concurrency 10: OCI `38.7 req/s`, Spin `50.8 req/s`.
- Load caveat: at `space`/c=200 there were client-side timeouts on both systems
  (`58` OCI, `40` Spin), while successful responses still validated.
- Memory load max: OCI `37.4 MiB` via `docker stats`, Spin `553.1 MiB` as host process RSS.
- File-tools c=50: Spin leads light routes such as `/health` (`3141.1` vs `1992.8` req/s),
  while Docker leads image resize (`594.1` vs `83.7` req/s).

## Reproduction

Full benchmark:

```bash
benchmarks/run_all.sh
```

The load harness records `space`, empty, and `enhanced` search scenarios.

Build the report:

```bash
latexmk -pdf -outdir=report -interaction=nonstopmode -halt-on-error report/final-report.tex
```

The presentation demo is described in `demo/demo-script.md`. The demo does not run the full
benchmark live; it shows functional parity and the final result artifacts.
