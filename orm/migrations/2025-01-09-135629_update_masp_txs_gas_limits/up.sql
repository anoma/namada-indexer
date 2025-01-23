-- Your SQL goes here

UPDATE gas SET gas_limit = 60_000 WHERE tx_kind = 'shielded_transfer';
UPDATE gas SET gas_limit = 60_000 WHERE tx_kind = 'unshielding_transfer';
