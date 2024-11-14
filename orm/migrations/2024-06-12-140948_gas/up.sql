CREATE TABLE gas (
    id SERIAL PRIMARY KEY,
    tx_kind TRANSACTION_KIND NOT NULL,
    gas_limit INT NOT NULL
);

ALTER TABLE gas ADD UNIQUE (tx_kind);

INSERT INTO gas (tx_kind, gas_limit)
VALUES ('transparent_transfer', 50_000);

INSERT INTO gas (tx_kind, gas_limit)
VALUES ('shielded_transfer', 50_000);

INSERT INTO gas (tx_kind, gas_limit)
VALUES ('shielding_transfer', 50_000);

INSERT INTO gas (tx_kind, gas_limit)
VALUES ('unshielding_transfer', 50_000);

INSERT INTO gas (tx_kind, gas_limit)
VALUES ('bond', 50_000);

INSERT INTO gas (tx_kind, gas_limit)
VALUES ('redelegation', 250_000);

INSERT INTO gas (tx_kind, gas_limit)
VALUES ('unbond', 150_000);

INSERT INTO gas (tx_kind, gas_limit)
VALUES ('withdraw', 150_000);

INSERT INTO gas (tx_kind, gas_limit)
VALUES ('claim_rewards', 50_000);

INSERT INTO gas (tx_kind, gas_limit)
VALUES ('vote_proposal', 50_000);

INSERT INTO gas (tx_kind, gas_limit)
VALUES ('init_proposal', 50_000);

INSERT INTO gas (tx_kind, gas_limit)
VALUES ('change_metadata', 50_000);

INSERT INTO gas (tx_kind, gas_limit)
VALUES ('change_commission', 50_000);

INSERT INTO gas (tx_kind, gas_limit)
VALUES ('reveal_pk', 50_000);

INSERT INTO gas (tx_kind, gas_limit)
VALUES ('become_validator', 50_000);

INSERT INTO gas (tx_kind, gas_limit)
VALUES ('ibc_msg_transfer', 50_000);

INSERT INTO gas (tx_kind, gas_limit)
VALUES ('unknown', 50_000);
