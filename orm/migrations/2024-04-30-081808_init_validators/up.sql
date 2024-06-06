-- Your SQL goes here

CREATE TYPE VALIDATOR_STATE AS ENUM ('consensus', 'inactive', 'jailed', 'below_capacity', 'below_threshold', 'unknown');

CREATE TABLE validators (
    id SERIAL PRIMARY KEY,
    namada_address VARCHAR NOT NULL,
    voting_power INT NOT NULL,
    max_commission VARCHAR NOT NULL,
    commission VARCHAR NOT NULL,
    name VARCHAR,
    email VARCHAR,
    website VARCHAR,
    description VARCHAR,
    discord_handle VARCHAR,
    avatar VARCHAR,
    state VALIDATOR_STATE NOT NULL
);

ALTER TABLE validators
ADD UNIQUE (namada_address);

CREATE INDEX index_validators_namada_address ON validators USING HASH (namada_address);
