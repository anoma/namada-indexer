-- Your SQL goes here
CREATE TABLE gas_estimations (
    id SERIAL PRIMARY KEY,
    wrapper_id VARCHAR(64) NOT NULL,
    transparent_transfer INT NOT NULL, 
    shielded_transfer INT NOT NULL, 
    shielding_transfer INT NOT NULL, 
    unshielding_transfer INT NOT NULL, 
    ibc_msg_transfer INT NOT NULL,
    bond INT NOT NULL,
    redelegation INT NOT NULL,
    unbond INT NOT NULL,
    withdraw INT NOT NULL,
    claim_rewards INT NOT NULL,
    vote_proposal INT NOT NULL,
    reveal_pk INT NOT NULL,
    tx_size INT NOT NULL,
    signatures INT NOT NULL,
    CONSTRAINT fk_wrapper_id FOREIGN KEY(wrapper_id) REFERENCES wrapper_transactions(id) ON DELETE CASCADE
);

ALTER TABLE wrapper_transactions ALTER COLUMN gas_used TYPE INTEGER  USING (gas_used::integer) ;

CREATE INDEX wrapper_transactions_gas ON wrapper_transactions (gas_used);

CREATE INDEX inner_transactions_kind ON inner_transactions (kind);