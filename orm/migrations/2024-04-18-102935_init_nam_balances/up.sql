-- Your SQL goes here

CREATE TABLE nam_balances (
  id SERIAL PRIMARY KEY,
  owner VARCHAR NOT NULL,
  raw_amount NUMERIC(78) NOT NULL
);

ALTER TABLE nam_balances
ADD UNIQUE (owner);
