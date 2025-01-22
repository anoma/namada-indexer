-- Your SQL goes here
CREATE TYPE HISTORY_KIND AS ENUM ('received', 'sent');

CREATE TABLE transaction_history (
    id SERIAL PRIMARY KEY,
    inner_tx_id VARCHAR(64) NOT NULL,
    target VARCHAR NOT NULL,
    kind HISTORY_KIND NOT NULL,
    CONSTRAINT fk_inner_tx_id FOREIGN KEY(inner_tx_id) REFERENCES inner_transactions(id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX index_transaction_history_target_inner_tx_id ON transaction_history(inner_tx_id, target, kind);
CREATE INDEX index_transaction_history_target ON transaction_history (target);
