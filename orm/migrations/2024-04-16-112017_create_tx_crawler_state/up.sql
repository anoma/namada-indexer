-- Your SQL goes here

CREATE TABLE tx_crawler_state (
  id SERIAL PRIMARY KEY,
  height INT NOT NULL,
  epoch INT NOT NULL
);

ALTER TABLE tx_crawler_state
ADD UNIQUE (height, epoch);
