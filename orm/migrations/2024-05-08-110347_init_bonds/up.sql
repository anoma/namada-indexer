-- Your SQL goes here

CREATE TABLE bonds (
    id SERIAL PRIMARY KEY,
    address VARCHAR NOT NULL,
    validator_id SERIAL references validators(id)
);

ALTER TABLE bonds
ADD UNIQUE (address, validator_id);
