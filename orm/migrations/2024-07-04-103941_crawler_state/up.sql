-- Your SQL goes here

CREATE TYPE CRAWLER_NAME AS ENUM ('chain', 'governance', 'parameters', 'pos', 'rewards', 'transactions');

CREATE TABLE crawler_state (
    name CRAWLER_NAME PRIMARY KEY NOT NULL,
    last_processed_block INT,
    first_block_in_epoch INT,
    last_processed_epoch INT,
    timestamp TIMESTAMP NOT NULL
);
