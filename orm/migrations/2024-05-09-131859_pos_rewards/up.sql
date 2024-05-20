-- Your SQL goes here

CREATE TABLE pos_rewards (
  id SERIAL PRIMARY KEY,
  owner VARCHAR NOT NULL,
  validator_id INT NOT NULL,
  raw_amount VARCHAR NOT NULL,
  CONSTRAINT fk_validator_id FOREIGN KEY(validator_id) REFERENCES validators(id) ON DELETE CASCADE
);

ALTER TABLE pos_rewards ADD UNIQUE (owner, validator_id);

CREATE INDEX index_pos_rewards_owner ON pos_rewards USING HASH  (owner);