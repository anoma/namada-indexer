build:
    cargo build --all

check:
    cargo check --all

fmt:
    cargo +nightly fmt --all

test:
    cargo test

clippy:
    cargo clippy

clippy-fix:
    cargo clippy --all --fix --allow-dirty --allow-staged

docker-up:
    docker compose up

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
