-- Your SQL goes here

CREATE TABLE pos_rewards (
  id SERIAL PRIMARY KEY,
  owner VARCHAR NOT NULL,
  epoch INT NOT NULL,
  raw_amount VARCHAR NOT NULL
);

ALTER TABLE pos_rewards
ADD UNIQUE (owner, epoch);
