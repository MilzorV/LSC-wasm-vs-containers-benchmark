#!/usr/bin/env python3
import argparse
import csv
import http.client
import json
import math
import os
import statistics
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from dataclasses import dataclass
from typing import Optional
from urllib.parse import urlparse


@dataclass(frozen=True)
class RequestSpec:
    method: str
    path: str
    content_type: Optional[str]
    body: bytes


def percentile(values, p: float) -> float:
    if not values:
        return float("nan")
    xs = sorted(values)
    if len(xs) == 1:
        return xs[0]
    # nearest-rank percentile
    k = math.ceil((p / 100.0) * len(xs)) - 1
    k = max(0, min(k, len(xs) - 1))
    return xs[k]


def make_spec(scenario: str, input_image: Optional[str]) -> RequestSpec:
    if scenario == "health":
        return RequestSpec("GET", "/health", None, b"")

    if scenario == "routes":
        return RequestSpec("GET", "/routes", None, b"")

    if scenario == "ping":
        return RequestSpec("GET", "/ping", None, b"")

    if scenario == "echo":
        return RequestSpec("POST", "/echo", "text/plain", b"hello benchmark")

    if scenario == "validate-json":
        return RequestSpec(
            "POST",
            "/validate/json",
            "application/json",
            b'{ "id": 1, "title": "The Matrix" }',
        )

    if scenario == "validate-json-schema":
        body = {
            "schema": {
                "type": "object",
                "required": ["id", "title"],
                "properties": {
                    "id": {"type": "integer"},
                    "title": {"type": "string"},
                    "year": {"type": "integer"},
                },
            },
            "document": {"id": 1, "title": "The Matrix", "year": 1999},
        }
        return RequestSpec(
            "POST",
            "/validate/json",
            "application/json",
            json.dumps(body, separators=(",", ":")).encode("utf-8"),
        )

    if scenario == "json-to-csv":
        body = [
            {"id": 1, "title": "The Matrix", "year": 1999},
            {"id": 2, "title": "Alien", "year": 1979},
            {"id": 3, "title": "Blade Runner", "year": 1982},
        ]
        return RequestSpec(
            "POST",
            "/convert/json-to-csv",
            "application/json",
            json.dumps(body, separators=(",", ":")).encode("utf-8"),
        )

    if scenario == "csv-to-json":
        body = "id,title,year\n1,The Matrix,1999\n2,Alien,1979\n3,Blade Runner,1982\n"
        return RequestSpec("POST", "/convert/csv-to-json", "text/csv", body.encode("utf-8"))

    if scenario in {"image-metadata", "image-grayscale", "image-resize"}:
        if not input_image:
            raise ValueError(f"{scenario} requires --input-image")
        with open(input_image, "rb") as f:
            body = f.read()

        if scenario == "image-metadata":
            return RequestSpec("POST", "/image/metadata", "image/png", body)
        if scenario == "image-grayscale":
            return RequestSpec("POST", "/image/grayscale?format=png", "image/png", body)
        if scenario == "image-resize":
            return RequestSpec("POST", "/image/resize?width=256&height=256&format=png", "image/png", body)

    raise ValueError(f"unknown scenario: {scenario}")


def send_once(base_url: str, spec: RequestSpec, timeout: float) -> dict:
    parsed = urlparse(base_url)
    if parsed.scheme != "http":
        raise ValueError("Only http:// URLs are supported by this simple benchmark script")

    host = parsed.hostname or "localhost"
    port = parsed.port or 80
    target = spec.path

    headers = {}
    if spec.content_type:
        headers["Content-Type"] = spec.content_type

    started = time.perf_counter_ns()
    status = 0
    nbytes = 0
    error = ""

    try:
        conn = http.client.HTTPConnection(host, port, timeout=timeout)
        conn.request(spec.method, target, body=spec.body if spec.body else None, headers=headers)
        resp = conn.getresponse()
        payload = resp.read()
        status = resp.status
        nbytes = len(payload)
        conn.close()
    except Exception as exc:
        error = f"{type(exc).__name__}: {exc}"

    ended = time.perf_counter_ns()
    elapsed_ms = (ended - started) / 1_000_000.0

    return {
        "status": status,
        "bytes": nbytes,
        "latency_ms": elapsed_ms,
        "error": error,
    }


