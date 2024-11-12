-- This file should undo anything in `up.sql`
DROP INDEX one_native_token;

DROP TABLE IF EXISTS ibc_token;
DROP TABLE IF EXISTS token;

DROP TYPE TOKEN_TYPE;
