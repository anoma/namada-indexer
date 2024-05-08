# Namada Interface Indexer

A set of microservices that crawler data from a namada node, store them in a postgres database and serve them via a REST api.

## Architecture

The indexer is composed of a set microservices and a webserver, each one of these lives in his own crate. Each microservice is responsible of indexing some data from the chain and store them in the postgres database. Right now, there are 4 microservices:

- `chain`: goes block by block and fetches information from transactions (e.g balances)
- `pos`: fetches the validator set each new epoch
- `rewards`: fetches PoS rewards each new epoch
- `governance`: fetches new proposal and the corresponding votes

The `webserver` is responsible to serve the data via a REST api.

![Namada indexer architecture](docs/architecture.png "Architecture")

## How to run locally

### DBs setup

```sh
docker compose -f docker-compose-db.yml up
```

### Crawlers

#### Run migrations manually(for now)

Make sure you have DATABASE_URL set in your environment variables. Take a look at .env_sample for the reference.

```sh
cd orm
diesel migrations run
```

#### Re-run migrations manually(for now)

```sh
cd orm
diesel migrations redo --all
```

#### Run crawler(s)

Assuming you have a namada node running locally on port 27657

```sh
cd chain # or pos
./run.sh

```

### Run webserver

To run the webserver you do not need to have crawlers running. To fill the database with fake data:

```sh
cd orm
make run-test-migrations
```

Then you can run the webserver:

```sh
cd webserver
./run.sh
```
