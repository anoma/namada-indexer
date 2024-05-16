-- Your SQL goes here

CREATE TABLE unbonds (
    id SERIAL PRIMARY KEY,
    address VARCHAR NOT NULL,
    validator_id SERIAL references validators(id),
    raw_amount NUMERIC(78) NOT NULL,
    epoch INT NOT NULL,
    withdraw_epoch INT NOT NULL
);

ALTER TABLE unbonds
ADD UNIQUE (address, validator_id, withdraw_epoch, epoch);
