-- Alters the pos_rewards table to add a fourth column:
-- epoch: INTEGER, representing the epoch number
ALTER TABLE pos_rewards ADD COLUMN epoch INTEGER NOT NULL;