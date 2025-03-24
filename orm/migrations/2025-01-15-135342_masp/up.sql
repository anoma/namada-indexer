-- Your SQL goes here
CREATE TYPE MASP_POOL_DIRECTION AS ENUM (
    'in',
    'out'
);

CREATE TABLE masp_pool (
    id SERIAL PRIMARY KEY,
    token_address VARCHAR(45) NOT NULL,
    timestamp TIMESTAMP NOT NULL,
    raw_amount NUMERIC(78, 0) NOT NULL,
    direction MASP_POOL_DIRECTION NOT NULL,
    inner_tx_id VARCHAR(64) NOT NULL,
    CONSTRAINT fk_inner_tx_id FOREIGN KEY(inner_tx_id) REFERENCES inner_transactions(id) ON DELETE CASCADE
);

CREATE INDEX index_masp_pool_address_timestamp ON masp_pool (token_address, timestamp DESC);
CREATE UNIQUE INDEX index_masp_pool_inner_tx_id ON masp_pool (inner_tx_id);

CREATE TYPE MASP_POOL_AGGREGATE_WINDOW AS ENUM (
    'one_day',
    'seven_days',
    'thirty_days',
    'all_time'
);

CREATE TYPE MASP_POOL_AGGREGATE_KIND AS ENUM (
    'inflows',
    'outflows'
);

CREATE TABLE masp_pool_aggregate (
    id SERIAL PRIMARY KEY,
    token_address VARCHAR(45) NOT NULL,
    time_window MASP_POOL_AGGREGATE_WINDOW NOT NULL,
    kind MASP_POOL_AGGREGATE_KIND NOT NULL,
    total_amount NUMERIC(78, 0) NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX index_masp_pool_aggregate_token_address_window_kind ON masp_pool_aggregate (token_address, time_window, kind);


CREATE OR REPLACE FUNCTION update_masp_pool_aggregate_sum()
RETURNS TRIGGER AS $$
DECLARE
  cutoff_1d TIMESTAMP := now() - INTERVAL '1 day';
  cutoff_7d TIMESTAMP := now() - INTERVAL '7 days';
  cutoff_30d TIMESTAMP := now() - INTERVAL '30 days';
  nk MASP_POOL_AGGREGATE_KIND; -- Declare kind as the ENUM type
BEGIN
  -- Determine the kind based on the direction
  nk := CASE
            WHEN NEW.direction = 'in' THEN 'inflows'::MASP_POOL_AGGREGATE_KIND
            ELSE 'outflows'::MASP_POOL_AGGREGATE_KIND
          END;
  -- 1 day
  INSERT INTO masp_pool_aggregate (token_address, time_window, kind, total_amount)
  VALUES (
    NEW.token_address,
    'one_day',
    nk,
    (SELECT COALESCE(SUM(raw_amount), 0)
     FROM masp_pool
     WHERE token_address = NEW.token_address
       AND direction = NEW.direction
       AND timestamp >= cutoff_1d)
  )
  ON CONFLICT (token_address, time_window, kind)
  DO UPDATE SET total_amount = (
    SELECT COALESCE(SUM(raw_amount), 0)
    FROM masp_pool
    WHERE token_address = NEW.token_address
      AND direction = NEW.direction
      AND timestamp >= cutoff_1d
  );

  -- 7 days
  INSERT INTO masp_pool_aggregate (token_address, time_window, kind, total_amount)
  VALUES (
    NEW.token_address,
    'seven_days',
    nk,
    (SELECT COALESCE(SUM(raw_amount), 0)
     FROM masp_pool
     WHERE token_address = NEW.token_address
       AND direction = NEW.direction
       AND timestamp >= cutoff_1d)
  )
  ON CONFLICT (token_address, time_window, kind) 
  DO UPDATE SET total_amount = (
    SELECT COALESCE(SUM(raw_amount), 0)
    FROM masp_pool
    WHERE token_address = NEW.token_address
      AND direction = NEW.direction
      AND timestamp >= cutoff_7d
  );

  -- 30 days
  INSERT INTO masp_pool_aggregate (token_address, time_window, kind, total_amount)
  VALUES (
    NEW.token_address,
    'thirty_days',
    nk,
    (SELECT COALESCE(SUM(raw_amount), 0)
     FROM masp_pool
     WHERE token_address = NEW.token_address
       AND direction = NEW.direction
       AND timestamp >= cutoff_1d)
  )
  ON CONFLICT (token_address, time_window, kind) 
  DO UPDATE SET total_amount = (
    SELECT COALESCE(SUM(raw_amount), 0)
    FROM masp_pool
    WHERE token_address = NEW.token_address
      AND direction = NEW.direction
      AND timestamp >= cutoff_30d
  );

  INSERT INTO masp_pool_aggregate (token_address, time_window, kind, total_amount)
  VALUES (
    NEW.token_address,
    'all_time',
    nk,
    (SELECT COALESCE(SUM(raw_amount), 0)
     FROM masp_pool
     WHERE token_address = NEW.token_address
       AND direction = NEW.direction)
  )
  ON CONFLICT (token_address, time_window, kind) 
  DO UPDATE SET total_amount = masp_pool_aggregate.total_amount + NEW.raw_amount;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_masp_pool_aggregate_sum_trigger
AFTER INSERT ON masp_pool
FOR EACH ROW
EXECUTE FUNCTION update_masp_pool_aggregate_sum();
