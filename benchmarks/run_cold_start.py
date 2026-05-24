#!/usr/bin/env python3
"""Measure cold start to /health and optionally first /search."""

from __future__ import annotations

import argparse
import csv
import traceback
import time
from pathlib import Path

from benchmark_lib import (
    RESULTS_RAW,
    ensure_result_dirs,
    parse_csv_arg,
    post_search,
    start_service,
    stop_service,
    timestamp_slug,
    wait_for_health,
)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--systems", default="spin,oci", help="Comma-separated systems: spin,oci")
    parser.add_argument("--iterations", type=int, default=20)
    parser.add_argument("--timeout", type=float, default=30.0)
    parser.add_argument("--include-first-search", action="store_true")
    parser.add_argument("--search-query", default="space")
    parser.add_argument("--build-oci", action="store_true")
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()

    ensure_result_dirs()
    systems = parse_csv_arg(args.systems)
    output = args.output or RESULTS_RAW / f"cold_start_{timestamp_slug()}.csv"
    scenario = "health_plus_search" if args.include_first_search else "health"

    with output.open("w", newline="", encoding="utf-8") as fh:
        writer = csv.DictWriter(
            fh,
            fieldnames=[
                "system",
                "scenario",
                "iteration",
                "success",
                "ready_ms",
                "first_search_ms",
                "error",
            ],
        )
        writer.writeheader()

        for system in systems:
            for iteration in range(1, args.iterations + 1):
                run_id = f"cold-{system}-{iteration}-{timestamp_slug()}"
                service = None
                success = False
                ready_ms = ""
                first_search_ms = ""
                error = ""
                started = time.perf_counter()

                try:
                    service = start_service(system, run_id, build_oci=args.build_oci)
                    ready_ms = f"{wait_for_health(service.url, args.timeout):.3f}"

                    if args.include_first_search:
                        valid, _status, _latency_ms, search_error = post_search(
                            service.url,
                            args.search_query,
                            timeout=args.timeout,
                        )
                        first_search_ms = f"{(time.perf_counter() - started) * 1000:.3f}"
                        if not valid:
                            raise RuntimeError(search_error)

                    success = True
                except Exception as exc:
                    error = f"{type(exc).__name__}: {exc}"
                    print(f"[cold-start] {system} iteration {iteration} failed: {error}")
                    traceback.print_exc()
                finally:
                    if service is not None:
                        stop_service(service)

                writer.writerow(
                    {
                        "system": system,
                        "scenario": scenario,
                        "iteration": iteration,
                        "success": str(success).lower(),
                        "ready_ms": ready_ms,
                        "first_search_ms": first_search_ms,
                        "error": error,
                    }
                )
                fh.flush()
                print(
                    f"[cold-start] {system} {iteration}/{args.iterations}: "
                    f"success={success} ready_ms={ready_ms} first_search_ms={first_search_ms}"
                )

    print(f"Wrote {output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
