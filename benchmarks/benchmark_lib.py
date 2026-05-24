#!/usr/bin/env python3
"""Shared helpers for the movie-search benchmark scripts."""

from __future__ import annotations

import json
import os
import signal
import subprocess
import time
import urllib.error
import urllib.request
from dataclasses import dataclass
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
SPIN_DIR = ROOT / "spin-meili"
OCI_DIR = ROOT / "oci-movie-search"
RESULTS_RAW = ROOT / "results" / "raw"
RESULTS_PROCESSED = ROOT / "results" / "processed"
RESULTS_PLOTS = ROOT / "results" / "plots"

SPIN_URL = "http://127.0.0.1:8080"
OCI_URL = "http://127.0.0.1:8081"
OCI_CONTAINER = "lsc-movie-search-oci"
CONTENT_TYPE_JSON = {"content-type": "application/json"}

EXPECTED_HIT_IDS = {
    "space": [62, 957, 1542, 2157, 2300, 2614, 5551, 6795, 7269, 7290],
    "": [2, 3, 5, 6, 11, 12, 13, 14, 15, 16],
}


@dataclass
class StartedService:
    system: str
    url: str
    process: subprocess.Popen[str] | None = None
    log_file: Any | None = None


def ensure_result_dirs() -> None:
    RESULTS_RAW.mkdir(parents=True, exist_ok=True)
    RESULTS_PROCESSED.mkdir(parents=True, exist_ok=True)
    RESULTS_PLOTS.mkdir(parents=True, exist_ok=True)


def timestamp_slug() -> str:
    return time.strftime("%Y%m%d-%H%M%S")


def parse_csv_arg(value: str) -> list[str]:
    return [part.strip() for part in value.split(",") if part.strip()]


def parse_int_csv_arg(value: str) -> list[int]:
    return [int(part) for part in parse_csv_arg(value)]


def normalize_query(value: str) -> str:
    return "" if value in {"empty", '""'} else value


def start_service(system: str, run_id: str, build_oci: bool = False) -> StartedService:
    if system == "spin":
        return start_spin(run_id)
    if system == "oci":
        return start_oci(build=build_oci)
    raise ValueError(f"unknown system: {system}")


def stop_service(service: StartedService) -> None:
    if service.system == "spin":
        stop_spin(service)
    elif service.system == "oci":
        stop_oci()


def start_spin(run_id: str) -> StartedService:
    ensure_result_dirs()
    log_path = RESULTS_RAW / f"spin-{run_id}.log"
    log_file = log_path.open("a", encoding="utf-8")
    process = subprocess.Popen(
        ["spin", "up", "--listen", "127.0.0.1:8080"],
        cwd=SPIN_DIR,
        stdout=log_file,
        stderr=subprocess.STDOUT,
        text=True,
        start_new_session=True,
    )
    return StartedService(system="spin", url=SPIN_URL, process=process, log_file=log_file)


def stop_spin(service: StartedService) -> None:
    process = service.process
    if process is not None and process.poll() is None:
        try:
            os.killpg(process.pid, signal.SIGTERM)
            process.wait(timeout=5)
        except Exception:
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except Exception:
                pass
            try:
                process.wait(timeout=5)
            except Exception:
                pass
    if service.log_file is not None:
        service.log_file.close()


def start_oci(build: bool = False) -> StartedService:
    stop_oci()
    command = ["docker", "compose", "up", "-d", "--force-recreate"]
    if build:
        command.append("--build")
    else:
        command.append("--no-build")
    run(command, cwd=OCI_DIR)
    return StartedService(system="oci", url=OCI_URL)


def stop_oci() -> None:
    subprocess.run(
        ["docker", "compose", "down", "--remove-orphans"],
        cwd=OCI_DIR,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        text=True,
        check=False,
    )


def run(command: list[str], cwd: Path = ROOT) -> subprocess.CompletedProcess[str]:
    return subprocess.run(command, cwd=cwd, text=True, check=True)


def wait_for_health(url: str, timeout_seconds: float = 30.0) -> float:
    started = time.perf_counter()
    deadline = started + timeout_seconds
    last_error = ""

    while time.perf_counter() < deadline:
        status, payload, error = request_json("GET", f"{url}/health")
        if status == 200 and isinstance(payload, dict) and payload.get("status") == "available":
            return (time.perf_counter() - started) * 1000
        last_error = error
        time.sleep(0.05)

    raise TimeoutError(f"{url}/health did not become available: {last_error}")


