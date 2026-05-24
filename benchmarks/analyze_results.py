#!/usr/bin/env python3
"""Analyze raw benchmark CSV files and generate summary CSVs and plots."""

from __future__ import annotations

import csv
import json
import math
import statistics
from collections import defaultdict
from pathlib import Path

from benchmark_lib import RESULTS_PLOTS, RESULTS_PROCESSED, RESULTS_RAW, ensure_result_dirs


def main() -> int:
    ensure_result_dirs()
    cold_summary = summarize_cold_start()
    load_summary = summarize_load()
    memory_summary = summarize_memory()

    write_csv(RESULTS_PROCESSED / "cold_start_summary.csv", cold_summary)
    write_csv(RESULTS_PROCESSED / "load_summary.csv", load_summary)
    write_csv(RESULTS_PROCESSED / "memory_summary.csv", memory_summary)

    make_plots(cold_summary, load_summary, memory_summary)
    return 0


def summarize_cold_start() -> list[dict[str, object]]:
    groups: dict[tuple[str, str], list[dict[str, str]]] = defaultdict(list)
    for path in sorted(RESULTS_RAW.glob("cold_start_*.csv")):
        with path.open(newline="", encoding="utf-8") as fh:
            for row in csv.DictReader(fh):
                groups[(row["system"], row["scenario"])].append(row)

    rows: list[dict[str, object]] = []
    for (system, scenario), values in sorted(groups.items()):
        ready = [float(row["ready_ms"]) for row in values if row.get("ready_ms")]
        first_search = [
            float(row["first_search_ms"]) for row in values if row.get("first_search_ms")
        ]
        rows.append(
            {
                "system": system,
                "scenario": scenario,
                "metric": "ready_ms",
                **stats_for(values, ready),
            }
        )
        if first_search:
            rows.append(
                {
                    "system": system,
                    "scenario": scenario,
                    "metric": "first_search_ms",
                    **stats_for(values, first_search),
                }
            )
    return rows


def summarize_load() -> list[dict[str, object]]:
    groups: dict[tuple[str, str, str], list[dict[str, str]]] = defaultdict(list)
    durations: dict[tuple[str, str, str], float] = defaultdict(float)

    for path in sorted(RESULTS_RAW.glob("load_*.csv")):
        file_groups: set[tuple[str, str, str]] = set()
        with path.open(newline="", encoding="utf-8") as fh:
            for row in csv.DictReader(fh):
                key = (row["system"], row["query"], row["concurrency"])
                groups[key].append(row)
                file_groups.add(key)

        duration = read_load_duration(path)
        for key in file_groups:
            durations[key] += duration

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
        rows.append(
            {
                "system": system,
                "query": query,
                "concurrency": concurrency,
                "request_rate": f"{success_count / duration:.3f}",
                **stats_for(values, latencies),
            }
        )
    return rows


def summarize_memory() -> list[dict[str, object]]:
    groups: dict[tuple[str, str, str], list[dict[str, str]]] = defaultdict(list)
    for path in sorted(RESULTS_RAW.glob("memory_*.csv")):
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
            label_query = query if query else "empty"
            plt.plot(
                [int(row["concurrency"]) for row in series],
                [float(row["p95"]) for row in series],
                marker="o",
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
            label_query = query if query else "empty"
            plt.plot(
                [int(row["concurrency"]) for row in series],
                [float(row["request_rate"]) for row in series],
                marker="o",
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
    labels = [f"{row['system']} {row['phase']}" for row in plot_rows]
    values_mib = [float(row["max"]) / (1024 * 1024) for row in plot_rows]
    plt.figure(figsize=(8, 4))
    plt.bar(labels, values_mib)
    plt.ylabel("Peak memory (MiB)")
    plt.title("Memory samples")
    plt.xticks(rotation=20, ha="right")
    plt.tight_layout()
    path = RESULTS_PLOTS / "memory_peak.png"
    plt.savefig(path)
    plt.close()
    print(f"Wrote {path}")


if __name__ == "__main__":
    raise SystemExit(main())
