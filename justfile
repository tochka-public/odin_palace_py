default: check

lint-py:
    uv run ruff check . --fix

lint-rust:
    cargo clippy --all-targets --all-features

lint: lint-py lint-rust

clippy: lint

fmt-py:
    uv run ruff format .

fmt-rust:
    cargo fmt

fmt: fmt-py fmt-rust

check: fmt lint

dev:
    uv cache clean
    uv venv --allow-existing
    uv run maturin develop

test: dev
    uv run pytest tests/ -v
    cargo test

build:
    uv run maturin build --release

dev-release:
    uv cache clean
    uv venv --allow-existing
    uv run maturin develop --release

clean:
    cargo clean
    rm -rf dist/
    rm -rf target/wheels/
    uv cache clean
