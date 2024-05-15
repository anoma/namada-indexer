-- Your SQL goes here

CREATE TABLE validators (
    id SERIAL PRIMARY KEY,
    namada_address VARCHAR NOT NULL,
    voting_power INT NOT NULL,
    max_commission VARCHAR NOT NULL,
    commission VARCHAR NOT NULL,
    email VARCHAR NOT NULL,
    website VARCHAR,
    description VARCHAR,
    discord_handle VARCHAR,
    avatar VARCHAR
);

ALTER TABLE validators
ADD UNIQUE (namada_address);
