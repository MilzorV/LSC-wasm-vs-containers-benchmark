# Demo Script: Movie Search on Spin/WASM vs OCI

**Goal:** Show the same app on two runtimes, prove identical results, then show benchmark evidence.

**Live demo time:** ~4–5 minutes (fits a 10–12 min talk; leave benchmark *runs* for pre-recorded or backup slides).

**Story arc:** Standalone apps → side-by-side compare → benchmark dashboard.

---

## How to run (before the audience)

### Prerequisites

- Spin CLI 3.x, Rust `wasm32-wasip2`, Docker Compose, Node.js 22+, Python 3
- Repo root: `LSC-wasm-vs-containers-benchmark`

### One-time build (or after frontend changes)

```bash
make frontend-build    # embeds UI + copies dashboard.json/plots into dist/
cd spin-meili && spin build
cd oci-movie-search && docker compose build
```

Optional — refresh benchmark charts in the UI:

```bash
make analyze           # writes results/processed/dashboard.json + plots
make frontend-build    # re-embed updated dashboard data
```

### Three terminals during the demo

| Terminal | Command | Serves |
|----------|---------|--------|
| **1 — Spin** | `cd spin-meili && spin up --listen 127.0.0.1:8080` | WASM app + static UI on **:8080** |
| **2 — OCI** | `cd oci-movie-search && docker compose up` | Container app + static UI on **:8081** |
| **3 — Bench UI** (optional) | `make bench-ui` | Helper on **:8092** for “Run pilot” in browser |

### Health check (run once both are up)

```bash
curl -fsS http://127.0.0.1:8080/health && echo
curl -fsS http://127.0.0.1:8081/health && echo
```

Expected: `{"status":"available"}` for both.

### Quick smoke (optional confidence check)

```bash
make smoke
```

### URL cheat sheet

| What | URL |
|------|-----|
| Spin app (search + browse) | http://127.0.0.1:8080/spin |
| OCI app (search + browse) | http://127.0.0.1:8081/oci |
| Side-by-side compare | http://127.0.0.1:8080/ |
| Benchmark dashboard | http://127.0.0.1:8080/benchmarks |

Print URLs: `make demo`

---

## Live demo — three acts

### Act 1 — Two real applications (~1.5 min)

**Message:** Same product, two deployment models — not two different codebases.

1. Open **Spin app:** http://127.0.0.1:8080/spin  
   - Point out badge **Spin / WASM · :8080** and catalog line (~44k movies).
2. **Search** for `space` — scroll results, open one movie card (details dialog).
3. Turn on enhanced controls: set genre `Science Fiction`, year from `1970`, enable typo tolerance and highlights, then search for `spce`.
4. Show autocomplete suggestions while typing `dark kn`.
5. Switch to **Browse** — next page of catalog.
6. Open **OCI app:** http://127.0.0.1:8081/oci
   - Same UI, badge **OCI / Docker · :8081**.
7. Repeat `space` or the enhanced search — same titles/order on the first page.

**Say:** One Rust `movie-search-core`; Spin adapter vs OCI adapter. The Meilisearch-like features are demo features in the shared core, so they do not break benchmark fairness.

---

### Act 2 — Side-by-side runtime compare (~1 min)

**Message:** Under the hood, rankings must match — that’s our fairness guarantee.

1. Open http://127.0.0.1:8080/ (or `/demo`, same page).
2. Query `space` → click **Search both**.
3. Show both columns with latency (ms) and green **Match** badge.
4. Enable enhanced demo mode and search `spce`.
5. Show matching filtered/faceted/highlighted payloads, then try `toy story` or `dark knight` in legacy mode.

**Say:** Compare page calls `:8080` and `:8081` directly (CORS). Mismatch would mean a bug in shared core or adapter, including for enhanced payloads.

**Backup (terminal):**

```bash
benchmarks/compare_results.sh
# → Spin and OCI movie-search results match.
```

---

### Act 3 — Benchmarks (~1.5 min)

**Message:** We measured cold start, load, and memory — not just a hand-waved “WASM is faster.”

1. Open http://127.0.0.1:8080/benchmarks
2. Walk through:
   - **Spin vs OCI** metric cards (cold p95, load p95, throughput, host RSS).
   - **Plots** (cold start, latency, throughput, memory).
   - **Summary tables** (optional detail).
3. Do **not** run a full benchmark live unless you pre-tested timing.
4. Mention that load benchmarks now include `space`, empty, and `enhanced` scenarios.

**If `make bench-ui` is running:** mention **Run pilot** runs a shortened `run_all.sh` (~few min) — use only if you have time and a stable machine.

**Pre-generated artifacts (no live run needed):**

```bash
ls results/processed results/plots
open results/plots/cold_start_p95.png   # macOS
```

**Say:** Full harness: `make benchmark` or `benchmarks/run_all.sh`; analysis: `make analyze`. Plots in report/presentation match these files.

---

## Timing guide (4–5 min live UI only)

| Segment | Time | URL |
|---------|------|-----|
| Spin app: search + browse | ~45 s | `/spin` |
| OCI app: same flow | ~45 s | `/oci` |
| Compare: Search both + Match | ~60 s | `/` |
| Benchmark dashboard | ~90 s | `/benchmarks` |

---

## Talking points (one-liners)

- Same fixture: **44,471** movies in `fixtures/movies.json`.
- Spin: `wasm32-wasip2` + WASI HTTP; OCI: native binary in Docker.
- Search semantics are deterministic (token match, title > genre > overview, tie-break by id).
- Enhanced search adds filters, facets, sorting, highlights, typo tolerance, and `/suggest`, but only when requested.
- Benchmarks compare **systems**, not two different search engines.

---

## Plan B — if a runtime fails

1. Show the **other** app + **benchmarks** page (plots/CSV still valid).
2. Terminal: `results/processed/*.csv`, `results/plots/*.png`.
3. Slide: architecture + pre-captured plots from `presentation/` or `report/`.
4. Explain parity was verified with `make smoke` / `compare_results.sh` before the session.

---

## After the demo (not live)

```bash
make benchmark-pilot   # short end-to-end (~minutes)
make benchmark         # full study (longer)
make analyze           # refresh dashboard.json + plots
make frontend-build    # re-embed UI data, then rebuild Spin/OCI images
```

Stop services:

```bash
# Ctrl+C in Spin and compose terminals, or:
cd oci-movie-search && docker compose down
```

---

## Short recap

1. **Applications:** `/spin` and `/oci` — normal search, enhanced search, and browsing.
2. **Comparison:** `/` — Search both, enhanced mode, Match badge.
3. **Benchmarks:** `/benchmarks` — plots from `make analyze`; pilot runs only through `make bench-ui`.
4. **Startup:** two terminals (Spin + Docker), optional third terminal for `make bench-ui`.
