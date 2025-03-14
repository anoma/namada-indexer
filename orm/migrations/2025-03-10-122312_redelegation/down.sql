DROP TABLE IF EXISTS redelegation;

ALTER TABLE chain_parameters DROP COLUMN IF EXISTS slash_processing_epoch_offset;
