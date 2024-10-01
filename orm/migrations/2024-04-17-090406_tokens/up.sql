-- Your SQL goes here

CREATE TYPE TOKEN_TYPE AS ENUM ('native', 'ibc');

CREATE TABLE token (
    address VARCHAR(45) PRIMARY KEY,
    token_type TOKEN_TYPE NOT NULL
);

CREATE TABLE ibc_token (
    address VARCHAR(45) PRIMARY KEY,
    ibc_trace VARCHAR NOT NULL,
    CONSTRAINT fk_ibc_token_token FOREIGN KEY(address) REFERENCES token(address) ON DELETE CASCADE
);
