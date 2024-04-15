# use the default dart image as the build image
FROM rust:1.70 AS builder

ARG PRODUCTION_BUILD="false"

RUN apt-get update && apt-get install -y protobuf-compiler

# copy the current folder into the build folder
COPY . /app

# set the work directory
WORKDIR /app

RUN DEBIAN_FRONTEND=noninteractive apt-get update

# build app
RUN cargo build --release --package webserver

# use a slim image
FROM debian:bullseye-slim

RUN DEBIAN_FRONTEND=noninteractive apt-get update && apt-get install -y ca-certificates libpq5

# copy the runtime files
COPY --from=builder /app/target/release/webserver /app/webserver
WORKDIR /app

# start the webserver
CMD ["./webserver"]