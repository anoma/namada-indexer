-- Your SQL goes here
CREATE TABLE blocks (
    id VARCHAR(32) PRIMARY KEY,
    block_height INT NOT NULL,
    epoch INT NOT NULL,
    app_hash VARCHAR(32) NOT NULL,
    validator_id INT NOT NULL,
    CONSTRAINT fk_validator_id FOREIGN KEY(validator_id) REFERENCES validators(id) ON DELETE CASCADE
);