-- Your SQL goes here
CREATE TABLE gas (
    id SERIAL PRIMARY KEY,
    tx_kind TRANSACTION_KIND NOT NULL,
    token VARCHAR NOT NULL,
    gas INT NOT NULL
);

ALTER TABLE gas ADD UNIQUE (tx_kind, token);
