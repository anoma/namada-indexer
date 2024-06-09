-- Your SQL goes here
CREATE TABLE consensus_keys (
    id VARCHAR(32) PRIMARY KEY,
    validator_id INT NOT NULL,
    consensus_key INT NOT NULL,
    epoch INT NOT NULL,
    CONSTRAINT fk_validator_id FOREIGN KEY(validator_id) REFERENCES validators(id) ON DELETE CASCADE
);

ALTER TABLE consensus_keys ADD UNIQUE (validator_id, consensus_key, epoch);