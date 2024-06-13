-- Your SQL goes here

CREATE TABLE block_crawler_state (
  id SERIAL PRIMARY KEY,
  height INT NOT NULL,
  epoch INT NOT NULL,
  timestamp BIGINT NOT NULL
);

ALTER TABLE block_crawler_state
ADD UNIQUE (height, epoch);
