-- This file should undo anything in `up.sql`
ALTER TYPE transaction_kind RENAME value 'transfer' TO 'transparent_transfer';

