#!/usr/bin/env python3
"""Analyze raw benchmark CSV files and generate summary CSVs and plots."""

from __future__ import annotations

import argparse
import csv
import json
import math
import statistics
from collections import defaultdict
from datetime import datetime, timezone
from pathlib import Path

from benchmark_lib import (
    RESULTS_PLOTS,
    RESULTS_PROCESSED,
    RESULTS_RAW,
    ensure_result_dirs,
    latest_raw_files,
)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--all-runs",
        action="store_true",
        help="Aggregate every raw CSV instead of only the latest best/latest run id per prefix.",
    )
    args = parser.parse_args()

    ensure_result_dirs()
    cold_summary = summarize_cold_start(use_latest_only=not args.all_runs)
    load_summary = summarize_load(use_latest_only=not args.all_runs)
    memory_summary = summarize_memory(use_latest_only=not args.all_runs)

    write_csv(RESULTS_PROCESSED / "cold_start_summary.csv", cold_summary)
    write_csv(RESULTS_PROCESSED / "load_summary.csv", load_summary)
    write_csv(RESULTS_PROCESSED / "memory_summary.csv", memory_summary)

    make_plots(cold_summary, load_summary, memory_summary)
    write_dashboard_json(cold_summary, load_summary, memory_summary)
    return 0


def write_dashboard_json(
    cold_summary: list[dict[str, object]],
    load_summary: list[dict[str, object]],
    memory_summary: list[dict[str, object]],
) -> None:
    plots: list[dict[str, str]] = []
    plot_specs = [
        ("cold_start_p95.png", "Cold start (p95 ready ms)"),
        ("load_latency_p95.png", "Load latency (p95)"),
        ("load_throughput.png", "Load throughput"),
        ("memory_peak.png", "Memory peak by source"),
        ("memory_peak_host_rss.png", "Host RSS memory"),
    ]
    for filename, title in plot_specs:
        path = RESULTS_PLOTS / filename
        if path.exists():
            plots.append({"path": f"plots/{filename}", "title": title})

    payload = {
        "generatedAt": datetime.now(timezone.utc).isoformat(),
        "cold": cold_summary,
        "load": load_summary,
        "memory": memory_summary,
        "plots": plots,
    }
    out = RESULTS_PROCESSED / "dashboard.json"
    out.write_text(json.dumps(payload, indent=2), encoding="utf-8")
    print(f"Wrote {out}")


def raw_paths(prefix: str, use_latest_only: bool) -> list[Path]:
    if use_latest_only:
        latest = latest_raw_files(prefix)
        return latest if latest else sorted(RESULTS_RAW.glob(f"{prefix}_*.csv"))
    return sorted(RESULTS_RAW.glob(f"{prefix}_*.csv"))


def summarize_cold_start(use_latest_only: bool = True) -> list[dict[str, object]]:
    groups: dict[tuple[str, str], list[dict[str, str]]] = defaultdict(list)
    for path in raw_paths("cold_start", use_latest_only):
        with path.open(newline="", encoding="utf-8") as fh:
            for row in csv.DictReader(fh):
                groups[(row["system"], row["scenario"])].append(row)

    rows: list[dict[str, object]] = []
    for (system, scenario), values in sorted(groups.items()):
        for metric in cold_metrics(values):
            rows.append(
                {
                    "system": system,
                    "scenario": scenario,
                    "metric": metric["name"],
                    **metric["stats"],
                }
            )
    return rows


def cold_metrics(values: list[dict[str, str]]) -> list[dict[str, object]]:
    metrics: list[dict[str, object]] = []
    metric_fields = [
        ("ready_ms", "ready_ms"),
        ("search_after_ready_ms", "search_after_ready_ms"),
        ("total_cold_path_ms", "total_cold_path_ms"),
        ("first_search_ms", "first_search_ms"),
    ]
    for name, field in metric_fields:
        samples = [
            float(row[field])
            for row in values
            if row.get(field)
        ]
        if not samples and field == "first_search_ms":
            samples = [
                float(row["first_search_ms"])
                for row in values
                if row.get("first_search_ms")
            ]
        if not samples and field == "total_cold_path_ms":
            samples = [
                float(row["first_search_ms"])
                for row in values
                if row.get("first_search_ms") and not row.get("total_cold_path_ms")
            ]
        if samples:
            metrics.append({"name": name, "stats": stats_for(values, samples)})
    return metrics


