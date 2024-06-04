CREATE TABLE chain_parameters (
  epoch INT PRIMARY KEY,
  unbonding_length INT NOT NULL,
  pipeline_length INT NOT NULL,
  epochs_per_year INT NOT NULL,
  max_signatures_per_transaction INT NOT NULL,
);