# Environment Record

This file captures the local environment for final benchmark collection. Re-record these values before publishing final numbers.

| Item | Value |
|---|---|
| Date recorded | 2026-05-24 03:38:35 CEST |
| OS | Darwin 24.6.0 arm64 |
| Kernel | Darwin Kernel Version 24.6.0 |
| Hostname | Miloszs-MacBook-Air.local |
| CPU | Apple M4 |
| CPU cores | 10 physical / 10 logical |
| RAM | 24 GiB |
| Power mode | macOS low power mode off (`lowpowermode 0`) |
| Rust stable | `rustc 1.95.0 (59807616e 2026-04-14)` |
| Installed WASI target | `wasm32-wasip2` |
| Spin | `spin 3.6.3 (88d51cf 2026-04-09)` |
| Docker | `Docker version 28.5.1, build e180ab8` |
| Docker Compose | `Docker Compose version v2.40.2-desktop.1` |
| Python | `Python 3.12.7` |
| Matplotlib | `3.10.8` |
| Git base commit | `e028269` |
| Benchmark code state | working tree includes local benchmark harness changes until committed |
| OCI image | local `lsc-movie-search-oci:latest` |
| Shared core | `movie-search-core` |
| Fixture | `fixtures/movies.json`, 44,471 documents |

## Historical Verification Artifacts

- Native upstream Meilisearch check log: `docs/upstream-native-check.log`.
- Layered WASI report: `docs/upstream-wasi-blockers.md`.
- Per-package WASI logs: `docs/upstream-wasi-check-*.log`.

These are retained as evidence for the earlier Meilisearch feasibility path, not as the primary movie-search benchmark.