def summarize_load(use_latest_only: bool = True) -> list[dict[str, object]]:
    groups: dict[tuple[str, str, str], list[dict[str, str]]] = defaultdict(list)
    repeat_rates: dict[tuple[str, str, str], list[float]] = defaultdict(list)
    durations: dict[tuple[str, str, str], float] = defaultdict(float)

    for path in raw_paths("load", use_latest_only):
        duration = read_load_duration(path)
        repeat_groups: dict[tuple[str, str, str, str], list[dict[str, str]]] = defaultdict(list)
        with path.open(newline="", encoding="utf-8") as fh:
            for row in csv.DictReader(fh):
                key = (row["system"], row["query"], row["concurrency"])
                groups[key].append(row)
                repeat = row.get("repeat") or "1"
                repeat_groups[(row["system"], row["query"], row["concurrency"], repeat)].append(row)

        for repeat_key, repeat_rows in repeat_groups.items():
            system, query, concurrency, _repeat = repeat_key
            latencies = [
                float(row["latency_ms"])
                for row in repeat_rows
                if row.get("success") == "true" and row.get("latency_ms")
            ]
            if latencies:
                repeat_rates[(system, query, concurrency)].append(len(latencies) / max(duration, 0.001))
                durations[(system, query, concurrency)] += duration

    rows: list[dict[str, object]] = []
    for (system, query, concurrency), values in sorted(
        groups.items(), key=lambda item: (item[0][0], item[0][1], int(item[0][2]))
    ):
        latencies = [
            float(row["latency_ms"])
            for row in values
            if row.get("success") == "true" and row.get("latency_ms")
        ]
        success_count = len(latencies)
        duration = max(durations[(system, query, concurrency)], 0.001)
        rates = repeat_rates.get((system, query, concurrency), [])
        rate_stats = repeat_stats(rates)
        rows.append(
            {
                "system": system,
                "query": query,
                "concurrency": concurrency,
                "request_rate": f"{success_count / duration:.3f}",
                "request_rate_stddev": rate_stats["stddev"],
                "request_rate_runs": rate_stats["runs"],
                **stats_for(values, latencies),
            }
        )
    return rows


def summarize_memory(use_latest_only: bool = True) -> list[dict[str, object]]:
    groups: dict[tuple[str, str, str], list[dict[str, str]]] = defaultdict(list)
    for path in raw_paths("memory", use_latest_only):
        with path.open(newline="", encoding="utf-8") as fh:
            for row in csv.DictReader(fh):
                groups[(row["system"], row["phase"], row["source"])].append(row)

    rows: list[dict[str, object]] = []
    for (system, phase, source), values in sorted(groups.items()):
        memory = [float(row["memory_bytes"]) for row in values if row.get("memory_bytes")]
        rows.append(
            {
                "system": system,
                "phase": phase,
                "source": source,
                "metric": "memory_bytes",
                **stats_for(values, memory),
            }
        )
    return rows


def repeat_stats(samples: list[float]) -> dict[str, object]:
    if len(samples) <= 1:
        return {"stddev": "", "runs": len(samples)}
    return {
        "stddev": f"{statistics.pstdev(samples):.3f}",
        "runs": len(samples),
    }


def stats_for(all_rows: list[dict[str, str]], samples: list[float]) -> dict[str, object]:
    success_count = sum(1 for row in all_rows if row.get("success") == "true")
    if "success" not in (all_rows[0] if all_rows else {}):
        success_count = len(samples)
    error_count = max(len(all_rows) - success_count, 0)

    if not samples:
        return {
            "samples": len(all_rows),
            "success_count": success_count,
            "error_count": error_count,
            "min": "",
            "mean": "",
            "median": "",
            "p50": "",
            "p95": "",
            "p99": "",
            "max": "",
            "stddev": "",
        }

    ordered = sorted(samples)
    return {
        "samples": len(all_rows),
        "success_count": success_count,
        "error_count": error_count,
        "min": f"{ordered[0]:.3f}",
        "mean": f"{statistics.fmean(ordered):.3f}",
        "median": f"{statistics.median(ordered):.3f}",
        "p50": f"{percentile(ordered, 50):.3f}",
        "p95": f"{percentile(ordered, 95):.3f}",
        "p99": f"{percentile(ordered, 99):.3f}",
        "max": f"{ordered[-1]:.3f}",
        "stddev": f"{statistics.pstdev(ordered):.3f}" if len(ordered) > 1 else "0.000",
    }


def percentile(ordered: list[float], percent: int) -> float:
    if not ordered:
        return 0.0
    index = max(math.ceil((percent / 100) * len(ordered)) - 1, 0)
    return ordered[min(index, len(ordered) - 1)]


def read_load_duration(path: Path) -> float:
    meta_path = path.with_suffix(".json")
    if not meta_path.exists():
        return 30.0
    try:
        meta = json.loads(meta_path.read_text(encoding="utf-8"))
        return float(meta.get("duration_seconds", 30.0))
    except Exception:
        return 30.0


def write_csv(path: Path, rows: list[dict[str, object]]) -> None:
    if not rows:
        print(f"No rows for {path.name}; skipping")
        return
    fields: list[str] = []
    for row in rows:
        for key in row:
            if key not in fields:
                fields.append(key)
    with path.open("w", newline="", encoding="utf-8") as fh:
        writer = csv.DictWriter(fh, fieldnames=fields)
        writer.writeheader()
        writer.writerows(rows)
    print(f"Wrote {path}")


def query_label(query: object) -> str:
    value = str(query)
    if value == "":
        return "empty"
    if value == "enhanced":
        return "enhanced features"
    return value


