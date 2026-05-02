# Environment template

Record **measured** values for each benchmark campaign in this repository. Copy this section into the final report or attach exported `sysctl`/CPU dumps as needed.

## Host

| Field | Value (fill in) |
|--------|-----------------|
| OS | e.g. macOS 14.x / Ubuntu 24.04 |
| CPU model | `sysctl -n machdep.cpu.brand_string` or `/proc/cpuinfo` |
| RAM | |
| Disk | SSD / NVMe / … |
| Power | AC / battery |

## Tool versions

| Tool | Command | Version |
|------|---------|---------|
| Spin | `spin --version` | |
| Docker | `docker version` | Client + Engine |
| Python | `python3 --version` | 3.11+ recommended |
| hyperfine | `hyperfine --version` | optional, for CLI timing checks |
| wrk | `wrk -v` | optional if `USE_WRK=1` |
| git | `git rev-parse HEAD` | pin results to commit |

## Python (analysis)

From repo root:

```bash
python3 -m venv .venv
source .venv/bin/activate   # Windows: .venv\Scripts\activate
pip install -e ".[analysis]"
```

## Pre-run checklist

- [ ] Other heavy apps closed (browsers optional tabs, video, backups).
- [ ] VPN off unless required (can skew latency).
- [ ] Docker Desktop has enough CPUs/RAM allocated.
- [ ] Same Wi-Fi vs Ethernet as prior runs (prefer Ethernet or localhost-only — all URLs are `127.0.0.1`).
