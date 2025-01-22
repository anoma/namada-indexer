devs:
    rustup toolchain install 1.79.0 --no-self-update --component clippy,rustfmt
    rustup toolchain install nightly-2024-06-14 --no-self-update --component clippy,rustfmt

build:
    cargo build --all

check:
    cargo check --all

fmt:
    cargo +nightly-2024-06-14 fmt --all

fmt-check:
    cargo +nightly-2024-06-14 fmt --all --check

test:
    cargo test

clippy:
    cargo clippy

clippy-fix:
    cargo clippy --all --fix --allow-dirty --allow-staged

docker-up:
    docker compose up

docker-up-d:
    docker compose up -d

clean:
    cargo clean

run-chain:
    (cd chain && ./run.sh)

test-chain:
    (cd chain && ./run-test.sh)

run-governance:
    (cd governance && ./run.sh)

run-parameters:
    (cd parameters && ./run.sh)

run-pos:
    (cd pos && ./run.sh)

run-rewards:
    (cd rewards && ./run.sh)

run-seeder:
    (cd seeder && ./run.sh)

run-transactions:
    (cd transactions && ./run.sh)

run-webserver:
    (cd webserver && ./run.sh)
