-- This file should undo anything in `up.sql`

UPDATE gas SET gas_limit = 50_000 WHERE tx_kind = 'shielded_transfer';
UPDATE gas SET gas_limit = 50_000 WHERE tx_kind = 'unshielding_transfer';
