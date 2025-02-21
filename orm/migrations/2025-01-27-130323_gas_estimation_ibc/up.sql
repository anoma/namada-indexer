-- Your SQL goes here
ALTER TABLE gas_estimations ADD COLUMN ibc_unshielding_transfer INT NOT NULL DEFAULT 0;
ALTER TABLE gas_estimations ADD COLUMN ibc_shielding_transfer INT NOT NULL DEFAULT 0;