def make_plots(
    cold_summary: list[dict[str, object]],
    load_summary: list[dict[str, object]],
    memory_summary: list[dict[str, object]],
) -> None:
    try:
        import matplotlib.pyplot as plt
    except Exception as exc:
        raise SystemExit(
            "matplotlib is required for plots. Install it with `python3 -m pip install matplotlib`."
        ) from exc

    plot_cold_start(plt, cold_summary)
    plot_load_latency(plt, load_summary)
    plot_load_throughput(plt, load_summary)
    plot_memory(plt, memory_summary)
    plot_memory_host_rss(plt, memory_summary)


def plot_cold_start(plt, rows: list[dict[str, object]]) -> None:
    ready = [row for row in rows if row.get("metric") == "ready_ms" and row.get("p95")]
    if not ready:
        return
    labels = [str(row["system"]) for row in ready]
    values = [float(row["p95"]) for row in ready]
    plt.figure(figsize=(6, 4))
    plt.bar(labels, values)
    plt.ylabel("p95 ready time (ms)")
    plt.title("Cold start to /health")
    plt.tight_layout()
    path = RESULTS_PLOTS / "cold_start_p95.png"
    plt.savefig(path)
    plt.close()
    print(f"Wrote {path}")


def plot_load_latency(plt, rows: list[dict[str, object]]) -> None:
    plot_rows = [row for row in rows if row.get("p95")]
    if not plot_rows:
        return
    plt.figure(figsize=(8, 5))
    for system in sorted({str(row["system"]) for row in plot_rows}):
        for query in sorted({str(row["query"]) for row in plot_rows}):
            series = [
                row for row in plot_rows if row["system"] == system and row["query"] == query
            ]
            if not series:
                continue
            series.sort(key=lambda row: int(row["concurrency"]))
            label_query = query_label(query)
            y = [float(row["p95"]) for row in series]
            yerr = [
                float(row["stddev"]) if row.get("stddev") not in ("", None) else 0.0
                for row in series
            ]
            plt.errorbar(
                [int(row["concurrency"]) for row in series],
                y,
                yerr=yerr,
                marker="o",
                capsize=3,
                label=f"{system} {label_query}",
            )
    plt.xlabel("Concurrency")
    plt.ylabel("p95 latency (ms)")
    plt.title("Search latency")
    plt.legend()
    plt.tight_layout()
    path = RESULTS_PLOTS / "load_latency_p95.png"
    plt.savefig(path)
    plt.close()
    print(f"Wrote {path}")


def plot_load_throughput(plt, rows: list[dict[str, object]]) -> None:
    plot_rows = [row for row in rows if row.get("request_rate")]
    if not plot_rows:
        return
    plt.figure(figsize=(8, 5))
    for system in sorted({str(row["system"]) for row in plot_rows}):
        for query in sorted({str(row["query"]) for row in plot_rows}):
            series = [
                row for row in plot_rows if row["system"] == system and row["query"] == query
            ]
            if not series:
                continue
            series.sort(key=lambda row: int(row["concurrency"]))
            label_query = query_label(query)
            y = [float(row["request_rate"]) for row in series]
            yerr = [
                float(row["request_rate_stddev"])
                if row.get("request_rate_stddev") not in ("", None)
                else 0.0
                for row in series
            ]
            plt.errorbar(
                [int(row["concurrency"]) for row in series],
                y,
                yerr=yerr,
                marker="o",
                capsize=3,
                label=f"{system} {label_query}",
            )
    plt.xlabel("Concurrency")
    plt.ylabel("Successful requests/s")
    plt.title("Search throughput")
    plt.legend()
    plt.tight_layout()
    path = RESULTS_PLOTS / "load_throughput.png"
    plt.savefig(path)
    plt.close()
    print(f"Wrote {path}")


def plot_memory(plt, rows: list[dict[str, object]]) -> None:
    plot_rows = [row for row in rows if row.get("max")]
    if not plot_rows:
        return
    labels = [f"{row['system']} {row['phase']} ({row['source']})" for row in plot_rows]
    values_mib = [float(row["max"]) / (1024 * 1024) for row in plot_rows]
    plt.figure(figsize=(10, 4))
    plt.bar(labels, values_mib)
    plt.ylabel("Peak memory (MiB)")
    plt.title("Memory samples by source")
    plt.xticks(rotation=25, ha="right")
    plt.tight_layout()
    path = RESULTS_PLOTS / "memory_peak.png"
    plt.savefig(path)
    plt.close()
    print(f"Wrote {path}")


def plot_memory_host_rss(plt, rows: list[dict[str, object]]) -> None:
    plot_rows = [
        row
        for row in rows
        if row.get("source") == "host_process_rss" and row.get("max")
    ]
    if not plot_rows:
        return
    labels = [f"{row['system']} {row['phase']}" for row in plot_rows]
    values_mib = [float(row["max"]) / (1024 * 1024) for row in plot_rows]
    plt.figure(figsize=(8, 4))
    plt.bar(labels, values_mib)
    plt.ylabel("Peak host RSS (MiB)")
    plt.title("Comparable host RSS memory")
    plt.xticks(rotation=20, ha="right")
    plt.tight_layout()
    path = RESULTS_PLOTS / "memory_peak_host_rss.png"
    plt.savefig(path)
    plt.close()
    print(f"Wrote {path}")


if __name__ == "__main__":
    raise SystemExit(main())
