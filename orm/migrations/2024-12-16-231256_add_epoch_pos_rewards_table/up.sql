-- Alters the pos_rewards table to add a fourth column:
-- epoch: INTEGER, representing the epoch number
-- Populate existing records with epoch = 0
ALTER TABLE pos_rewards ADD COLUMN epoch INTEGER NOT NULL DEFAULT 0;
-- Now we can safely drop the default
ALTER TABLE pos_rewards ALTER COLUMN epoch DROP DEFAULT;