def request_json(
    method: str,
    url: str,
    payload: dict[str, Any] | None = None,
    timeout: float = 30.0,
) -> tuple[int | None, Any | None, str]:
    data = None
    headers = {}
    if payload is not None:
        data = json.dumps(payload).encode("utf-8")
        headers.update(CONTENT_TYPE_JSON)

    request = urllib.request.Request(url, data=data, headers=headers, method=method)
    try:
        with urllib.request.urlopen(request, timeout=timeout) as response:
            body = response.read()
            decoded = json.loads(body.decode("utf-8")) if body else None
            return response.status, decoded, ""
    except urllib.error.HTTPError as exc:
        body = exc.read().decode("utf-8", errors="replace")
        try:
            decoded = json.loads(body) if body else None
        except json.JSONDecodeError:
            decoded = body
        return exc.code, decoded, str(exc)
    except Exception as exc:
        return None, None, str(exc)


def post_search(
    url: str,
    query: str,
    limit: int = 10,
    timeout: float = 30.0,
) -> tuple[bool, int | None, float, str]:
    payload = {"q": query, "limit": limit}
    started = time.perf_counter()
    status, response, error = request_json("POST", f"{url}/search", payload, timeout=timeout)
    latency_ms = (time.perf_counter() - started) * 1000

    if status != 200:
        return False, status, latency_ms, error or f"unexpected status {status}"

    valid, validation_error = validate_search_response(response, query, limit)
    return valid, status, latency_ms, validation_error


def validate_search_response(response: Any, query: str, limit: int = 10) -> tuple[bool, str]:
    if not isinstance(response, dict):
        return False, "response is not an object"
    if response.get("query") != query:
        return False, f"query mismatch: {response.get('query')!r}"
    hits = response.get("hits")
    if not isinstance(hits, list):
        return False, "hits is not a list"
    if "estimatedTotalHits" not in response:
        return False, "missing estimatedTotalHits"

    expected = EXPECTED_HIT_IDS.get(query)
    if expected is not None and limit <= len(expected):
        hit_ids = [hit.get("id") for hit in hits[:limit] if isinstance(hit, dict)]
        if hit_ids != expected[:limit]:
            return False, f"hit ids mismatch: expected {expected[:limit]}, got {hit_ids}"

    return True, ""


def spin_process_tree_memory_bytes(root_pid: int) -> int | None:
    result = subprocess.run(
        ["ps", "-axo", "pid=,ppid=,rss="],
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if result.returncode != 0:
        return None

    children: dict[int, list[int]] = {}
    rss_by_pid: dict[int, int] = {}
    for line in result.stdout.splitlines():
        parts = line.split()
        if len(parts) < 3:
            continue
        pid, ppid, rss_kib = map(int, parts[:3])
        children.setdefault(ppid, []).append(pid)
        rss_by_pid[pid] = rss_kib

    total_kib = 0
    stack = [root_pid]
    seen: set[int] = set()
    while stack:
        pid = stack.pop()
        if pid in seen:
            continue
        seen.add(pid)
        total_kib += rss_by_pid.get(pid, 0)
        stack.extend(children.get(pid, []))

    return total_kib * 1024 if total_kib else None


def docker_container_memory_bytes() -> int | None:
    result = subprocess.run(
        ["docker", "stats", "--no-stream", "--format", "{{.MemUsage}}", OCI_CONTAINER],
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if result.returncode != 0:
        return None
    usage = result.stdout.strip().split("/", 1)[0].strip()
    return parse_memory_size(usage)


def parse_memory_size(value: str) -> int | None:
    units = {
        "b": 1,
        "kb": 1000,
        "mb": 1000**2,
        "gb": 1000**3,
        "kib": 1024,
        "mib": 1024**2,
        "gib": 1024**3,
    }
    compact = value.strip().lower().replace(" ", "")
    for unit, multiplier in sorted(units.items(), key=lambda item: len(item[0]), reverse=True):
        if compact.endswith(unit):
            number = compact[: -len(unit)]
            try:
                return int(float(number) * multiplier)
            except ValueError:
                return None
    try:
        return int(float(compact))
    except ValueError:
        return None
