-- Your SQL goes here
ALTER TYPE TRANSACTION_KIND ADD VALUE 'ibc_transparent_transfer';
ALTER TYPE TRANSACTION_KIND ADD VALUE 'ibc_shielding_transfer';
ALTER TYPE TRANSACTION_KIND ADD VALUE 'ibc_unshielding_transfer';