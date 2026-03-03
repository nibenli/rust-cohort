VENV_BIN = .venv/bin
PYTHON = $(shell [ -f $(VENV_BIN)/python ] && echo $(VENV_BIN)/python || echo python3)
PYTEST = $(shell [ -f $(VENV_BIN)/pytest ] && echo $(VENV_BIN)/pytest || echo pytest)
CARGO  = cargo
MATURIN = $(shell [ -f $(VENV_BIN)/maturin ] && echo $(VENV_BIN)/maturin || echo maturin)
PKG_NAME = rust_json_parser

.PHONY: all develop release test test-rust test-python build benchmark doc clean help

all: develop

## Install/Update the extension in the local environment
develop:
	$(MATURIN) develop

## Install/Update the extension in optimized mode (Essential for Benchmarking)
release:
	$(MATURIN) develop --release

## Run all tests (Rust and Python)
test: test-rust test-python

## Run Rust unit tests (disabling python features for pure Rust testing)
test-rust:
	$(CARGO) test --no-default-features

## Run Python integration tests
test-python:
	$(PYTEST) -v tests/test_python_integration.py

## Build the Rust library without python feature
build:
	$(CARGO) build --no-default-features --lib

## --- CLI Examples ---

run-file:
	@echo '{"status": "ok", "code": 200}' > data.json
	$(PYTHON) -m rust_json_parser data.json

run-string:
	$(PYTHON) -m rust_json_parser '{"hello":"world"}'

run-pipe:
	echo '{"test": 123}' | $(PYTHON) -m rust_json_parser

## Run the performance comparison suite
benchmark:
	$(PYTHON) -m $(PKG_NAME) --benchmark

## Generate Rust documentation and open in browser
doc:
	$(CARGO) doc --open --no-deps

## --- Utility ---

clean:
	$(CARGO) clean
	rm -f data.json
	find . -name "__pycache__" -exec rm -rf {} +
	find . -name "*.so" -delete

help:
	@echo "Rust-JSON-Parser Management Commands:"
	@echo "  make develop       Compile Rust and install extension"
	@echo "  make release       Compile Rust (Optimized/Release) and install"
	@echo "  make benchmark     Run performance suite (Python json vs Rust vs simplejson)"
	@echo "  make doc           Generate and open project documentation"
	@echo "  make test          Run both Rust and Python tests"
	@echo "  make test-python   Run Python tests only (triggers develop)"
	@echo "  make test-rust     Run Rust tests only"
	@echo "  make run-file      Parse data.json using CLI"
	@echo "  make run-string    Parse inline string using CLI"
	@echo "  make run-pipe      Parse from stdin"
	@echo "  make clean         Remove build artifacts"