-- This file should undo anything in `up.sql`
DROP VIEW balances;

CREATE INDEX index_balances_owner ON balance_changes (OWNER, token);

DROP INDEX index_balance_changes_owner_token_height;

ALTER TABLE balance_changes
    DROP CONSTRAINT balance_changes_owner_token_height_key;

ALTER TABLE balance_changes RENAME TO balances;

ALTER TABLE balances
    ADD CONSTRAINT balances_owner_token_key UNIQUE (OWNER, token);

ALTER TABLE balances
    DROP COLUMN height;

