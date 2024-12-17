-- Your SQL goes here
CREATE TABLE blocks (
    height integer PRIMARY KEY,
    hash VARCHAR(64),
    app_hash varchar(64),
    timestamp timestamp,
    proposer varchar,
    epoch int
);

ALTER TABLE blocks
    ADD UNIQUE (hash);

CREATE INDEX index_blocks_epoch ON blocks (epoch);

-- Populate null blocks for all existing wrapper_transactions and balance_changes to satisfy foreign key constraints
INSERT INTO blocks ( SELECT DISTINCT
        height,
        NULL::varchar AS hash,
        NULL::varchar AS app_hash,
        NULL::timestamp AS timestamp,
        NULL::varchar AS proposer,
        NULL::int AS epoch
    FROM ( SELECT DISTINCT
            block_height AS height
        FROM
            wrapper_transactions
        UNION
        SELECT DISTINCT
            height
        FROM
            balance_changes));

-- Create foreign key constraints for wrapper_transactions and balance_changes
ALTER TABLE wrapper_transactions
    ADD CONSTRAINT fk_wrapper_transactions_height FOREIGN KEY (block_height) REFERENCES blocks (height) ON DELETE RESTRICT;

ALTER TABLE balance_changes
    ADD CONSTRAINT fk_balance_changes_height FOREIGN KEY (height) REFERENCES blocks (height) ON DELETE RESTRICT;

