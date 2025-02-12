-- Your SQL goes here

CREATE TABLE ibc_token_flows (
    id SERIAL PRIMARY KEY,
    address VARCHAR(45) NOT NULL,
    epoch INT NOT NULL,
    deposit NUMERIC(78, 0) NOT NULL,
    withdraw NUMERIC(78, 0) NOT NULL,
    -- reference the `address` column in the `token` table
    CONSTRAINT fk_ibc_token_flows_address
        FOREIGN KEY(address) REFERENCES token(address) ON DELETE CASCADE
);

ALTER TABLE ibc_token_flows ADD UNIQUE (address, epoch);
