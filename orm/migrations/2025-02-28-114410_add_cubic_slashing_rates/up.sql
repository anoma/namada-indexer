ALTER TABLE chain_parameters
ADD COLUMN duplicate_vote_min_slash_rate DECIMAL NOT NULL DEFAULT 0;

ALTER TABLE chain_parameters
ADD COLUMN light_client_attack_min_slash_rate DECIMAL NOT NULL DEFAULT 0;
