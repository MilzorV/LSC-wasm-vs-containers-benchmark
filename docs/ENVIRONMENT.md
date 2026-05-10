# Environment Record

This file captures the local tool versions observed when the Week 1 scaffold was created. Re-record these values before collecting final benchmark results.

| Item | Value |
|---|---|
| Date | 2026-05-10 |
| OS | Darwin 24.6.0 arm64 |
| Hostname | Miloszs-MacBook-Air.local |
| CPU | Not captured in sandbox (`sysctl` denied) |
| RAM | Not captured in sandbox (`sysctl` denied) |
| Rust | Not installed in the current shell (`rustc`, `cargo`, `rustup` missing) |
| Spin | `spin 3.6.3 (88d51cf 2026-04-09)` |
| Docker | `Docker version 28.5.1, build e180ab8` |
| Docker Compose | `Docker Compose version v2.40.2-desktop.1` |
| Python | `Python 3.12.7` |
| Meilisearch OCI image | `getmeili/meilisearch:v1.43.0` |
| Git base commit at scaffold time | `f968ad9` |

Before Week 2 benchmark collection, update this file with:

- exact CPU model and core count;
- RAM;
- power/performance mode;
- Rust version after installation;
- Git commit SHA of the benchmarked code.
