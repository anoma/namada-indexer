-- Your SQL goes here

CREATE TABLE validators (
    id SERIAL PRIMARY KEY,
    namada_address VARCHAR NOT NULL,
    voting_power INT NOT NULL,
    max_commission VARCHAR NOT NULL,
    commission VARCHAR NOT NULL,
    email VARCHAR,
    website VARCHAR,
    description VARCHAR,
    discord_handle VARCHAR,
    avatar VARCHAR
);

ALTER TABLE validators
ADD UNIQUE (namada_address);

CREATE INDEX index_validators_namada_address ON validators USING HASH (namada_address);
