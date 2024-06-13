CREATE TABLE chain_parameters (
  id SERIAL PRIMARY KEY,
  unbonding_length INT NOT NULL,
  pipeline_length INT NOT NULL,
  epochs_per_year INT NOT NULL,
  min_num_of_blocks INT NOT NULL,
  min_duration INT NOT NULL,
  apr VARCHAR NOT NULL,
  native_token_address VARCHAR NOT NULL
);

ALTER TABLE chain_parameters ADD UNIQUE (native_token_address);
