# Raport Końcowy: Movie Search na Spin/WASM vs OCI

Właściwym raportem końcowym jest wersja LaTeX:

- źródło: `report/final-report.tex`;
- PDF do oddania: `report/final-report.pdf`.

Ten plik Markdown pełni rolę krótkiego indeksu i streszczenia pakietu.

## Zakres

Projekt porównuje ten sam mikroserwis HTTP `movie-search` w dwóch modelach
izolacji:

- Spin/wasmtime jako komponent WASI HTTP;
- natywny serwer Rust uruchamiany w kontenerze OCI.

Porzucono próbę bezpośredniego porównania z Meilisearch, ponieważ oficjalny
Meilisearch opiera się na LMDB, memory-mapped storage i natywnych założeniach
runtime, których nie dało się przenieść 1:1 do Spin/WASI w ramach projektu.

## Finalne Artefakty

- Raport: `report/final-report.pdf`.
- Prezentacja edytowalna: `presentation/movie-search-spin-vs-oci.pptx`.
- Prezentacja PDF: `presentation/movie-search-spin-vs-oci.pdf`.
- Demo: `demo/demo-script.md`.
- Wyniki surowe: `results/raw/`.
- Wyniki przetworzone: `results/processed/`.
- Wykresy: `results/plots/`.

## Najważniejsze Wyniki

- Parytet funkcjonalny: Spin i OCI zwracają te same `hit.id` dla zapytań
  `space`, `toy story`, `dark knight`, `romance` i pustego zapytania.
- Cold start `/health` p95: OCI `28.420 ms`, Spin `144.220 ms`.
- Cold start + pierwszy `/search` p95: OCI `376.983 ms`, Spin `217.504 ms`.
- Empty query, concurrency 10: OCI `3314.9 req/s`, Spin `132.8 req/s`.
- Query `space`, concurrency 10: OCI `54.0 req/s`, Spin `77.2 req/s`.
- Memory load max: OCI `31.9 MiB` przez `docker stats`, Spin `493.5 MiB`
  jako host process RSS.

## Reprodukcja

Pełny benchmark:

```bash
benchmarks/run_all.sh
```

Kompilacja raportu:

```bash
latexmk -pdf -outdir=report -interaction=nonstopmode -halt-on-error report/final-report.tex
```

Demo prezentacyjne opisuje `demo/demo-script.md`. Demo nie uruchamia pełnego
benchmarku na żywo; pokazuje parytet funkcjonalny i finalne artefakty wyników.
