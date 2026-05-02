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

### Week 1 working log

Status, gaps, and learnings from implementation so far (keep this updated through Week 1 exit).

#### Done (current checkpoint)

**Repository layout**

- Directories in place: `spin-bartholomew/`, `oci-baseline/`, `benchmarks/`, `results/{raw,processed,plots}/`, `analysis/`, `report/`, `docs/`.
- Root `.gitignore` matches the `v1` pattern (venv, pycache, generated results/plots where applicable).
- Empty dirs tracked with `.gitkeep` where needed.

**Wasm workload (Bartholomew on Spin)**

- Bartholomew tree vendored from workspace scaffold `../v1/spin-bartholomew/` (content, templates, `spin.toml`, static assets, Rhai scripts, etc.).
- Benchmark routes aligned with requirements: `/`, `/blog/example`, `/static/style.css`.
- **Smoke test (local):** `spin up --listen 127.0.0.1:8080` — all three URLs returned HTTP 200 (Spin 3.6.x).

**OCI baseline**

- nginx image + `docker-compose.yml` (host **8081** → container 80), `default.conf`, and `site/` mirroring the same three paths.
- **Smoke test:** `docker compose up -d --build` — same three URLs returned HTTP 200.

**Documentation (partial Week 1)**

- [`docs/ENVIRONMENT.md`](docs/ENVIRONMENT.md): template for host, tool versions, pre-run checklist.
- [`README.md`](README.md): minimal “run Spin” / “run OCI” + `curl` checks (not yet full prerequisites + benchmark/analysis flow).

#### Still to do for Week 1 exit

Week 1 is complete when *both stacks run*, *scripts emit valid raw CSVs*, and *methodology + environment* are documented (see **Exit** above).

| Item | Notes |
|------|--------|
| [`docs/METHODOLOGY.md`](docs/METHODOLOGY.md) | Missing: fairness rules, cold-start definition, concurrency grid `10, 50, 100, 200`, repetition counts (e.g. ≥20 cold starts). Port from `../v1/docs/METHODOLOGY.md` and adapt repo name/paths. |
| `benchmarks/` harness | Currently only placeholders. Need: `common.sh`, `cold_start.sh`, `throughput.sh`, `memory.sh`, `run_all.sh`, `loadgen.py`, env vars (`COLD_RUNS`, `DURATION_SEC`, `CONCURRENCIES`, `MEMORY_SAMPLES`). Port from `../v1/benchmarks/`. |
| [`README.md`](README.md) | Extend: prerequisites (Spin, Docker, Python), how to run `run_all.sh` / individual scripts, where raw CSVs land, pointer to analysis (once `requirements-analysis.txt` or `pyproject` exists). |
| **Smoke test (scripts)** | Short dry runs through the harness; confirm CSV headers/schema and no port leaks between Spin (8080) and OCI (8081). |
| [`docs/ENVIRONMENT.md`](docs/ENVIRONMENT.md) | Fill in measured rows when you run a real campaign (versions, CPU, RAM, git SHA). |
| **Equivalence narrative** | OCI README points at methodology; once `METHODOLOGY.md` exists, ensure it states *same paths / comparable workload*, not identical implementation (static HTML vs Bartholomew rendering). |

Optional polish: `spin-bartholomew/README.md` still mentions default port 3000 in places; root README is the source of truth for **8080**.

#### Learnings (so far)

1. **Scaffold reuse worked.** Copying `v1`’s Spin + nginx trees avoided re-deriving URL parity; keep `v1` as reference until benchmarks/analysis are ported.

2. **Spin major version vs stack table.** The table above says Spin 2.x; **Spin 3.6.x** runs this manifest (`spin_version = "1"` in `spin.toml`) without changes. Document “tested with Spin 3.x” in `ENVIRONMENT.md` when recording versions.

3. **One listener per measurement.** Spin and OCI must not both bind 8080/8081 during a single benchmark cell; `common.sh` in `v1` encodes stop/teardown — important when scripting cold starts.

4. **Bartholomew log noise.** A **“Cannot create Cache file”** message appeared on `/blog/example` while still returning 200. Worth watching under load; may be permissions or cache path under `.spin/` — note in methodology/limitations if it persists.

5. **Process lifecycle vs automation.** Stopping Spin with `kill` on the port yields **exit 137** (SIGKILL) for a background `spin up` job — expected when tearing down after smoke tests, not an app crash.

6. **`ENVIRONMENT.md` vs repo reality.** The template includes `pip install -e ".[analysis]"`; this repo does not yet ship a matching `pyproject.toml` / installable package. Either add packaging in Week 1/2 or soften that section until `analysis/` dependencies are defined.

7. **nginx image pin.** Baseline uses `nginx:1.27-alpine` in the Dockerfile; the stack table says `nginx:stable` — both are fine; record the **actual** image tag in `ENVIRONMENT.md` for reproducibility.

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
