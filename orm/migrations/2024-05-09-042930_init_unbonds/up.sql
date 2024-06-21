-- Your SQL goes here

CREATE TABLE unbonds (
    id SERIAL PRIMARY KEY,
    address VARCHAR NOT NULL,
    validator_id INT NOT NULL,
    raw_amount NUMERIC(78) NOT NULL,
    withdraw_epoch INT NOT NULL,
    CONSTRAINT fk_validator_id FOREIGN KEY(validator_id) REFERENCES validators(id) ON DELETE CASCADE
);

ALTER TABLE unbonds ADD UNIQUE (address, validator_id, withdraw_epoch);

CREATE INDEX index_unbonds_owner_withdraw_epoch ON unbonds (address, withdraw_epoch);
