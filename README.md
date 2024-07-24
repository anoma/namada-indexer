# Namada Interface Indexer

A set of microservices that crawler data from a namada node, store them in a postgres database and serve them via a REST api.

> 🔧 This is currently being worked on. Don't expect things to work! 🔧

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

Create the `.env` file. You can use the `.env_sample` as a reference

```
cp .env_sample .env
```

### With docker

- Get a Namada RPC url
  - [Either create a local chain ](https://github.com/anoma/namada/blob/main/scripts/gen_localnet.py)
  - Or use a Public RPC
- Install [just](https://github.com/casey/just)
- Run `just docker-run`

### Without docker

- Install rust/cargo
- Get a Namada RPC url
  - [Either create a local chain ](https://github.com/anoma/namada/blob/main/scripts/gen_localnet.py)
  - Or use a Public RPC
- Create a `.env` file in the root of the project with the following content:
  ```env
  DATABASE_URL=postgres://postgres:password@0.0.0.0:5435/namada-indexer
  TENDERMINT_URL=http://127.0.0.1:27657
  ```
  (Change the values to match your setup)
- For `webserver` only, you may also add the following to the `.env` file:
  ```env
  CACHE_URL=redis://redis@0.0.0.0:6379
  PORT=5000
  ```
  (Change the values to match your setup)
- Use the `run.sh` script inside each package. Keep in mind that PoS package have to be run always while other service might not

## Testing via seeder

Instead of fetching data from a running network, for testing porpuses it's also possible to populate the databse with some random data.

- `cargo build`
- `cd seeder && cargo run -- --database-url postgres://postgres:password@0.0.0.0:5435/namada-indexer`

It's possible to only run the webserver and have access to the data via API.
