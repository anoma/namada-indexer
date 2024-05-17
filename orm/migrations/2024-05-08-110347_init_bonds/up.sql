-- Your SQL goes here

CREATE TABLE bonds (
    id SERIAL PRIMARY KEY,
    address VARCHAR NOT NULL,
    validator_id INT NOT NULL,
    raw_amount VARCHAR NOT NULL,
    CONSTRAINT fk_validator_id FOREIGN KEY(validator_id) REFERENCES validators(id) ON DELETE CASCADE
);

ALTER TABLE bonds ADD UNIQUE (address, validator_id);

CREATE INDEX index_bonds_owner ON bonds USING HASH  (address);