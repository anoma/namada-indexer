FROM rust:1.77-bookworm AS builder

RUN apt-get update && apt-get install -y protobuf-compiler build-essential clang-tools-14

COPY . /app

WORKDIR /app

RUN DEBIAN_FRONTEND=noninteractive apt-get update

RUN cargo build --release --package seeder

FROM debian:bookworm-slim

RUN DEBIAN_FRONTEND=noninteractive apt-get update && apt-get install -y ca-certificates libpq5

COPY --from=builder /app/target/release/seeder /app/seeder
WORKDIR /app

CMD ["./seeder"]