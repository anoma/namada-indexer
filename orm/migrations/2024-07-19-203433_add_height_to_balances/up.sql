-- Your SQL goes here
ALTER TABLE balances
    ADD COLUMN height integer NOT NULL DEFAULT 0;

ALTER TABLE balances
    ALTER COLUMN height DROP DEFAULT;

ALTER TABLE balances
    DROP CONSTRAINT balances_owner_token_key;

ALTER TABLE balances RENAME TO balance_changes;

ALTER TABLE balance_changes
    ADD CONSTRAINT balance_changes_owner_token_height_key UNIQUE (OWNER, token, height);

CREATE INDEX index_balance_changes_owner_token_height ON balance_changes (OWNER, token, height);

DROP INDEX index_balances_owner;

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
            OWNER,
            token,
            MAX(height) AS max_height
        FROM
            balance_changes
        GROUP BY
            OWNER,
            token) max_heights ON bc.owner = max_heights.owner
    AND bc.token = max_heights.token
    AND bc.height = max_heights.max_height;

