-- Your SQL goes here
CREATE TABLE masp_pool (
    id SERIAL PRIMARY KEY,
    token_address VARCHAR(45) NOT NULL,
    timestamp TIMESTAMP NOT NULL,
    raw_amount NUMERIC(78, 0) NOT NULL,
    inner_tx_id VARCHAR(64) NOT NULL,
    CONSTRAINT fk_inner_tx_id FOREIGN KEY(inner_tx_id) REFERENCES inner_transactions(id) ON DELETE CASCADE
);

CREATE INDEX index_masp_pool_address_timestamp ON masp_pool (token_address, timestamp DESC);

CREATE TYPE MASP_POOL_AGGREGATE_WINDOW AS ENUM (
    '1',
    '7',
    '30',
    'Inf'
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

-- if it doesn't work ask for a fix to https://chatgpt.com
CREATE OR REPLACE FUNCTION update_masp_pool_aggregate_sum()
RETURNS TRIGGER AS $$
-- 
-- This function is triggered before an insert into the `masp_pool` table.
-- It calculates the running sum of amounts for different time windows (1-day, 7-day, 30-day, and all-time).
-- Depending on whether the `raw_amount` is positive or negative, it updates the corresponding `inflow` or `outflow`
-- entry in the `masp_pool_aggregate` table. 
-- 
-- The `inflow` entry is updated if `raw_amount` is positive, while the `outflow` entry is updated if `raw_amount`
-- is negative. The sum is incrementally updated for each of the windows:
-- 1-day, 7-day, 30-day, and all-time.
--
-- The trigger ensures that the `masp_pool_aggregate` table reflects the running total of both inflow and outflow
-- amounts, for each token address, over different time windows.
--
DECLARE
  cutoff_1d TIMESTAMP := now() - INTERVAL '1 day';
  cutoff_7d TIMESTAMP := now() - INTERVAL '7 days';
  cutoff_30d TIMESTAMP := now() - INTERVAL '30 days';
BEGIN
  -- Update 1-day time_window for 'inflow' or 'outflow'
  IF NEW.raw_amount > 0 THEN
    -- Inflow: update inflow entry
    INSERT INTO masp_pool_aggregate (token_address, time_window, kind, total_amount)
    VALUES (
      NEW.token_address,
      '1d',
      'inflows',
      NEW.raw_amount
    )
    ON CONFLICT (token_address, time_window, kind)
    DO UPDATE SET total_amount = masp_pool_aggregate.total_amount + EXCLUDED.total_amount;
  ELSE
    -- Outflow: update outflow entry
    INSERT INTO masp_pool_aggregate (token_address, time_window, kind, total_amount)
    VALUES (
      NEW.token_address,
      '1d',
      'outflows',
      NEW.raw_amount
    )
    ON CONFLICT (token_address, time_window, kind)
    DO UPDATE SET total_amount = masp_pool_aggregate.total_amount + EXCLUDED.total_amount;
  END IF;

  -- Update 7-day time_window for 'inflow' or 'outflow'
  IF NEW.raw_amount > 0 THEN
    -- Inflow: update inflow entry
    INSERT INTO masp_pool_aggregate (token_address, time_window, kind, total_amount)
    VALUES (
      NEW.token_address,
      '7d',
      'inflows',
      NEW.raw_amount
    )
    ON CONFLICT (token_address, time_window, kind)
    DO UPDATE SET total_amount = masp_pool_aggregate.total_amount + EXCLUDED.total_amount;
  ELSE
    -- Outflow: update outflow entry
    INSERT INTO masp_pool_aggregate (token_address, time_window, kind, total_amount)
    VALUES (
      NEW.token_address,
      '7d',
      'outflows',
      NEW.raw_amount
    )
    ON CONFLICT (token_address, time_window, kind)
    DO UPDATE SET total_amount = masp_pool_aggregate.total_amount + EXCLUDED.total_amount;
  END IF;

  -- Update 30-day time_window for 'inflow' or 'outflow'
  IF NEW.raw_amount > 0 THEN
    -- Inflow: update inflow entry
    INSERT INTO masp_pool_aggregate (token_address, time_window, kind, total_amount)
    VALUES (
      NEW.token_address,
      '30d',
      'inflows',
      NEW.raw_amount
    )
    ON CONFLICT (token_address, time_window, kind)
    DO UPDATE SET total_amount = masp_pool_aggregate.total_amount + EXCLUDED.total_amount;
  ELSE
    -- Outflow: update outflow entry
    INSERT INTO masp_pool_aggregate (token_address, time_window, kind, total_amount)
    VALUES (
      NEW.token_address,
      '30d',
      'outflows',
      NEW.raw_amount
    )
    ON CONFLICT (token_address, time_window, kind)
    DO UPDATE SET total_amount = masp_pool_aggregate.total_amount + EXCLUDED.total_amount;
  END IF;

  -- Update all-time time_window for 'inflow' or 'outflow'
  IF NEW.raw_amount > 0 THEN
    -- Inflow: update inflow entry
    INSERT INTO masp_pool_aggregate (token_address, time_window, kind, total_amount)
    VALUES (
      NEW.token_address,
      'Inf',
      'inflows',
      NEW.raw_amount
    )
    ON CONFLICT (token_address, time_window, kind)
    DO UPDATE SET total_amount = masp_pool_aggregate.total_amount + EXCLUDED.total_amount;
  ELSE
    -- Outflow: update outflow entry
    INSERT INTO masp_pool_aggregate (token_address, time_window, kind, total_amount)
    VALUES (
      NEW.token_address,
      'Inf',
      'outflows',
      NEW.raw_amount
    )
    ON CONFLICT (token_address, time_window, kind)
    DO UPDATE SET total_amount = masp_pool_aggregate.total_amount + EXCLUDED.total_amount;
  END IF;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_masp_pool_aggregate_sum_trigger
AFTER INSERT ON masp_pool
FOR EACH ROW
EXECUTE FUNCTION update_masp_pool_aggregate_sum();