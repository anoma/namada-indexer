-- Your SQL goes here

CREATE TABLE balances (
  id SERIAL PRIMARY KEY,
  owner VARCHAR NOT NULL,
  token VARCHAR NOT NULL,
  raw_amount NUMERIC(78) NOT NULL
);

ALTER TABLE balances
ADD UNIQUE (owner);
