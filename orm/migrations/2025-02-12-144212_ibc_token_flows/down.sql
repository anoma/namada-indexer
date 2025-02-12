-- This file should undo anything in `up.sql`

ALTER TABLE ibc_token_flows
    DROP CONSTRAINT fk_ibc_token_flows_address;

DROP TABLE ibc_token_flows;
