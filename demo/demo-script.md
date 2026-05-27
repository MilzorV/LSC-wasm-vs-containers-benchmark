# Demo Script: Movie Search on Spin/WASM vs OCI

Cel demo: pokazać, że oba runtime'y uruchamiają tę samą aplikację, zwracają te
same wyniki i mają przygotowane pełne artefakty benchmarkowe.

Zakładany czas: 3-4 minuty w prezentacji 10-12 minut.

## Przygotowanie Przed Prezentacją

W terminalu 1:

```bash
make frontend-build
cd spin-meili && spin build
spin up --listen 127.0.0.1:8080
```

W terminalu 2:

```bash
cd oci-movie-search
docker compose up --build
```

W terminalu 3 (opcjonalnie — przyciski „Run pilot” na dashboardzie):

```bash
make bench-ui
```

W terminalu 4, z katalogu głównego repo:

```bash
curl -fsS http://127.0.0.1:8080/health
curl -fsS http://127.0.0.1:8081/health
```

Oczekiwany wynik dla obu usług:

```json
{"status":"available"}
```

## Demo Na Żywo

1. Otwórz porównanie runtime'ów:

```text
http://127.0.0.1:8080/
```

Wpisz `space` i kliknij **Search both** — obie kolumny (Spin `:8080` i OCI `:8081`)
powinny pokazać te same tytuły i badge **Match**.

Stary URL `/demo` nadal działa (alias do tej samej strony).

2. Otwórz dashboard benchmarków:

```text
http://127.0.0.1:8080/benchmarks
```

Pokaż tabele i wykresy z ostatniego `make analyze`. Z uruchomionym `make bench-ui`
możesz odpalić **Run pilot** z przeglądarki (kilka minut).

3. Backup — parytet w terminalu:

```bash
benchmarks/compare_results.sh
```

Oczekiwane zakończenie:

```text
Spin and OCI movie-search results match.
```

4. Artefakty benchmarku:

```bash
ls results/raw results/processed results/plots
```

## Komentarz Do Demo

- To nie są dwie różne wyszukiwarki, tylko jeden Rust core w dwóch izolacjach.
- `/` to strona porównawcza Spin vs OCI; `/benchmarks` to wyniki pomiarów.
- Pełny benchmark: `make benchmark` lub przycisk **Run full** (wymaga `make bench-ui`).

## Plan Awaryjny

Jeżeli live runtime nie wystartuje:

1. Pokaż `results/processed/*.csv`, `dashboard.json` i wykresy z `results/plots/`.
2. Wyjaśnij, że pełny benchmark był wykonany komendą `make benchmark`.
