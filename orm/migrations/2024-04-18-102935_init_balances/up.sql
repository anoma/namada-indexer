-- Your SQL goes here

CREATE TABLE balances (
  id SERIAL PRIMARY KEY,
  owner VARCHAR NOT NULL,
  token VARCHAR(64) NOT NULL,
  raw_amount NUMERIC(78, 0) NOT NULL,
  CONSTRAINT fk_balances_token FOREIGN KEY(token) REFERENCES token(address) ON DELETE CASCADE
);

ALTER TABLE balances ADD UNIQUE (owner, token);

CREATE INDEX index_balances_owner ON balances (owner, token);
