# 🟡 Namada Indexer

## Architecture

An overview of the Namada Indexer's system design and how data flows through its components.

![Namada indexer architecture](architecture.png "Architecture")

## Data Flow

This diagram illustrates the high-level flow of data within the Namada Indexer.

![High Level Flow](high_level_flow.png "High Level Flow")

## Components

Namada Indexer consists of the following services:
- Chain Service
- Governance Service
- Parameters Service
- PoS Service
- Rewards Service
- Transactions Service
- Webserver Service

## SRC Cards for Each Service

An overview of each service with its **Service**, **Responsibility**, and **Collaborator** details.

![Cards](cards.png "Cards")


## Database

The Namada Indexer utilizes PostgreSQL with Diesel as the ORM. The database schema is defined in the [schema.rs](../orm/src/schema.rs) file.

![Database Schema](db.png "DB")

## Service Communication/Orchestration

Currently, service communication is achieved by checking the database state. In the future, a message broker like Redis Streams might be introduced.

## API

API documentation is available in the [swagger.yml](../swagger.yml) file. The client is generated using the [OpenAPI Generator](https://github.com/OpenAPITools/openapi-generator), and the published versions can be found on [npm](https://www.npmjs.com/package/@namada/indexer-client).

Graphs and cards are created with ❤️ using [Excalidraw](https://excalidraw.com/).
