CREATE TABLE chain_parameters (
  id SERIAL PRIMARY KEY,
  unbonding_length INT NOT NULL,
  pipeline_length INT NOT NULL,
  epochs_per_year INT NOT NULL,
  min_num_of_blocks INT NOT NULL,
  min_duration INT NOT NULL,
  apr VARCHAR NOT NULL,
  native_token_address VARCHAR NOT NULL,
  chain_id VARCHAR NOT NULL,
  genesis_time BIGINT NOT NULL,
  epoch_switch_blocks_delay INT NOT NULL
);

ALTER TABLE chain_parameters ADD UNIQUE (chain_id);
