-- Your SQL goes here
ALTER TABLE gas_estimations ADD COLUMN ibc_unshielding_transfer INT NOT NULL DEFAULT 0;
ALTER TABLE gas_estimations ADD COLUMN ibc_shielding_transfer INT NOT NULL DEFAULT 0;

ALTER TABLE gas_estimations ADD COLUMN token VARCHAR;

UPDATE gas_estimations 
SET token = (SELECT address FROM "token" WHERE token_type = 'native' LIMIT 1)
WHERE token IS NULL;

ALTER TABLE gas_estimations ALTER COLUMN token SET NOT NULL;