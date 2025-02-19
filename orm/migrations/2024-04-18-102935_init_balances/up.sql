-- Your SQL goes here

CREATE TABLE balance_changes (
  id SERIAL PRIMARY KEY,
  height INTEGER NOT NULL,
  owner VARCHAR NOT NULL,
  token VARCHAR(64) NOT NULL,
  raw_amount NUMERIC(78, 0) NOT NULL,
  CONSTRAINT fk_balances_token FOREIGN KEY(token) REFERENCES token(address) ON DELETE CASCADE
);

CREATE UNIQUE INDEX index_balance_changes_owner_token_height ON balance_changes (owner, token, height);

CREATE VIEW balances AS
SELECT
    bc.id,
    bc.owner,
    bc.token,
    bc.raw_amount
FROM
    balance_changes bc
    JOIN (
        SELECT
            owner,
            token,
            MAX(height) AS max_height
        FROM
            balance_changes
        GROUP BY
            owner,
            token) max_heights ON bc.owner = max_heights.owner
    AND bc.token = max_heights.token
    AND bc.height = max_heights.max_height;
