#!/usr/bin/env python3
"""Run throughput and latency benchmarks against POST /search."""

from __future__ import annotations

import argparse
import csv
import json
import threading
import time
from concurrent.futures import ThreadPoolExecutor
from itertools import count
from pathlib import Path

from benchmark_lib import (
    RESULTS_RAW,
    ensure_result_dirs,
    normalize_query,
    parse_csv_arg,
    parse_int_csv_arg,
    post_search,
    start_service,
    stop_service,
    timestamp_slug,
    wait_for_health,
)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--systems", default="spin,oci")
    parser.add_argument("--queries", default="space,empty")
    parser.add_argument("--concurrency", default="10,50,100,200")
    parser.add_argument("--duration", type=float, default=30.0)
    parser.add_argument("--timeout", type=float, default=30.0)
    parser.add_argument("--limit", type=int, default=10)
    parser.add_argument("--build-oci", action="store_true")
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()

    ensure_result_dirs()
    systems = parse_csv_arg(args.systems)
    queries = [normalize_query(query) for query in parse_csv_arg(args.queries)]
    concurrencies = parse_int_csv_arg(args.concurrency)
    output = args.output or RESULTS_RAW / f"load_{timestamp_slug()}.csv"
    meta_output = output.with_suffix(".json")

    with output.open("w", newline="", encoding="utf-8") as fh:
        writer = csv.DictWriter(
            fh,
            fieldnames=[
                "system",
                "query",
                "concurrency",
                "request_id",
                "success",
                "status",
                "latency_ms",
                "error",
            ],
        )
        writer.writeheader()
        writer_lock = threading.Lock()
        request_ids = count(1)

        for system in systems:
            run_id = f"load-{system}-{timestamp_slug()}"
            service = start_service(system, run_id, build_oci=args.build_oci)
            try:
                wait_for_health(service.url, args.timeout)
                post_search(service.url, "space", limit=args.limit, timeout=args.timeout)

                for query in queries:
                    for concurrency in concurrencies:
                        run_scenario(
                            writer=writer,
                            writer_lock=writer_lock,
                            request_ids=request_ids,
                            system=system,
                            url=service.url,
                            query=query,
                            concurrency=concurrency,
                            duration=args.duration,
                            timeout=args.timeout,
                            limit=args.limit,
                        )
                        fh.flush()
            finally:
                stop_service(service)

    meta = {
        "kind": "load",
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "duration_seconds": args.duration,
        "systems": systems,
        "queries": queries,
        "concurrency": concurrencies,
        "limit": args.limit,
    }
    meta_output.write_text(json.dumps(meta, indent=2) + "\n", encoding="utf-8")
    print(f"Wrote {output}")
    print(f"Wrote {meta_output}")
    return 0


def run_scenario(
    writer: csv.DictWriter,
    writer_lock: threading.Lock,
    request_ids: count,
    system: str,
    url: str,
    query: str,
    concurrency: int,
    duration: float,
    timeout: float,
    limit: int,
) -> None:
    deadline = time.perf_counter() + duration
    started = time.perf_counter()
    successes = 0
    errors = 0
    counter_lock = threading.Lock()

    def worker() -> None:
        nonlocal successes, errors
        while time.perf_counter() < deadline:
            request_id = next(request_ids)
            success, status, latency_ms, error = post_search(url, query, limit=limit, timeout=timeout)
            row = {
                "system": system,
                "query": query,
                "concurrency": concurrency,
                "request_id": request_id,
                "success": str(success).lower(),
                "status": status if status is not None else "",
                "latency_ms": f"{latency_ms:.3f}",
                "error": error,
            }
            with writer_lock:
                writer.writerow(row)
            with counter_lock:
                if success:
                    successes += 1
                else:
                    errors += 1

    with ThreadPoolExecutor(max_workers=concurrency) as executor:
        futures = [executor.submit(worker) for _ in range(concurrency)]
        for future in futures:
            future.result()

    elapsed = max(time.perf_counter() - started, 0.001)
    rate = successes / elapsed
    label = query if query else "<empty>"
    print(
        f"[load] system={system} query={label} concurrency={concurrency} "
        f"successes={successes} errors={errors} rate={rate:.2f}/s"
    )


if __name__ == "__main__":
    raise SystemExit(main())
