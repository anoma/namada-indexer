-- This file should undo anything in `up.sql`

-- Drop the foreign key constraint on the address column
ALTER TABLE token_supplies_per_epoch
    DROP CONSTRAINT fk_token_supplies_per_epoch_address;

-- Drop the trigger for enforcing the effective constraint
DROP TRIGGER enforce_effective_constraint ON token_supplies_per_epoch;

-- Drop the table itself
DROP TABLE token_supplies_per_epoch;