def run_level(base_url: str, service: str, scenario: str, spec: RequestSpec, concurrency: int, requests: int, timeout: float):
    results = []
    wall_started = time.perf_counter()

    with ThreadPoolExecutor(max_workers=concurrency) as executor:
        futures = [
            executor.submit(send_once, base_url, spec, timeout)
            for _ in range(requests)
        ]
        for fut in as_completed(futures):
            results.append(fut.result())

    wall_ms = (time.perf_counter() - wall_started) * 1000.0

    ok = [r for r in results if 200 <= r["status"] < 300 and not r["error"]]
    failed = [r for r in results if not (200 <= r["status"] < 300) or r["error"]]
    latencies = [r["latency_ms"] for r in ok]

    summary = {
        "service": service,
        "scenario": scenario,
        "concurrency": concurrency,
        "requests": requests,
        "ok": len(ok),
        "failed": len(failed),
        "wall_ms": wall_ms,
        "rps": (len(ok) / (wall_ms / 1000.0)) if wall_ms > 0 else float("nan"),
        "min_ms": min(latencies) if latencies else float("nan"),
        "avg_ms": statistics.mean(latencies) if latencies else float("nan"),
        "median_ms": statistics.median(latencies) if latencies else float("nan"),
        "p90_ms": percentile(latencies, 90),
        "p95_ms": percentile(latencies, 95),
        "p99_ms": percentile(latencies, 99),
        "max_ms": max(latencies) if latencies else float("nan"),
        "bytes_avg": statistics.mean([r["bytes"] for r in ok]) if ok else float("nan"),
    }

    return summary, results


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--service", required=True, choices=["spin", "docker", "custom"])
    parser.add_argument("--base-url", required=True)
    parser.add_argument("--scenario", required=True)
    parser.add_argument("--concurrency", default="1,5,10,25,50")
    parser.add_argument("--requests", type=int, default=100)
    parser.add_argument("--warmup", type=int, default=10)
    parser.add_argument("--timeout", type=float, default=30.0)
    parser.add_argument("--input-image")
    parser.add_argument("--out-dir", default="bench-results")
    args = parser.parse_args()

    os.makedirs(args.out_dir, exist_ok=True)

    spec = make_spec(args.scenario, args.input_image)

    # Warmup is sequential by design, so it does not hide concurrency effects.
    for _ in range(max(0, args.warmup)):
        send_once(args.base_url, spec, args.timeout)

    levels = [int(x.strip()) for x in args.concurrency.split(",") if x.strip()]
    summaries = []

    timestamp = time.strftime("%Y%m%d-%H%M%S")
    raw_path = os.path.join(args.out_dir, f"raw-{args.service}-{args.scenario}-{timestamp}.csv")
    summary_path = os.path.join(args.out_dir, f"summary-{args.service}-{args.scenario}-{timestamp}.csv")

    with open(raw_path, "w", newline="", encoding="utf-8") as raw_file:
        raw_fields = [
            "service",
            "scenario",
            "concurrency",
            "request_no",
            "status",
            "bytes",
            "latency_ms",
            "error",
        ]
        writer = csv.DictWriter(raw_file, fieldnames=raw_fields)
        writer.writeheader()

        for level in levels:
            summary, results = run_level(
                args.base_url,
                args.service,
                args.scenario,
                spec,
                level,
                args.requests,
                args.timeout,
            )
            summaries.append(summary)

            for idx, result in enumerate(results, start=1):
                writer.writerow({
                    "service": args.service,
                    "scenario": args.scenario,
                    "concurrency": level,
                    "request_no": idx,
                    "status": result["status"],
                    "bytes": result["bytes"],
                    "latency_ms": f"{result['latency_ms']:.6f}",
                    "error": result["error"],
                })

            print(
                f"{args.service:6s} {args.scenario:20s} "
                f"c={level:<4d} ok={summary['ok']:<5d} failed={summary['failed']:<3d} "
                f"avg={summary['avg_ms']:.3f}ms p90={summary['p90_ms']:.3f}ms "
                f"p95={summary['p95_ms']:.3f}ms p99={summary['p99_ms']:.3f}ms "
                f"rps={summary['rps']:.1f}"
            )

    with open(summary_path, "w", newline="", encoding="utf-8") as summary_file:
        summary_fields = [
            "service",
            "scenario",
            "concurrency",
            "requests",
            "ok",
            "failed",
            "wall_ms",
            "rps",
            "min_ms",
            "avg_ms",
            "median_ms",
            "p90_ms",
            "p95_ms",
            "p99_ms",
            "max_ms",
            "bytes_avg",
        ]
        writer = csv.DictWriter(summary_file, fieldnames=summary_fields)
        writer.writeheader()
        for row in summaries:
            writer.writerow(row)

    print(f"\nRaw results:     {raw_path}")
    print(f"Summary results: {summary_path}")


if __name__ == "__main__":
    main()
