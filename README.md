# Namada Interface Indexer

A set of microservices that crawler data from a namada node, store them in a postgres database and serve them via a REST api.

> 🔧 This is currently being worked on. Don't expect things to work! 🔧

# Namadillo integration

When using this project as a backend for [Namadillo](https://github.com/anoma/namada-interface), always checkout the latest tag, as the `main` branch could have an incompatible set of APIs.

## Architecture

The indexer is composed of a set microservices and a webserver, each one of these lives in his own crate. Each microservice is responsible of indexing some data from the chain and store them in the postgres database. Right now, there are 4 microservices:

- `chain`: goes block by block and fetches information from transactions (e.g balances)
- `pos`: fetches the validator set each new epoch
- `rewards`: fetches PoS rewards each new epoch
- `governance`: fetches new proposal and the corresponding votes
- `parameters`: fetches the chain parameters

The `webserver` is responsible to serve the data via a REST API, which are described in the `swagger.yml` file in the project root.

![Namada indexer architecture](docs/architecture.png "Architecture")

## How to run

### Prerequisites

- Create the `.env` file in the root of the project. You can use the `.env_sample` file as a reference:

```sh
cp .env_sample .env
```

- Set the `TENDERMINT_URL` with the Namada RPC url:
  - [Either create a local chain](https://docs.namada.net/operators/networks/local-network)
  - Or use a Public RPC

### With docker

- Install [just](https://github.com/casey/just)
- Run `just docker-up`

### Without docker

- Install rust/cargo
- Update the `.env` values to match your setup, for example:
  ```env
  DATABASE_URL=postgres://postgres:password@0.0.0.0:5433/namada-indexer
  TENDERMINT_URL=http://127.0.0.1:27657
  CACHE_URL=redis://redis@0.0.0.0:6379
  PORT=5001
  ```
- Use the `run.sh` script inside each package. Keep in mind that PoS package have to be run always while other service might not

## Testing via seeder

Instead of fetching data from a running network, for testing porpuses it's also possible to populate the databse with some random data.

- `cargo build`
- `cd seeder && cargo run -- --database-url postgres://postgres:password@0.0.0.0:5433/namada-indexer`

It's possible to only run the webserver and have access to the data via API.
