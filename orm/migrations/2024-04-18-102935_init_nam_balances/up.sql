-- Your SQL goes here

CREATE TABLE nam_balances (
  id SERIAL PRIMARY KEY,
  address VARCHAR NOT NULL,
  raw_amount NUMERIC(78) NOT NULL
);

ALTER TABLE nam_balances
ADD UNIQUE (address);
