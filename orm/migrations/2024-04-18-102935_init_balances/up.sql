-- Your SQL goes here

CREATE TABLE balances (
  id SERIAL PRIMARY KEY,
  owner VARCHAR NOT NULL,
  token VARCHAR NOT NULL,
  raw_amount VARCHAR NOT NULL
);

ALTER TABLE balances ADD UNIQUE (owner, token);

CREATE INDEX index_balances_owner ON balances (owner, token);
