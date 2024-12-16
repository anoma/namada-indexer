-- Creates the pos_rewards table with four columns:
-- owner: TEXT, representing the delegator address
-- validator_id: INTEGER, representing the validator identifier
-- raw_amount: NUMERIC, holding large numeric values for rewards
-- epoch: INTEGER, representing the epoch number
CREATE TABLE pos_rewards (
    owner TEXT NOT NULL,
    validator_id INTEGER NOT NULL,
    raw_amount NUMERIC NOT NULL,
    epoch INTEGER NOT NULL
);
