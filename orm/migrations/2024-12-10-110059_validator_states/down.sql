-- This file should undo anything in `up.sql`

-- Step 1: Rename the existing enum type
ALTER TYPE VALIDATOR_STATE RENAME TO VALIDATOR_STATE_OLD;

-- Step 2: Create the new enum type without the added values
CREATE TYPE VALIDATOR_STATE AS ENUM ('consensus', 'inactive', 'jailed', 'below_capacity', 'below_threshold', 'unknown');

-- Step 3: Update all columns to use the new enum type
ALTER TABLE validators ALTER COLUMN state TYPE VALIDATOR_STATE
USING state::text::VALIDATOR_STATE;

-- Step 4: Drop the old enum type
DROP TYPE VALIDATOR_STATE_OLD;
