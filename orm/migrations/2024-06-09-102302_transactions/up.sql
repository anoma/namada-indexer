CREATE TYPE TRANSACTION_KIND AS ENUM (
    'transparent_transfer', 
    'shielded_transfer', 
    'shielding_transfer', 
    'unshielding_transfer', 
    'ibc_msg_transfer',
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
    'become_validator',
    'unknown'
);

CREATE TYPE TRANSACTION_RESULT AS ENUM (
    'applied',
    'rejected'
);

CREATE TABLE wrapper_transactions (
    id VARCHAR(64) PRIMARY KEY,
    fee_payer VARCHAR NOT NULL,
    fee_token VARCHAR NOT NULL,
    gas_limit VARCHAR NOT NULL,
    block_height INT NOT NULL,
    exit_code TRANSACTION_RESULT NOT NULL,
    atomic BOOLEAN NOT NULL
);

CREATE TABLE inner_transactions (
    id VARCHAR(64) PRIMARY KEY,
    wrapper_id VARCHAR(64) NOT NULL,
    kind TRANSACTION_KIND NOT NULL,
    data VARCHAR,
    memo VARCHAR, -- hex serialized
    exit_code TRANSACTION_RESULT NOT NULL,
    CONSTRAINT fk_wrapper_id FOREIGN KEY(wrapper_id) REFERENCES wrapper_transactions(id) ON DELETE CASCADE
);

CREATE INDEX index_wrapper_transactions_block_height ON wrapper_transactions (block_height);
CREATE INDEX index_inner_transactions_memo ON inner_transactions (memo);
