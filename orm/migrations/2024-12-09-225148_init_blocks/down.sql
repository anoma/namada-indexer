-- This file should undo anything in `up.sql`
ALTER TABLE balance_changes
    DROP CONSTRAINT fk_balance_changes_height;

ALTER TABLE wrapper_transactions
    DROP CONSTRAINT fk_wrapper_transactions_height;

DROP TABLE IF EXISTS blocks;

