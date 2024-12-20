-- This file should undo anything in `up.sql`
DROP TABLE IF EXISTS gas_estimations;

DROP INDEX IF EXISTS wrapper_transactions_gas;
DROP INDEX IF EXISTS inner_transactions_kind;