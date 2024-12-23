-- Your SQL goes here
CREATE TYPE IBC_STATUS AS ENUM ('fail', 'success', 'timeout', 'unknown');

CREATE TABLE ibc_ack (
    id VARCHAR PRIMARY KEY,
    tx_hash VARCHAR NOT NULL,
    timeout BIGINT NOT NULL,
    status IBC_STATUS NOT NULL
);
