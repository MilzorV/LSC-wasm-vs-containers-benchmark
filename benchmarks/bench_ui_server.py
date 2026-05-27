#!/usr/bin/env python3
"""Local helper for the benchmark dashboard (127.0.0.1:8092)."""

from __future__ import annotations

import json
import subprocess
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RUN_ALL = ROOT / "benchmarks" / "run_all.sh"
DASHBOARD = ROOT / "results" / "processed" / "dashboard.json"
LOG_PATH = ROOT / "results" / "raw" / "bench_ui_run.log"

HOST = "127.0.0.1"
PORT = 8092

_state_lock = threading.Lock()
_state: dict[str, object] = {
    "running": False,
    "exitCode": None,
    "logTail": "",
    "lastRunId": None,
    "profile": None,
}


def _set_state(**kwargs: object) -> None:
    with _state_lock:
        _state.update(kwargs)


def _read_log_tail(max_lines: int = 40) -> str:
    if not LOG_PATH.exists():
        return ""
    lines = LOG_PATH.read_text(encoding="utf-8", errors="replace").splitlines()
    return "\n".join(lines[-max_lines:])


def _run_benchmark(profile: str) -> None:
    LOG_PATH.parent.mkdir(parents=True, exist_ok=True)
    LOG_PATH.write_text(f"== Starting {profile} benchmark ==\n", encoding="utf-8")
    cmd = [str(RUN_ALL)]
    if profile == "pilot":
        cmd.append("--pilot")
    _set_state(running=True, exitCode=None, profile=profile, logTail=_read_log_tail())
    try:
        with LOG_PATH.open("a", encoding="utf-8") as logfh:
            proc = subprocess.run(
                cmd,
                cwd=str(ROOT),
                stdout=logfh,
                stderr=subprocess.STDOUT,
                check=False,
            )
        exit_code = proc.returncode
    except Exception as exc:
        with LOG_PATH.open("a", encoding="utf-8") as logfh:
            logfh.write(f"\nERROR: {exc}\n")
        exit_code = 1

    run_id = time.strftime("%Y%m%d_%H%M%S")
    _set_state(
        running=False,
        exitCode=exit_code,
        lastRunId=run_id,
        logTail=_read_log_tail(),
    )


class BenchHandler(BaseHTTPRequestHandler):
    def log_message(self, format: str, *args: object) -> None:
        return

    def _cors(self) -> None:
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "Content-Type")

    def _json(self, status: int, payload: object) -> None:
        body = json.dumps(payload).encode("utf-8")
        self.send_response(status)
        self._cors()
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_OPTIONS(self) -> None:
        self.send_response(204)
        self._cors()
        self.end_headers()

    def do_GET(self) -> None:
        if self.path == "/status":
            with _state_lock:
                payload = dict(_state)
            payload["logTail"] = _read_log_tail()
            self._json(200, payload)
            return
        if self.path == "/summary":
            if not DASHBOARD.exists():
                self._json(404, {"error": "dashboard.json not found — run make analyze"})
                return
            data = json.loads(DASHBOARD.read_text(encoding="utf-8"))
            self._json(200, data)
            return
        self._json(404, {"error": "not found"})

    def do_POST(self) -> None:
        if self.path != "/run":
            self._json(404, {"error": "not found"})
            return
        length = int(self.headers.get("Content-Length", "0"))
        raw = self.rfile.read(length) if length else b"{}"
        try:
            body = json.loads(raw.decode("utf-8"))
        except json.JSONDecodeError:
            self._json(400, {"error": "invalid JSON"})
            return
        profile = body.get("profile", "pilot")
        if profile not in ("pilot", "full"):
            self._json(400, {"error": "profile must be pilot or full"})
            return
        with _state_lock:
            if _state.get("running"):
                self._json(409, {"error": "benchmark already running"})
                return
        thread = threading.Thread(target=_run_benchmark, args=(profile,), daemon=True)
        thread.start()
        self._json(202, {"started": True, "profile": profile})


def main() -> int:
    print(f"Benchmark UI helper on http://{HOST}:{PORT}")
    print("Endpoints: GET /status, GET /summary, POST /run")
    server = ThreadingHTTPServer((HOST, PORT), BenchHandler)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\nShutting down.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
