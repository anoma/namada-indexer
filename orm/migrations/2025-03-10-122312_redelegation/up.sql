-- Your SQL goes here

CREATE TABLE redelegation (
    id SERIAL PRIMARY KEY,
    delegator VARCHAR NOT NULL,
    validator_id INT NOT NULL,
    epoch INT NOT NULL,
    CONSTRAINT fk_redelegation_validator_id FOREIGN KEY(validator_id) REFERENCES validators(id) ON DELETE CASCADE
);

ALTER TABLE redelegation ADD UNIQUE (delegator, validator_id);
