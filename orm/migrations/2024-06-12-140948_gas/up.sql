CREATE TABLE gas (
    id SERIAL PRIMARY KEY,
    tx_kind TRANSACTION_KIND NOT NULL,
    token VARCHAR NOT NULL,
    gas_limit INT NOT NULL
);

ALTER TABLE gas ADD UNIQUE (tx_kind, token);

INSERT INTO gas (tx_kind, token, gas_limit)
VALUES ('transparent_transfer', 'native', 100000);

INSERT INTO gas (tx_kind, token, gas_limit)
VALUES ('shielded_transfer', 'native', 100000);

INSERT INTO gas (tx_kind, token, gas_limit)
VALUES ('shielding_transfer', 'native', 100000);

INSERT INTO gas (tx_kind, token, gas_limit)
VALUES ('unshielding_transfer', 'native', 100000);

INSERT INTO gas (tx_kind, token, gas_limit)
VALUES ('bond', 'native', 100000);

INSERT INTO gas (tx_kind, token, gas_limit)
VALUES ('redelegation', 'native', 100000);

INSERT INTO gas (tx_kind, token, gas_limit)
VALUES ('unbond', 'native', 100000);

INSERT INTO gas (tx_kind, token, gas_limit)
VALUES ('withdraw', 'native', 100000);

INSERT INTO gas (tx_kind, token, gas_limit)
VALUES ('claim_rewards', 'native', 100000);

INSERT INTO gas (tx_kind, token, gas_limit)
VALUES ('vote_proposal', 'native', 100000);

INSERT INTO gas (tx_kind, token, gas_limit)
VALUES ('init_proposal', 'native', 100000);

INSERT INTO gas (tx_kind, token, gas_limit)
VALUES ('change_metadata', 'native', 100000);

INSERT INTO gas (tx_kind, token, gas_limit)
VALUES ('change_commission', 'native', 100000);

INSERT INTO gas (tx_kind, token, gas_limit)
VALUES ('reveal_pk', 'native', 100000);

INSERT INTO gas (tx_kind, token, gas_limit)
VALUES ('unknown', 'native', 100000);
