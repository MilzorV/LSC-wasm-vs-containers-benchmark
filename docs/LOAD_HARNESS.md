# Load Harness Notes

The default load benchmark uses a Python `ThreadPoolExecutor` client in
[`benchmarks/run_load.py`](../benchmarks/run_load.py). A reference scenario with
[`hey`](https://github.com/rakyll/hey) is available in
[`benchmarks/run_load_hey.py`](../benchmarks/run_load_hey.py).

## Why two harnesses?

| Harness | Role |
|---|---|
| Python threaded client | Default path integrated with response validation and golden hit-id checks |
| `hey` | External reference tool for one representative scenario when installed |

The Python harness is not a dedicated load generator. It measures end-to-end
request latency from the client process, including HTTP client overhead. For the
movie-search workloads here, that overhead is small relative to server work for
`q=space`, but it can dominate empty-query scenarios where the server returns
quickly.

## Validation built into the Python harness

During load and cold-start runs, successful responses are validated against:

- HTTP 200 status;
- response schema;
- known `hit.id` ordering for `q=space` and empty `q`.

This catches silent correctness regressions that a raw throughput tool would miss.

## Reference `hey` run

Install:

```bash
go install github.com/rakyll/hey@latest
```

Run:

```bash
python3 benchmarks/run_load_hey.py --build-spin --build-oci
```

If `hey` is not installed, the script exits successfully after printing install
instructions. Full orchestration via `benchmarks/run_all.sh` treats a missing
`hey` as optional (`|| true`).

Output is written to `results/raw/load_hey_<timestamp>.csv`.

## Repeats and variance

`run_load.py` accepts `--repeats` (default `3` in full runs, `1` in pilot runs).
Each repeat restarts the target service and reruns all query/concurrency
scenarios. Analysis computes:

- aggregate latency percentiles across all repeats;
- per-repeat throughput standard deviation (`request_rate_stddev` in
  `results/processed/load_summary.csv`);
- error bars on throughput and latency plots when repeat variance exists.

## When to trust which number

- Use Python load results for apples-to-apples comparison with validation enabled.
- Use `hey` results as an external sanity check, not as the primary dataset.
- Interpret empty-query throughput with extra caution: client-side overhead and
  runtime HTTP stack differences affect this path more than full scans.
