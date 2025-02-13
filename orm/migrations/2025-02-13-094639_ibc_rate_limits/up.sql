-- Your SQL goes here

CREATE TABLE ibc_rate_limits (
    id SERIAL PRIMARY KEY,
    address VARCHAR(45) NOT NULL,
    epoch INT NOT NULL,
    throughput_limit NUMERIC(78, 0) NOT NULL,
    -- reference the `address` column in the `token` table
    CONSTRAINT fk_ibc_rate_limits_address
        FOREIGN KEY(address) REFERENCES token(address) ON DELETE CASCADE
);

ALTER TABLE ibc_rate_limits ADD UNIQUE (address, epoch);
