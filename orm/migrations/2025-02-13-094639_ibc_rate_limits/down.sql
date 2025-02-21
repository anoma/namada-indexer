-- This file should undo anything in `up.sql`

ALTER TABLE ibc_rate_limits
    DROP CONSTRAINT fk_ibc_rate_limits_address;

DROP TABLE ibc_rate_limits;
