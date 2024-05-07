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
    avatar VARCHAR,
    epoch INT NOT NULL
);

ALTER TABLE validators
ADD UNIQUE (namada_address, epoch);

CREATE INDEX epoch_asc ON validators (epoch ASC);
CREATE INDEX epoch_desc ON validators (epoch DESC);
