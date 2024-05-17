-- Your SQL goes here

CREATE TABLE pos_rewards (
  id SERIAL PRIMARY KEY,
  owner VARCHAR NOT NULL,
  validator_id SERIAL references validators(id),
  raw_amount VARCHAR NOT NULL
);
