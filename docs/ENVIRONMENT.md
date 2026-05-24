# Environment Record

This file captures local tool versions observed during the Week 1 scaffold and later movie-search pivot. Re-record these values before collecting final benchmark results.

| Item | Value |
|---|---|
| Date | 2026-05-11 |
| OS | Darwin 24.6.0 arm64 |
| Hostname | Miloszs-MacBook-Air.local |
| CPU | Not captured in sandbox (`sysctl` denied) |
| RAM | Not captured in sandbox (`sysctl` denied) |
| Rust stable | `rustc 1.95.0 (59807616e 2026-04-14)` |
| Upstream Rust toolchain | `1.91.1-aarch64-apple-darwin` installed by upstream Meilisearch |
| Installed WASI target | `wasm32-wasip2` |
| Spin | `spin 3.6.3 (88d51cf 2026-04-09)` |
| Docker | `Docker version 28.5.1, build e180ab8` |
| Docker Compose | `Docker Compose version v2.40.2-desktop.1` |
| Python | `Python 3.12.7` |
| OCI image | local `lsc-movie-search-oci:latest` |
| Shared core | `movie-search-core` |
| Fixture | `fixtures/movies.json`, 44,471 documents |
| Historical upstream source tag | `v1.43.0` |
| Historical upstream source commit | `475ed56e5612df0dbb826748add5f93e0e7d5500` |
| Git base commit at scaffold time | `f968ad9` |

## Verification artifacts

- Native upstream check log: `docs/upstream-native-check.log` (ignored by Git).
- Layered WASI report: `docs/upstream-wasi-blockers.md` (tracked).
- Per-package WASI logs: `docs/upstream-wasi-check-*.log` (ignored by Git).

Before Week 2 benchmark collection, update this file with:

- exact CPU model and core count;
- RAM;
- power/performance mode;
- Git commit SHA of the benchmarked code;
- whether the benchmark used the shared movie-search service or a historical Meilisearch feasibility run.
