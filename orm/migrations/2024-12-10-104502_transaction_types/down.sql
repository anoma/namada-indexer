-- This file should undo anything in `up.sql`

-- Step 1: Rename the existing enum type
ALTER TYPE TRANSACTION_KIND RENAME TO TRANSACTION_KIND_OLD;

-- Step 2: Create the new enum type without the added values
CREATE TYPE TRANSACTION_KIND AS ENUM (
    'transparent_transfer', 
    'shielded_transfer', 
    'shielding_transfer', 
    'unshielding_transfer', 
    'ibc_msg_transfer',
    'bond',
    'redelegation',
    'unbond',
    'withdraw',
    'claim_rewards',
    'vote_proposal',
    'init_proposal',
    'change_metadata',
    'change_commission',
    'reveal_pk',
    'become_validator',
    'unknown'
);

-- Step 3: Update all columns to use the new enum type
ALTER TABLE inner_transactions ALTER COLUMN kind TYPE TRANSACTION_KIND
USING kind::text::TRANSACTION_KIND;

ALTER TABLE gas ALTER COLUMN tx_kind TYPE TRANSACTION_KIND
USING tx_kind::text::TRANSACTION_KIND;

-- Step 4: Drop the old enum type
DROP TYPE TRANSACTION_KIND_OLD;
