#!/usr/bin/env python3
"""Run a reference load scenario with hey when available."""

from __future__ import annotations

import argparse
import csv
import json
import subprocess
import time
from pathlib import Path

from benchmark_lib import (
    RESULTS_RAW,
    CONTENT_TYPE_JSON,
    ensure_result_dirs,
    find_hey,
    normalize_query,
    parse_csv_arg,
    post_search,
    search_scenario_payload,
    start_service,
    stop_service,
    timestamp_slug,
    wait_for_health,
)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--systems", default="spin,oci")
    parser.add_argument("--query", default="space")
    parser.add_argument("--concurrency", type=int, default=10)
    parser.add_argument("--duration", type=float, default=10.0)
    parser.add_argument("--timeout", type=float, default=30.0)
    parser.add_argument("--limit", type=int, default=10)
    parser.add_argument("--build-oci", action="store_true")
    parser.add_argument("--build-spin", action="store_true")
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()

    hey = find_hey()
    if hey is None:
        print("hey is not installed; skipping reference load benchmark")
        print("Install with: go install github.com/rakyll/hey@latest")
        return 0

    ensure_result_dirs()
    systems = parse_csv_arg(args.systems)
    scenario = normalize_query(args.query)
    run_id = timestamp_slug()
    output = args.output or RESULTS_RAW / f"load_hey_{run_id}.csv"
    meta_output = output.with_suffix(".json")

    rows: list[dict[str, object]] = []
    for system in systems:
        service = start_service(
            system,
            f"load-hey-{system}-{run_id}",
            build_oci=args.build_oci,
            build_spin=args.build_spin,
        )
        try:
            wait_for_health(service.url, args.timeout)
            post_search(service.url, "space", limit=args.limit, timeout=args.timeout)
            rows.append(
                run_hey(
                    hey=hey,
                    system=system,
                    url=service.url,
                    scenario=scenario,
                    concurrency=args.concurrency,
                    duration=args.duration,
                    limit=args.limit,
                    run_id=run_id,
                )
            )
        finally:
            stop_service(service)

    with output.open("w", newline="", encoding="utf-8") as fh:
        writer = csv.DictWriter(
            fh,
            fieldnames=[
                "run_id",
                "tool",
                "system",
                "query",
                "concurrency",
                "duration_seconds",
                "request_rate",
                "latency_p50_ms",
                "latency_p95_ms",
                "latency_p99_ms",
                "success_count",
                "error_count",
            ],
        )
        writer.writeheader()
        writer.writerows(rows)

    meta = {
        "kind": "load_hey",
        "run_id": run_id,
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "duration_seconds": args.duration,
        "systems": systems,
        "query": scenario,
        "concurrency": args.concurrency,
        "limit": args.limit,
    }
    meta_output.write_text(json.dumps(meta, indent=2) + "\n", encoding="utf-8")
    print(f"Wrote {output}")
    print(f"Wrote {meta_output}")
    return 0


def run_hey(
    hey: str,
    system: str,
    url: str,
    scenario: str,
    concurrency: int,
    duration: float,
    limit: int,
    run_id: str,
) -> dict[str, object]:
    query, request_payload = search_scenario_payload(scenario, limit)
    payload = json.dumps(request_payload)
    command = [
        hey,
        "-z",
        f"{duration}s",
        "-c",
        str(concurrency),
        "-m",
        "POST",
        "-H",
        f"content-type: {CONTENT_TYPE_JSON['content-type']}",
        "-d",
        payload,
        f"{url}/search",
    ]
    result = subprocess.run(
        command,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        check=False,
    )
    if result.returncode != 0:
        raise RuntimeError(f"hey failed for {system}: {result.stdout}")

    parsed = parse_hey_output(result.stdout)
    label = scenario if scenario else "<empty>"
    print(
        f"[load-hey] run={run_id} system={system} query={label} "
        f"rate={parsed['request_rate']} req/s p95={parsed['latency_p95_ms']} ms"
    )
    return {
        "run_id": run_id,
        "tool": "hey",
        "system": system,
        "query": scenario,
        "concurrency": concurrency,
        "duration_seconds": duration,
        **parsed,
    }


def parse_hey_output(output: str) -> dict[str, object]:
    request_rate = ""
    latency_p50_ms = ""
    latency_p95_ms = ""
    latency_p99_ms = ""
    success_count = ""
    error_count = ""

    for line in output.splitlines():
        stripped = line.strip()
        if stripped.startswith("Requests/sec:"):
            request_rate = stripped.split(":", 1)[1].strip()
        elif stripped.startswith("200:"):
            success_count = stripped.split(":", 1)[1].strip()
        elif "[503]" in stripped or stripped.startswith("Error distribution:"):
            continue
        elif stripped.startswith("Slowest:") or stripped.startswith("Fastest:"):
            continue
        elif stripped.startswith("Average:"):
            continue
        elif stripped.startswith("Requests/sec"):
            request_rate = stripped.split(":", 1)[1].strip()

    for line in output.splitlines():
        stripped = line.strip()
        if stripped.startswith("50%"):
            latency_p50_ms = stripped.split("in", 1)[1].strip()
        elif stripped.startswith("95%"):
            latency_p95_ms = stripped.split("in", 1)[1].strip()
        elif stripped.startswith("99%"):
            latency_p99_ms = stripped.split("in", 1)[1].strip()

    return {
        "request_rate": request_rate,
        "latency_p50_ms": normalize_duration(latency_p50_ms),
        "latency_p95_ms": normalize_duration(latency_p95_ms),
        "latency_p99_ms": normalize_duration(latency_p99_ms),
        "success_count": success_count,
        "error_count": error_count,
    }


def normalize_duration(value: str) -> str:
    value = value.strip()
    if value.endswith("secs"):
        return f"{float(value[:-4].strip()) * 1000:.3f}"
    if value.endswith("sec"):
        return f"{float(value[:-3].strip()) * 1000:.3f}"
    if value.endswith("ms"):
        return f"{float(value[:-2].strip()):.3f}"
    return value


if __name__ == "__main__":
    raise SystemExit(main())
