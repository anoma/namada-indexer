-- This file should undo anything in `up.sql`
ALTER TABLE gas_estimations DROP COLUMN ibc_unshielding_transfer;
ALTER TABLE gas_estimations DROP COLUMN ibc_shielding_transfer;