# Submission Package

Primary artifacts:

- `report/final-report.pdf` - final written report.
- `presentation/main.pdf` - current LaTeX Beamer presentation.
- `presentation/main.tex` - maintained presentation source.
- `results/processed/` - final summary CSV files and dashboard JSON.
- `results/plots/` - final generated benchmark plots.
- `results/raw/cold_start_20260530-131300.csv` - final cold-start run.
- `results/raw/load_20260530-122840.csv` and `.json` - final full load run.
- `results/raw/memory_20260530-130944.csv` - final memory run.

Build and verification commands used before packaging:

```bash
npm run build
cargo test --manifest-path spin-meili/Cargo.toml
spin build
python3 -m py_compile benchmarks/*.py
make report
make -C presentation
benchmarks/compare_results.sh
```

The PowerPoint files in `presentation/movie-search-spin-vs-oci.*` are retained as legacy
artifacts. The authoritative deck for submission is the LaTeX Beamer deck in
`presentation/main.pdf`.
