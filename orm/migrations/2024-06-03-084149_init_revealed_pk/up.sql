-- Your SQL goes here

CREATE TABLE revealed_pk (
    id SERIAL PRIMARY KEY,
    address VARCHAR NOT NULL,
    pk VARCHAR NOT NULL
);

ALTER TABLE revealed_pk ADD UNIQUE (address, pk);
