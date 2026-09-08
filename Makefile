.PHONY: install dev build-deps test lint format format-check check build clean run run-devmode help venv lock

VENV_DIR ?= .venv
UV ?= uv

help:
	@echo "ShellGame - Makefile commands:"
	@echo "  make install      - Install package (base deps) via uv"
	@echo "  make dev          - Install package with dev deps via uv"
	@echo "  make lock         - Generate/update uv.lock"
	@echo "  make test         - Run tests with coverage (uv run)"
	@echo "  make lint         - Run Ruff and mypy (uv run)"
	@echo "  make format       - Format code with Ruff (uv run)"
	@echo "  make format-check - Check formatting without changing files"
	@echo "  make check        - Run lint, format check, and tests"
	@echo "  make build        - Build standalone binary with PyInstaller (installs build-only deps)"
	@echo "  make clean        - Clean build artifacts and cache"
	@echo "  make run          - Run ShellGame (normal mode; auto-launches wrapped subshell if needed)"
	@echo "  make run-devmode   - Run ShellGame with --devmode (auto-launches wrapped subshell if needed)"

$(VENV_DIR):
	$(UV) venv $(VENV_DIR) --quiet

venv: | $(VENV_DIR)

install: venv
	$(UV) sync

dev: venv
	$(UV) sync --extra dev

build-deps: venv
	$(UV) sync --extra build

lock:
	$(UV) lock

test: dev
	$(UV) run pytest tests/ -v --cov=shellgame --cov-report=term-missing

lint: dev
	$(UV) run ruff check src/shellgame/ tests/
	$(UV) run mypy src/shellgame/

format: dev
	$(UV) run ruff format src/shellgame/ tests/

format-check: dev
	$(UV) run ruff format --check src/shellgame/ tests/

check: lint format-check test

build: build-deps
	$(UV) run pyinstaller shellgame.spec

clean:
	rm -rf build/ dist/ *.egg-info .pytest_cache .mypy_cache .coverage
	find . -type d -name __pycache__ -exec rm -rf {} + 2>/dev/null || true
	find . -type f -name "*.pyc" -delete

run: dev
	# Normal run (no devmode). This will still auto-launch the wrapped subshell if needed.
	$(UV) run shellgame

run-devmode: dev
	# Dev mode. This will still auto-launch the wrapped subshell if needed.
	$(UV) run shellgame --devmode
