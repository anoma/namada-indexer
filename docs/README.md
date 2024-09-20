# Namada Indexer

## Goal

Main purpose of the Namada Indexer is to retrieve and store information required for the Namada Interface(Namadillo) to operate.
Because of this, the Namada Indexer does not store historical data.
Exception to this rule is the `Transactions Service` which is not used by the Namadillo and it indexes all transactions starting from block 1.

### Overview:

![Namada indexer architecture](architecture.png "Architecture")

### Flow:

![High Level Flow](high_level_flow.png "High Level Flow")

## Components

Namada Indexer consists of the following services:

- Chain Service
- PoS Service
- Parameters Service
- Governance Service
- Rewards Service
- Transactions Service
- Webserver

### SRC(Service, Responsibility, Collaborator) cards for each service:

![Cards](cards.png "Cards")

## Database

We use PostgreSQL and diesel as ORM. You can find the schema in the [schema.rs](../orm/src/schema.rs) file.

![DB](db.png "DB")

## Service communication/orchestration

At this point only by checking the state of the database. In the future we might use a simple message broker like Redis streams.

## API

For the API documentation, please refer to the [swagger.yml](../swagger.yml) file.
We generate the client using [OpenApi generator](https://github.com/OpenAPITools/openapi-generator).
You can find the published versions [here](https://www.npmjs.com/package/@anomaorg/namada-indexer-client).

Graphs/cards thanks to [excalidraw <3](docs_indexer_2024_09_20.excalidraw).
