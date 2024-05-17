build:
    cargo build --all

check:
    cargo check --all

fmt:
    cargo +nightly fmt --all

clippy:
    cargo clippy

clippy-fix:
    cargo clippy --all --fix --allow-dirty --allow-staged

docker-up:
    docker compose up

clean:
    cargo clean