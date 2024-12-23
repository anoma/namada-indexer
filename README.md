# 🟡 Namada Interface Indexer

```
🔧 This project is a work in progress. Functionality is not guaranteed at this stage! 🔧
```

##  About 

The **Namada Indexer** is a collection of microservices designed to crawl data from a Namada Node, store it in a PostgreSQL database, and provide access via a REST API.

## Namadillo Integration

When using this project as a backend for [Namadillo](https://github.com/anoma/namada-interface), always ensure you check out the latest tag. The `main` branch may contain an incompatible set of APIs.

## Contribution

We welcome contributions to this project! If you'd like to help, feel free to pick an issue labeled as `bug` or `good-first-issue`. If you want to propose a new feature, please open an issue first so we can discuss it.

- For **new features**, target the `main` branch.
- For **bug fixes**, check out the latest maintenance branch (e.g., `1.0.0-maint`) and target your changes there.

## Architecture

The Namada Indexer is composed of a set of microservices, with each component residing in its own crate. Each microservice is responsible for indexing specific data from the blockchain and storing it in the PostgreSQL database.

### Microservices & Containers
- `namada/chain`: Processes blocks sequentially and extracts information from transactions (e.g., balances).

- `namada/pos`: Retrieves the validator set at the start of each new epoch.

- `namada/rewards`: Fetches Proof-of-Stake rewards for each new epoch.

- `namada/governance`: Tracks new proposals and their corresponding votes.

- `namada/parameters`: Retrieves the chain parameters.

- `namada/transactions`: Processes transactions starting from block height 0 (or the last successfully processed block height).

- `namada/webserver-indexer`: The `webserver` serves indexed data via a REST API, enabling external applications and users to access blockchain data in a structured and accessible way. It listens on port `5001`. The API endpoints are described in the `swagger.yml` file located in the project root. A hosted HTML version of the API documentation is available at [Namada Interface Indexer REST API](https://anoma.github.io/namada-indexer).

- `postgres:16-alpine`: This container runs a PostgreSQL instance, serving as the primary database for storing indexed data fetched by the microservices. It listens on port `5433` and provides a reliable and scalable storage backend for the project.

- `docker.dragonflydb.io/dragonflydb/dragonfly`: This container runs a DragonflyDB instance, an advanced in-memory key-value store that acts as a caching layer. It listens on port `6379` and stores frequently accessed or temporary data, improving system performance by reducing the need for repeated database queries.

<p align="center">
  <img src="docs/architecture.png" alt="Namada Indexer Architecture" title="Architecture" width="500">
</p>


# 🚀 Getting Started

Follow these instructions to set up the project locally. The steps below will guide you through the process of getting a local copy up and running.

It is strongly recommended to change the default username and password for your PostgreSQL database for security purposes. Update these credentials in both the `.env` file and the `docker-compose.yml` file to reflect the changes.

## Installation with Docker 🐳 

### Prerequisites

Before starting, ensure you have the necessary tools and dependencies installed. Below are the steps to set up the required environment.

- **Packages**: Install prerequisite packages from the APT repository.

```sh
apt-get install -y curl apt-transport-https ca-certificates software-properties-common git nano just build-essential
```

- **Docker**: Follow the official instructions provided by Docker to install it: [Install Docker Engine](https://docs.docker.com/engine/install/).

- **Just**: Refer to the official documentation to install `just`: [Just Installation Guide](https://github.com/casey/just).

### Usage
Ensure you have the latest repository cloned to maintain compatibility with other Namada interfaces. Use the following commands to clone the repository and navigate into its directory.

```sh
git clone https://github.com/anoma/namada-indexer.git
cd namada-indexer
```

Create the `.env` file in the root of the project. You can use the `.env.sample` file as a reference. 

```sh
cp .env.sample .env
```
- The `TENDERMINT_URL` variable must point to a Namada RPC URL, which can be either public or local. For a public RPC URL, refer to the [Namada Ecosystem Repository](https://github.com/Luminara-Hub/namada-ecosystem/tree/main/user-and-dev-tools/mainnet). If running the Namada Ledger locally, use the preconfigured `http://host.docker.internal:26657`.

Build the required Docker containers for the project.
```sh
docker compose build
```

Launch the Namada Indexer using the `just` command, which orchestrates the Docker containers.
```sh
just docker-up
```

## Installation without Docker

If you prefer not to use Docker, you can follow the instructions below to set up and run the services manually.

- Install **Rust** and **Cargo** on your system. Refer to the [official Rust installation guide](https://www.rust-lang.org/tools/install).

- Update the `.env` file with values that match your setup.

- Use the `run.sh` script located inside each package to start the services.  
   - The **PoS** package must always be running.  
   - Other services can be run as needed based on your requirements.


## Populating the Database for Testing

Instead of fetching data from a running network, you can populate the database with random data for testing purposes. Build the project using the following command.

```sh
cargo build
# Run the seeder script to populate the database
cd seeder && cargo run -- --database-url postgres://postgres:password@0.0.0.0:5433/namada-indexer
```

After populating the database, you can run the webserver to access the data via the API. To query your PostgreSQL database, ensure the PostgreSQL client is installed.

```sh
apt-get install -y postgresql-client
```