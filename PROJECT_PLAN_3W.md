# Three-week project plan — Wasm/Spin vs OCI containers

This plan implements [requirements.md](requirements.md). Work is organized **by week** (no day-by-day schedule).

## Stack (fixed for the project)

| Layer | Choices |
|--------|---------|
| Wasm workload | **Spin CLI** (2.x), **wasmtime** (via Spin), **Bartholomew CMS** — add the app tree in Week 1 from [fermyon/bartholomew-site-template](https://github.com/fermyon/bartholomew-site-template) or by copying the working tree from `../v1/spin-bartholomew/` in this workspace |
| OCI baseline | **Docker** + **Docker Compose**, **nginx** (`nginx:stable`), equivalent URLs: `/`, `/blog/example`, `/static/style.css` |
| Benchmarks | Bash scripts: cold-start, throughput, memory, `run_all`; Python 3.11+ **stdlib** load generator (or optional `wrk` / `hey` / `bombardier`); optional **hyperfine** for CLI timing checks |
| Analysis | **Python** (`pandas`, `matplotlib`): CSV → plots + summary tables |
| Docs | **Markdown**: methodology, environment template, final report; optional **pandoc** PDF |

**Ports:** Spin on `127.0.0.1:8080`, OCI on `127.0.0.1:8081`; only one stack listening per measurement.

**Course reuse:** Scripted campaigns and archived raw results (Lab 4 — AWS load patterns), isolation vocabulary (Lab 6 — nginx, cgroups/namespaces), reproducible envs and analysis (Lab 2). Scaffold reference: `../v1/` (benchmarks, `docs/METHODOLOGY.md`, `analysis/`).

---

## Week 1 — Layout, workloads, harness

- Finalize repo layout to match requirements: `spin-bartholomew/`, `oci-baseline/`, `benchmarks/`, `results/raw|processed|plots/`, `analysis/`, `report/`, `docs/`.
- **Add Bartholomew** under `spin-bartholomew/` (template or `v1` copy); configure `spin.toml`, content, static assets; verify dynamic route (`/blog/example` or equivalent) and static route (`/static/style.css`).
- **OCI baseline:** `Dockerfile`, `docker-compose.yml`, nginx config and `site/` so paths and visible content align with the Spin app; document **equivalence** (same site shape, not same binary).
- Fill **`docs/ENVIRONMENT.md`**: OS, CPU, RAM, power, `spin --version`, `docker version`, `python3 --version`, `git rev-parse HEAD` for pinning results.
- Write **`docs/METHODOLOGY.md`**: fairness rules (same machine, same tool, repeated runs, cold vs warm separated), cold-start definition (stop → start → first HTTP 200 → teardown), concurrency grid `10, 50, 100, 200`, repetition counts (e.g. ≥20 cold starts per stack).
- Implement or port **`benchmarks/`**: `common.sh`, `cold_start.sh`, `throughput.sh`, `memory.sh`, `run_all.sh`, `loadgen.py`; env vars for `COLD_RUNS`, `DURATION_SEC`, `CONCURRENCIES`, `MEMORY_SAMPLES`.
- **`README.md`**: prerequisites, how to run each stack, how to run benchmarks and analysis.
- **Smoke test:** short dry runs; fix port leaks and CSV schema issues.

**Exit:** Both stacks run locally; scripts produce valid raw CSVs; methodology and environment documented.

---

## Week 2 — Full measurement campaign

- Run **cold-start** campaign per stack (target ≥20 iterations); export raw CSVs; compute median, mean, p95, min, max.
- Run **throughput** for each stack × route type (dynamic vs static) × concurrency; warmup before timed window; record RPS, p50/p95/p99, errors; repeat campaigns if variance is high.
- Run **memory**: idle, under load, post-load; OCI via `docker stats`; Spin via process RSS (`ps`) — document that Spin numbers are **directional** vs cgroup-backed container stats.
- **Quality pass:** validate CSVs, re-run failed cells only; archive runs with date + git SHA in filenames.
- Run **`analysis/analyze_results.py`** (or equivalent): processed summaries and plots (cold-start, throughput vs concurrency, latency, memory phases); copy figures into `report/figures/`.

**Exit:** Complete raw datasets, plots, and pinned commit for the measurement set.

---

## Week 3 — Report, limitations, demo

- Draft **`report/report.md`**: methodology, results tables, plots, interpretation.
- Answer explicitly: cold start, memory, throughput/tail latency, isolation (WASI vs containers), practicality (tooling, pitfalls, recommendation).
- **Limitations:** equivalence vs identity, measurement noise, nginx vs Bartholomew rendering asymmetry on “dynamic” routes, Spin memory sampling caveat.
- **Reproducibility QA:** clean follow of README from a fresh shell; regenerate at least one plot from raw CSVs.
- Optional: **`report/build_pdf.sh`** + pandoc/LaTeX if PDF is required for submission.
- **Demo script:** bullets for “run Spin → run OCI → run `run_all.sh` → show key figure + one-sentence takeaway.”

**Exit:** Submission-ready report, demo instructions, and conclusion on Wasm as a serverless-style isolation substrate.

---

## Success checklist (from requirements)

- [ ] Bartholomew runs under Spin locally with verified dynamic + static routes  
- [ ] Equivalent OCI baseline runs via Docker Compose  
- [ ] Cold-start, memory, and throughput/latency measured for both  
- [ ] Tables + plots in repo; report states limitations and clear conclusion  
