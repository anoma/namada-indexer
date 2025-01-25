-- Your SQL goes here
ALTER TYPE TRANSACTION_KIND ADD VALUE 'reactivate_validator';
ALTER TYPE TRANSACTION_KIND ADD VALUE 'deactivate_validator';
ALTER TYPE TRANSACTION_KIND ADD VALUE 'unjail_validator';