# Requirements: WebAssembly Microservices with Spin/wasmtime

## 1. Original assigned topic

> **11. WebAssembly Microservices with Spin/wasmtime**  
> Deploy a microservice application using the Spin framework (WASI components) and compare cold-start latency, memory isolation, and throughput against equivalent OCI containers, evaluating Wasm as a practical serverless isolation substrate.

This project is part of the **Large Scale Computing 2026** project list. The topic is about checking whether **WebAssembly (Wasm)**, executed with **Spin/wasmtime**, can be a practical alternative to traditional **OCI containers** for serverless-style workloads.

---

## 2. Chosen concrete project

### Chosen application: Bartholomew CMS on Spin

The selected application is **Bartholomew CMS**, a ready-made WebAssembly CMS application from Fermyon.

Bartholomew is suitable because it is not just a trivial `Hello World` function. It is a real web application that:

- serves dynamic pages,
- reads Markdown content,
- uses templates,
- serves static files,
- runs as a Spin/WebAssembly application,
- can be deployed locally with minimal modification.

The project will use Bartholomew as the main **Wasm/Spin workload** and compare it against an **equivalent OCI container-based web workload**.

---

## 3. Simple explanation of the project

The project is a comparison between two ways of running small web services:

1. **WebAssembly with Spin/wasmtime**  
   The application runs as lightweight Wasm components using the Spin framework.

2. **OCI containers with Docker/Podman**  
   The equivalent application runs as regular containers, similar to Docker containers used in Kubernetes or cloud platforms.

The goal is to answer:

> Can WebAssembly be a good serverless isolation substrate compared to containers?

In simple terms:

> Is Wasm a fast, safe, and lightweight way to run serverless microservices?

---

## 4. Project goals

The project must evaluate three main aspects required by the original topic.

### 4.1 Cold-start latency

Measure how long it takes for the application to become ready and serve the first request after it was not running before.

Examples of measurements:

- time from starting `spin up` to first successful HTTP response,
- time from starting a Docker container to first successful HTTP response,
- repeated cold starts to calculate average, minimum, maximum, and p95 latency.

Expected output:

- table with cold-start times,
- plot comparing Wasm/Spin and OCI containers,
- discussion of whether Wasm starts faster.

---

### 4.2 Memory isolation and memory overhead

Compare how much memory each approach uses and how isolation works.

Measurements:

- memory used by the Spin process,
- memory used by the Wasm application/component,
- memory used by the Docker container,
- memory used under low and high request load.

Qualitative comparison:

- Wasm isolation through runtime sandboxing and WASI capabilities,
- container isolation through namespaces, cgroups, filesystem isolation, and container runtime controls,
- discussion of which model is lighter and which is more mature.

Expected output:

- memory usage table,
- memory-over-time plot,
- short security/isolation discussion.

---

### 4.3 Throughput

Measure how many HTTP requests per second each implementation can handle.

Measurements:

- requests per second,
- average latency,
- p50 latency,
- p95 latency,
- p99 latency,
- error rate.

Benchmark scenarios:

- low concurrency, for example 10 concurrent clients,
- medium concurrency, for example 50 concurrent clients,
- high concurrency, for example 100 or 200 concurrent clients,
- dynamic page route,
- static asset route.

Expected output:

- throughput table,
- latency table,
- throughput-vs-concurrency plot,
- explanation of performance trade-offs.

---

## 5. In-scope work

The project should include the following work.

### 5.1 Deploy the Wasm/Spin application

Use the ready-made Bartholomew CMS template/application.

Required:

- install Spin CLI,
- run the Bartholomew Spin application locally,
- verify that the dynamic CMS route works,
- verify that the static file route works,
- document how to reproduce the setup.

The Wasm side should include at least two tested routes:

| Route type | Example | Purpose |
|---|---|---|
| Dynamic CMS page | `/` or `/blog/example` | Tests request-time rendering |
| Static file | `/static/style.css` | Tests static file serving |

---

### 5.2 Deploy an equivalent OCI container workload

Create an OCI container baseline that serves an equivalent website workload.

Recommended baseline:

- **Nginx or Caddy container** for static files,
- optionally a simple containerized dynamic Markdown/HTML server if we want a stronger comparison.

Minimum acceptable container baseline:

- same visible website/content,
- same static files,
- same benchmarked routes where possible,
- Dockerfile or Docker Compose file included.

