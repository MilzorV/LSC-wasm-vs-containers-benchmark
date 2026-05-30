# Draft Speaker Plan

Target length: 12-15 minutes plus questions. Names are placeholders; assign them shortly
before presenting.

| Segment | Slides | Time | Speaker | What to say |
|---|---:|---:|---|---|
| Opening and assignment | 1-2 | 1.5 min | Speaker A | State the project question: can Spin/wasmtime be a practical serverless isolation substrate compared with OCI containers? |
| Background | 3-4 | 2 min | Speaker A | Explain WebAssembly, WASI, wasmtime, Spin, OCI, and Docker at a high level. Keep it conceptual, not implementation-heavy. |
| Movie-search architecture and features | 5-7 | 2 min | Speaker B | Emphasize fairness: one Rust core, two adapters, same fixture, same API. Explain enhanced search as demo value, not as a benchmark shortcut. |
| Methodology and parity | 8-9 | 1.5 min | Speaker B | Say that parity comes before performance. Mention smoke checks, same hit IDs, enhanced query parity, and `/suggest`. |
| Movie-search results | 10-12 | 3 min | Speaker C | Explain cold start, throughput, tail latency, and memory. Call out that OCI-empty dominates the throughput chart because it is a low-application-cost path. Mention c=200 timeouts as saturation evidence. |
| File-tools workload | 13-16 | 2.5 min | Speaker C or D | Explain why this second workload helps intuition: light JSON paths favor Spin, image-heavy paths favor Docker. Point to the structure diagram before results. |
| Recommendations and demo | 17-20/21 | 2 min | Speaker D | Summarize when to use Spin vs OCI, then run the short demo or show saved artifacts if services are not ready. |

Demo rule: do not run the full benchmark live. Show health/version/stats, one normal search,
one enhanced search, `/suggest`, one file-tools JSON route, one file-tools image route, and
then open saved plots/CSV summaries.

Backup line: "The full benchmark has already been run; the live demo proves functionality and
parity, while the report and plots show the measured results."
