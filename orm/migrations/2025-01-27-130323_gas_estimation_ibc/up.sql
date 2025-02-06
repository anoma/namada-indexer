-- Your SQL goes here
ALTER TABLE gas_estimations ADD COLUMN ibc_unshielding_transfer INT NOT NULL;
ALTER TABLE gas_estimations ADD COLUMN ibc_shielding_transfer INT NOT NULL;