Important note:

The container baseline does not need to run the exact same Wasm binary. It must provide an equivalent web-serving workload so that we can compare Wasm/Spin against traditional OCI deployment.

---

### 5.3 Benchmark both systems

Use the same machine and same benchmark tools for both systems.

Required benchmark categories:

1. cold-start latency,
2. memory usage,
3. throughput and latency under load.

Each benchmark should be repeated multiple times, for example 10-30 runs, so the results are not based on one lucky measurement.

---

### 5.4 Analyze results

The final report must answer:

- Which system starts faster?
- Which system uses less memory?
- Which system handles more requests per second?
- Which system has better tail latency?
- Is Wasm good enough as a serverless isolation substrate?
- What are the limitations of this experiment?

---

## 6. Out-of-scope work

The project should avoid becoming too large.

Not required:

- building a new CMS from scratch,
- creating a full production Kubernetes deployment,
- implementing a custom WebAssembly runtime,
- writing a new serverless platform,
- adding authentication, databases, payments, or user accounts,
- modifying Bartholomew deeply.

The goal is not to invent a new application. The goal is to deploy, benchmark, and evaluate Wasm versus OCI containers.

---

## 7. Required stack

### 7.1 WebAssembly / Spin side

Recommended tools:

- **Spin CLI** — to run and manage Spin applications,
- **wasmtime** — WebAssembly runtime ecosystem used for running Wasm/WASI components,
- **WASI components** — portable component model/interface layer,
- **Bartholomew CMS** — ready-made Spin/WebAssembly CMS workload,
- **Spin static file server** — for serving static files in the Spin application.

Languages likely involved:

- Rust, because Bartholomew is Rust/Wasm-based,
- TOML, for Spin and site configuration,
- Markdown, for CMS content,
- Handlebars/Rhai, if template behavior is inspected or lightly modified.

---

### 7.2 OCI container side

Recommended tools:

- **Docker** or **Podman**,
- **Dockerfile**,
- **Docker Compose** if multiple containers are used,
- **Nginx** or **Caddy** as a containerized web server baseline.

Optional:

- a small Node.js/Python/Go HTTP server container if we want a more dynamic baseline than Nginx.

---

### 7.3 Benchmarking stack

Recommended tools:

- `hyperfine` — repeated command timing, useful for cold-start benchmarks,
- `curl` — first-response checks,
- `wrk`, `hey`, or `bombardier` — HTTP load testing,
- `docker stats` — container memory and CPU usage,
- `ps`, `time`, or platform-specific tools — Spin process memory and CPU usage,
- Python with `pandas` and `matplotlib` — processing CSV results and generating plots.

---

### 7.4 Documentation/reporting stack

Recommended:

- Markdown for notes and reproducible commands,
- CSV files for raw benchmark results,
- Python notebooks or scripts for plots,
- final PDF/Markdown report,
- optional presentation slides.

---

## 8. Suggested repository structure

```text
wasm-spin-vs-oci/
├── README.md
├── requirements.md
├── spin-bartholomew/
│   ├── spin.toml
│   ├── content/
│   ├── templates/
│   ├── static/
│   └── ...
├── oci-baseline/
│   ├── Dockerfile
│   ├── docker-compose.yml
│   ├── site/
│   └── nginx.conf
├── benchmarks/
│   ├── cold_start.sh
│   ├── throughput.sh
│   ├── memory.sh
│   └── run_all.sh
├── results/
│   ├── raw/
│   ├── processed/
│   └── plots/
├── analysis/
│   └── analyze_results.py
└── report/
    ├── report.md
    └── figures/
```

---

## 9. Benchmark methodology

### 9.1 General rules

To keep the comparison fair:

- run both systems on the same machine,
- close unnecessary background applications,
- use the same HTTP benchmark tool,
- use the same route types,
- repeat every benchmark several times,
- save raw results,
- report hardware and OS details,
- separate cold-start tests from warm throughput tests.

---

### 9.2 Cold-start benchmark

For each system:

1. ensure the service is stopped,
2. start the service,
3. send requests until the first `200 OK`,
4. record elapsed time,
5. stop the service,
6. repeat 10-30 times.

Metrics:

- average cold-start time,
- median cold-start time,
- p95 cold-start time,
- minimum and maximum.

---

### 9.3 Throughput benchmark

For each system:

