-- Your SQL goes here

CREATE TABLE token_supplies_per_epoch (
    id SERIAL PRIMARY KEY,
    address VARCHAR(45) NOT NULL,
    epoch INT NOT NULL,
    -- `2^256 - 1` will fit in `NUMERIC(78, 0)`
    total NUMERIC(78, 0) NOT NULL,
    effective NUMERIC(78, 0),
    -- reference the `address` column in the `token` table
    CONSTRAINT fk_token_supplies_per_epoch_address
        FOREIGN KEY(address) REFERENCES token(address) ON DELETE CASCADE
);

ALTER TABLE token_supplies_per_epoch ADD UNIQUE (address, epoch);

CREATE OR REPLACE FUNCTION check_effective_for_token_type()
RETURNS TRIGGER AS $$
BEGIN
    -- Check if the referenced token_type is 'native'
    IF EXISTS (
        SELECT 1
        FROM token
        WHERE token.address = NEW.address AND token.token_type = 'native'
    ) THEN
        -- If token_type is 'native', ensure token_supplies_per_epoch.effective is not NULL
        IF NEW.effective IS NULL THEN
            RAISE EXCEPTION 'token_supplies_per_epoch.effective cannot be NULL when token.token_type is ''native''';
        END IF;
    END IF;
    -- Allow the insert or update to proceed
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER enforce_effective_constraint
BEFORE INSERT OR UPDATE ON token_supplies_per_epoch
FOR EACH ROW
EXECUTE FUNCTION check_effective_for_token_type();
