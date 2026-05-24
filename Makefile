.PHONY: test build smoke benchmark benchmark-pilot analyze report demo clean

ROOT := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))

test:
	cargo test --manifest-path $(ROOT)/spin-meili/Cargo.toml

build:
	cd $(ROOT)/spin-meili && spin build
	cd $(ROOT)/oci-movie-search && docker compose build

smoke:
	$(ROOT)/benchmarks/smoke_spin.sh
	$(ROOT)/benchmarks/smoke_oci.sh
	$(ROOT)/benchmarks/compare_results.sh

benchmark:
	$(ROOT)/benchmarks/run_all.sh

benchmark-pilot:
	$(ROOT)/benchmarks/run_all.sh --pilot

analyze:
	python3 $(ROOT)/benchmarks/analyze_results.py

report:
	latexmk -pdf -outdir=$(ROOT)/report -interaction=nonstopmode -halt-on-error $(ROOT)/report/final-report.tex

demo:
	@echo "See $(ROOT)/demo/demo-script.md"

clean:
	cd $(ROOT)/oci-movie-search && docker compose down --remove-orphans || true
