# OCI baseline (nginx)

Serves the same **paths** as the Spin Bartholomew app for benchmarking: `/`, `/blog/example`, `/static/style.css`. Content is static HTML + CSS (see `docs/METHODOLOGY.md` in the repo root for the equivalence statement).

```bash
docker compose up --build -d
# open http://127.0.0.1:8081/
docker compose down
```
