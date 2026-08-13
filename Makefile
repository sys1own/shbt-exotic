VENV := .venv
PYTHON := $(VENV)/bin/python
MATURIN := $(VENV)/bin/maturin
PYTEST := $(VENV)/bin/pytest

.PHONY: all build test python-test rust-test clean audit

all: build test audit

build: $(VENV)/bin/activate
	$(MATURIN) develop

$(VENV)/bin/activate:
	python3 -m venv $(VENV)
	$(VENV)/bin/pip install --quiet maturin pytest numpy matplotlib

test: build python-test rust-test

python-test: build
	$(PYTEST) tests/ -q

rust-test:
	cargo test -q

audit: build
	$(PYTHON) -m shbt_exotic.cli --audit

clean:
	cargo clean
	rm -rf $(VENV) build dist *.egg-info .pytest_cache
	find . -type d -name __pycache__ -exec rm -rf {} +
