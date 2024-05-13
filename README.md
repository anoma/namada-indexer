# Namada Interface Indexer

A set of microservices that crawler data from a namada node, store them in a postgres database and serve them via a REST api.

> 🔧 This is currently being worked on. Don't expect things to work! 🔧

## Architecture

The indexer is composed of a set microservices and a webserver, each one of these lives in his own crate. Each microservice is responsible of indexing some data from the chain and store them in the postgres database. Right now, there are 4 microservices:
- `chain`: goes block by block and fetches information from transactions (e.g balances)
- `pos`: fetches the validator set each new epoch
- `rewards`: fetches PoS rewards each new epoch
- `governance`: fetches new proposal and the corresponding votes

The `webserver` is responsible to serve the data via a REST api.

![Namada indexer architecture](docs/architecture.png "Architecture")

## How to run

- Get a Namada RPC url
    - [Either create a local chain ](https://github.com/anoma/namada/blob/main/scripts/gen_localnet.py)
    - Or use a Public RPC 
- Change `CHAIN_ID` and `CHECKSUMS_FILE` env variable and file
- Install [just](https://github.com/casey/just)
- Run `just docker-run`