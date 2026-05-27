# Demo Script: Movie Search on Spin/WASM vs OCI

Cel demo: pokazać, że oba runtime'y uruchamiają tę samą aplikację, zwracają te
same wyniki i mają przygotowane pełne artefakty benchmarkowe.

Zakładany czas: 3-4 minuty w prezentacji 10-12 minut.

## Przygotowanie Przed Prezentacją

W terminalu 1:

```bash
cd spin-meili
spin build
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

0. Otwórz dashboard w przeglądarce:

```text
http://127.0.0.1:8080/
```

Wpisz `space`, kliknij **Search both**. Oczekiwane: obie kolumny pokazują
`62, 957, 1542...` i badge **Match**.

Alternatywnie:

```bash
make demo-open
```

1. Pokaż wersję silnika:

```bash
curl -fsS http://127.0.0.1:8080/version
curl -fsS http://127.0.0.1:8081/version
```

Oczekiwane: oba runtime'y zwracają `engine=movie-search-core`,
`datasetDocuments=44471`.

2. Pokaż statystyki:

```bash
curl -fsS http://127.0.0.1:8080/stats
curl -fsS http://127.0.0.1:8081/stats
```

Oczekiwane:

```json
{"documentCount":44471}
```

3. Pokaż realne wyszukiwanie:

```bash
curl -fsS \
  -X POST http://127.0.0.1:8080/search \
  -H 'content-type: application/json' \
  --data '{"q":"space","limit":3}'

curl -fsS \
  -X POST http://127.0.0.1:8081/search \
  -H 'content-type: application/json' \
  --data '{"q":"space","limit":3}'
```

Oczekiwane pierwsze `hit.id`: `62`, `957`, `1542`.

4. Uruchom parytet wyników:

```bash
benchmarks/compare_results.sh
```

Oczekiwane zakończenie:

```text
Spin and OCI movie-search results match.
```

5. Pokaż artefakty benchmarku:

```bash
ls results/raw results/processed results/plots
sed -n '1,80p' results/processed/cold_start_summary.csv
sed -n '1,120p' results/processed/load_summary.csv
sed -n '1,80p' results/processed/memory_summary.csv
```

Pokaż wykresy:

- `results/plots/cold_start_p95.png`
- `results/plots/load_throughput.png`
- `results/plots/load_latency_p95.png`
- `results/plots/memory_peak.png`

## Komentarz Do Demo

Krótka narracja:

- To nie są dwie różne wyszukiwarki, tylko jeden Rust core w dwóch izolacjach.
- Dashboard w przeglądarce pokazuje parytet wizualnie; curl i `compare_results.sh` to backup.
- Najpierw udowadniamy zgodność funkcjonalną, dopiero potem rozmawiamy o wydajności.
- Full benchmark nie jest odpalany na żywo, bo trwa około kilku-kilkunastu minut.
- Wyniki są już zapisane w CSV i na wykresach, więc demo jest reprodukowalne.

## Plan Awaryjny

Jeżeli live runtime nie wystartuje:

1. Pokaż `results/processed/*.csv`.
2. Pokaż wykresy z `results/plots/`.
3. Pokaż raw CSV z `results/raw/`.
4. Wyjaśnij, że pełny benchmark był wykonany komendą:

```bash
benchmarks/run_all.sh
```

I że raport korzysta z artefaktów z pełnego przebiegu, a nie z wyników pilota.
