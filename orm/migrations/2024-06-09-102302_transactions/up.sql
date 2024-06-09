CREATE TYPE TRANSACTION_KIND AS ENUM (
    'transparent_transfer', 
    'shielded_transfer', 
    'shielding_transfer', 
    'unshielding_transfer', 
    'bond',
    'redelegation',
    'unbond',
    'withdraw',
    'claim_rewards',
    'vote_proposal',
    'init_proposal',
    'change_metadata',
    'change_commission',
    'reveal_pk',
    'unknown'
);

CREATE TABLE wrapper_transactions (
    id VARCHAR(32) PRIMARY KEY,
    fee_amount_per_gas_unit_amount VARCHAR NOT NULL,
    fee_amount_per_gas_unit_denomination VARCHAR NOT NULL,
    fee_token VARCHAR(32) NOT NULL,
    gas_limit VARCHAR NOT NULL,
    block_height INT NOT NULL,
    atomic BOOLEAN NOT NULL
);

CREATE TABLE inner_transactions (
    id VARCHAR(32) PRIMARY KEY,
    wrapper_id VARCHAR(32) NOT NULL,
    kind VARCHAR NOT NULL,
    data VARCHAR NOT NULL,
    memo VARCHAR, -- hex serialized
    exit_code INT NOT NULL,
    CONSTRAINT fk_wrapper_id FOREIGN KEY(wrapper_id) REFERENCES wrapper_transactions(id) ON DELETE CASCADE
);

CREATE INDEX index_wrapper_transactions_block_height ON wrapper_transactions (block_height);
CREATE INDEX index_inner_transactions_memo ON inner_transactions (memo);