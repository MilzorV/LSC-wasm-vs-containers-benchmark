.PHONY: test build frontend-build smoke benchmark benchmark-pilot analyze report presentation demo demo-open clean

ROOT := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))

frontend-build:
	cd $(ROOT)/frontend && npm ci && npm run build

test: frontend-build
	cargo test --manifest-path $(ROOT)/spin-meili/Cargo.toml

build: frontend-build
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

presentation:
	$(MAKE) -C $(ROOT)/presentation

demo:
	@echo "App: http://127.0.0.1:8080/  Compare demo: http://127.0.0.1:8080/demo"

demo-open:
	open http://127.0.0.1:8080/

clean:
	cd $(ROOT)/oci-movie-search && docker compose down --remove-orphans || true