1. start the service,
2. warm it up briefly,
3. run load tests at different concurrency levels,
4. record requests per second and latency,
5. repeat for dynamic and static routes.

Example concurrency levels:

```text
10, 50, 100, 200
```

Metrics:

- requests per second,
- p50 latency,
- p95 latency,
- p99 latency,
- error rate.

---

### 9.4 Memory benchmark

For each system:

1. measure idle memory usage,
2. measure memory during low load,
3. measure memory during high load,
4. optionally measure memory after the benchmark to detect leaks or retained memory.

Metrics:

- idle RSS / memory usage,
- peak memory usage,
- average memory during load.

---

## 10. Expected deliverables

The final project should contain:

1. **Working Wasm/Spin deployment**
   - Bartholomew CMS runs locally through Spin.

2. **Working OCI container deployment**
   - Equivalent website workload runs through Docker/Podman.

3. **Benchmark scripts**
   - cold-start script,
   - throughput script,
   - memory measurement script.

4. **Raw benchmark results**
   - CSV or text files with all measurements.

5. **Processed results and plots**
   - cold-start comparison,
   - memory comparison,
   - throughput comparison,
   - latency comparison.

6. **Final report**
   - methodology,
   - results,
   - interpretation,
   - limitations,
   - conclusion about Wasm as a serverless isolation substrate.

7. **Demo instructions**
   - commands to run Spin version,
   - commands to run OCI version,
   - commands to reproduce benchmarks.

---

## 11. Evaluation questions

The final report should explicitly answer these questions.

### Cold start

- Does Spin/Wasm start faster than the container baseline?
- How large is the difference?
- Is the difference relevant for serverless workloads?

### Memory

- Does Spin/Wasm use less memory at idle?
- Does Spin/Wasm use less memory under load?
- How much memory overhead does each isolation model introduce?

### Throughput

- Which system handles more requests per second?
- Does Wasm lose throughput compared to containers?
- Does the result depend on static vs dynamic routes?

### Isolation

- What kind of isolation does Wasm provide?
- What kind of isolation do containers provide?
- Is Wasm isolation enough for multi-tenant serverless workloads?

### Practicality

- How easy was setup?
- How mature is the tooling?
- What problems were encountered?
- Would we recommend Wasm/Spin for this kind of serverless workload?

---

## 12. Success criteria

The project is successful if:

- Bartholomew CMS runs as a Spin/Wasm application,
- an equivalent OCI container baseline runs locally,
- cold-start latency is measured for both,
- memory usage is measured for both,
- throughput and latency are measured for both,
- results are shown in tables and plots,
- the report gives a clear conclusion about Wasm as a serverless isolation substrate.

---

## 13. Suggested division of work for two students

### Student A: Wasm/Spin side

Responsibilities:

- set up Spin,
- run Bartholomew,
- document Spin deployment,
- prepare Wasm benchmark commands,
- collect Spin/wasmtime memory and throughput results.

### Student B: OCI/container side and analysis

Responsibilities:

- prepare Docker/Podman baseline,
- document container deployment,
- prepare container benchmark commands,
- collect container memory and throughput results,
- process results and create plots.

Shared responsibilities:

- define fair benchmark methodology,
- write final report,
- prepare final presentation/demo,
- interpret whether Wasm is practical for serverless isolation.

---

## 14. Main risks and mitigations

| Risk | Why it matters | Mitigation |
|---|---|---|
| Bartholomew is not identical to the OCI baseline | Comparison may be criticized | Clearly describe the OCI workload as equivalent, not identical |
| Cold-start measurement is noisy | Results may vary | Repeat many times and report median/p95 |
| Wasm memory measurement may be harder than Docker memory measurement | Docker has built-in `docker stats` | Use OS process tools for Spin and document method |
| Nginx may be too optimized compared to Bartholomew | Throughput comparison may be unfair | Benchmark dynamic and static routes separately |
| Project becomes too big | Limited time | Do not build new app features; focus on deployment and measurement |

---

## 15. Final conclusion target

The final project should be able to conclude something like:

> WebAssembly with Spin/wasmtime provides a lightweight and fast-starting execution model for serverless-style microservices. Compared with OCI containers, it may reduce cold-start latency and memory overhead, but the practical result depends on workload type, runtime maturity, tooling, and how equivalent the container baseline is. For short-lived HTTP microservices, Wasm is a promising serverless isolation substrate, but containers remain more mature and broadly supported.
