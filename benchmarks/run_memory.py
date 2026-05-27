#!/usr/bin/env python3
"""Sample memory while the services are idle and under search load."""

from __future__ import annotations

import argparse
import csv
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
    post_search,
    sample_memory_metrics,
    start_service,
    stop_service,
    timestamp_slug,
    wait_for_health,
)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--systems", default="spin,oci")
    parser.add_argument("--idle-seconds", type=float, default=10.0)
    parser.add_argument("--load-seconds", type=float, default=30.0)
    parser.add_argument("--sample-interval", type=float, default=1.0)
    parser.add_argument("--load-concurrency", type=int, default=50)
    parser.add_argument("--query", default="space")
    parser.add_argument("--timeout", type=float, default=30.0)
    parser.add_argument("--build-oci", action="store_true")
    parser.add_argument("--build-spin", action="store_true")
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()

    ensure_result_dirs()
    systems = parse_csv_arg(args.systems)
    query = normalize_query(args.query)
    output = args.output or RESULTS_RAW / f"memory_{timestamp_slug()}.csv"

    with output.open("w", newline="", encoding="utf-8") as fh:
        writer = csv.DictWriter(
            fh,
            fieldnames=["system", "phase", "timestamp_ms", "memory_bytes", "source"],
        )
        writer.writeheader()

        for system in systems:
            run_id = f"memory-{system}-{timestamp_slug()}"
            service = start_service(
                system,
                run_id,
                build_oci=args.build_oci,
                build_spin=args.build_spin,
            )
            try:
                wait_for_health(service.url, args.timeout)
                post_search(service.url, "space", timeout=args.timeout)

                sample_phase(writer, fh, service, "idle", args.idle_seconds, args.sample_interval)
                run_load_and_sample(
                    writer=writer,
                    fh=fh,
                    service=service,
                    query=query,
                    duration=args.load_seconds,
                    sample_interval=args.sample_interval,
                    concurrency=args.load_concurrency,
                    timeout=args.timeout,
                )
            finally:
                stop_service(service)

    print(f"Wrote {output}")
    return 0


def sample_phase(
    writer: csv.DictWriter,
    fh,
    service,
    phase: str,
    duration: float,
    sample_interval: float,
) -> None:
    deadline = time.perf_counter() + duration
    while time.perf_counter() < deadline:
        write_memory_sample(writer, fh, service, phase)
        time.sleep(sample_interval)


def run_load_and_sample(
    writer: csv.DictWriter,
    fh,
    service,
    query: str,
    duration: float,
    sample_interval: float,
    concurrency: int,
    timeout: float,
) -> None:
    stop_event = threading.Event()
    request_ids = count(1)
    counters = {"successes": 0, "errors": 0}
    counters_lock = threading.Lock()

    def worker() -> None:
        while not stop_event.is_set():
            next(request_ids)
            success, _status, _latency_ms, _error = post_search(service.url, query, timeout=timeout)
            with counters_lock:
                if success:
                    counters["successes"] += 1
                else:
                    counters["errors"] += 1

    with ThreadPoolExecutor(max_workers=concurrency) as executor:
        futures = [executor.submit(worker) for _ in range(concurrency)]
        deadline = time.perf_counter() + duration
        while time.perf_counter() < deadline:
            write_memory_sample(writer, fh, service, "load")
            time.sleep(sample_interval)
        stop_event.set()
        for future in futures:
            future.result()

    print(
        f"[memory] system={service.system} load_successes={counters['successes']} "
        f"load_errors={counters['errors']}"
    )


def write_memory_sample(writer: csv.DictWriter, fh, service, phase: str) -> None:
    samples = sample_memory_metrics(service)
    if not samples:
        print(f"[memory] skipped sample system={service.system} phase={phase}")
        return
    for sample in samples:
        writer.writerow(
            {
                "system": service.system,
                "phase": phase,
                "timestamp_ms": int(time.time() * 1000),
                "memory_bytes": sample.memory_bytes,
                "source": sample.source,
            }
        )
    fh.flush()


def sample_memory(service):
    """Backward-compatible helper for tests and older imports."""
    samples = sample_memory_metrics(service)
    if not samples:
        return None, "unknown"
    first = samples[0]
    return first.memory_bytes, first.source


if __name__ == "__main__":
    raise SystemExit(main())
