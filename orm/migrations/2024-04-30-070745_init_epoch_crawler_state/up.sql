-- Your SQL goes here

CREATE TABLE epoch_crawler_state (
  id SERIAL PRIMARY KEY,
  epoch INT NOT NULL
);

ALTER TABLE epoch_crawler_state
ADD UNIQUE (epoch);
