# Demo Script: Movie Search on Spin/WASM vs OCI

Cel demo: pokazać, że oba runtime'y uruchamiają tę samą aplikację, zwracają te
same wyniki i mają przygotowane pełne artefakty benchmarkowe.

Zakładany czas: 3-4 minuty w prezentacji 10-12 minut.

## Przygotowanie Przed Prezentacją

W terminalu 1:

```bash
cd frontend && npm ci && npm run build
cd ../spin-meili && spin build
spin up --listen 127.0.0.1:8080
```

W terminalu 2:

```bash
cd oci-movie-search
docker compose up --build
```

W terminalu 3, z katalogu głównego repo:

```bash
curl -fsS http://127.0.0.1:8080/health
curl -fsS http://127.0.0.1:8081/health
```

Oczekiwany wynik dla obu usług:

```json
{"status":"available"}
```

## Demo Na Żywo

0. Otwórz aplikację w przeglądarce:

```text
http://127.0.0.1:8080/
```

Pokaż **Search** (`space`) i **Browse** — normalna aplikacja do wyszukiwania filmów.
Kliknij kartę filmu, aby pokazać szczegóły.

Następnie otwórz demo porównawcze:

```text
http://127.0.0.1:8080/demo
```

Wpisz `space` — obie kolumny (Spin i OCI) powinny pokazać te same tytuły i badge **Match**.

1. Uruchom parytet wyników (backup):

```bash
benchmarks/compare_results.sh
```

Oczekiwane zakończenie:

```text
Spin and OCI movie-search results match.
```

2. Pokaż artefakty benchmarku:

```bash
ls results/raw results/processed results/plots
```

Wykresy: `results/plots/cold_start_p95.png`, `load_throughput.png`, `load_latency_p95.png`, `memory_peak.png`

## Komentarz Do Demo

- To nie są dwie różne wyszukiwarki, tylko jeden Rust core w dwóch izolacjach.
- `/` to normalna aplikacja; `/demo` to strona porównawcza na prezentację.
- Full benchmark nie jest odpalany na żywo; wyniki są w CSV i na wykresach.

## Plan Awaryjny

Jeżeli live runtime nie wystartuje:

1. Pokaż `results/processed/*.csv` i wykresy z `results/plots/`.
2. Wyjaśnij, że pełny benchmark był wykonany komendą `benchmarks/run_all.sh`.
