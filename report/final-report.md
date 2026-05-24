# Movie Search on Spin/wasmtime vs OCI

## Summary

This project evaluates whether WebAssembly microservices running through Spin/wasmtime are a practical serverless isolation substrate compared with an equivalent OCI container. The benchmark application is a deterministic movie-search HTTP service implemented once in Rust and exposed through two adapters:

- Spin/WASI HTTP component;
- native Rust HTTP server packaged as an OCI container.

The benchmark intentionally uses one shared `movie-search-core` crate in both runtimes so performance differences are attributable to the runtime and isolation model rather than application logic differences.

## Pivot From Meilisearch

The project initially investigated running upstream Meilisearch on Spin. That path was not suitable for a same-application benchmark within the project timeline because upstream Meilisearch depends on native runtime layers and storage assumptions such as LMDB/heed and memory-mapped files. The Spin implementation available at that stage was a custom fallback with different ranking semantics, so comparing its search results with official Meilisearch would not have been honest.

The Meilisearch feasibility evidence is preserved in `docs/upstream-wasi-blockers.md`.

## Implementation Architecture

Both services embed the same `fixtures/movies.json` file with 44,471 deduplicated movie records. The shared core owns:

- loading and deduplication by `id`;
- query tokenization;
- deterministic substring matching;
- ranking by matched token count, field priority, and `id`;
- response types for health, version, stats, listing, and search.

The shared API surface is:

```text
GET  /health
GET  /version
GET  /stats
GET  /movies?offset=&limit=
POST /search
```

## Functional Parity Evidence

Before collecting performance data, both services must pass:

```bash
cargo test --manifest-path spin-meili/Cargo.toml
cd spin-meili && spin build
benchmarks/smoke_spin.sh
benchmarks/smoke_oci.sh
benchmarks/compare_results.sh
```

`compare_results.sh` validates matching engine identity, document count, total hits, and hit IDs for representative queries including `space`, `toy story`, `dark knight`, `romance`, and an empty query.

## Benchmark Methodology

The benchmark harness records:

- cold start to first successful `/health`;
- optional cold start plus first successful `/search`;
- throughput and latency for `POST /search`;
- idle and under-load memory samples.

Raw CSVs are written to `results/raw`, processed summaries to `results/processed`, and plots to `results/plots`.

Full benchmark command:

```bash
benchmarks/run_all.sh
```

Pilot command:

```bash
benchmarks/run_all.sh --pilot
```

## Cold-Start Results

Populate this section from:

- `results/processed/cold_start_summary.csv`
- `results/plots/cold_start_p95.png`

Discuss min, median, p95, and failures for Spin and OCI. Build time is excluded; Docker image build is completed before cold-start measurements.

## Throughput And Latency Results

Populate this section from:

- `results/processed/load_summary.csv`
- `results/plots/load_latency_p95.png`
- `results/plots/load_throughput.png`

Discuss request rate and p50/p95/p99 latency for `space` and empty-query scenarios at concurrency `10`, `50`, `100`, and `200`.

## Memory And Isolation Results

Populate this section from:

- `results/processed/memory_summary.csv`
- `results/plots/memory_peak.png`

Interpret memory numbers carefully. OCI memory is sampled through Docker/container metrics, while Spin memory is sampled from the host process tree RSS. These are useful for comparing observed overhead, but they are not identical measurement surfaces.

The qualitative isolation comparison should cover:

- OCI process/container isolation and cgroup resource accounting;
- Spin/WASI capability-oriented sandboxing;
- startup and deployment shape differences;
- operational tradeoffs for read-heavy microservices.

## Meilisearch Feasibility Appendix

The earlier Meilisearch feasibility work found that:

- small upstream crates such as `flatten-serde-json` and `filter-parser` compiled for `wasm32-wasip2`;
- higher layers failed through dependencies such as `ring`, Tokio wasm feature constraints, and expected deeper storage blockers;
- LMDB/heed and memory-mapped storage assumptions make a faithful Spin port non-trivial.

This supports the project pivot: the final benchmark compares the same movie-search application across runtimes, while Meilisearch remains an evidence-backed feasibility case study.

## Conclusion

Populate this section after final benchmark collection. Answer:

- whether Spin and OCI produced identical functional results;
- which runtime started faster;
- which runtime had better throughput and tail latency;
- how memory overhead differed;
- whether Spin/wasmtime looks practical for this class of microservice.
