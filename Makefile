VENV := .venv
PYTHON := $(VENV)/bin/python
MATURIN := $(VENV)/bin/maturin
PYTEST := $(VENV)/bin/pytest

.PHONY: all rust test cargo-test figures macros paper audit clean

all: rust test paper audit

audit:
	$(PYTHON) -m shbt_exotic.cli --audit

$(VENV)/bin/activate:
	python3 -m venv $(VENV)
	$(VENV)/bin/pip install --quiet maturin pytest numpy matplotlib

rust: $(VENV)/bin/activate
	$(MATURIN) develop

figures: rust
	$(PYTHON) -m shbt_exotic.plots

macros: rust
	$(PYTHON) -m shbt_exotic.latex

paper: figures macros
	pdflatex -interaction=nonstopmode -jobname=exotic main.tex
	pdflatex -interaction=nonstopmode -jobname=exotic main.tex

test: rust
	$(PYTEST) tests/ -q

cargo-test:
	cargo test -q

clean:
	rm -rf $(VENV) target python/shbt_exotic/*.so python/shbt_exotic/__pycache__ tests/__pycache__ exotic_results.tex figures/*.pdf exotic.aux exotic.log exotic.out exotic.toc exotic.synctex.gz exoticNotes.bib *.aux *.log *.out *.toc *.synctex.gz *Notes.bib
	find . -type d -name __pycache__ -exec rm -rf {} + 2>/dev/null || true
