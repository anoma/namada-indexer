-- This file should undo anything in `up.sql`
DROP TABLE IF EXISTS inner_transactions;

DROP TABLE IF EXISTS wrapper_transactions;

DROP TYPE TRANSACTION_KIND;
DROP TYPE TRANSACTION_RESULT;
