-- Your SQL goes here
ALTER TABLE gas_estimations ALTER COLUMN ibc_unshielding_transfer DROP DEFAULT;
ALTER TABLE gas_estimations ALTER COLUMN ibc_shielding_transfer DROP DEFAULT;
