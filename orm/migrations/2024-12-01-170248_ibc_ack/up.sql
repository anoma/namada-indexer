-- Your SQL goes here
CREATE TABLE ibc_ack (
    id VARCHAR PRIMARY KEY,
    tx_hash VARCHAR NOT NULL,
    timeout INT NOT NULL
);
