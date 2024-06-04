CREATE TYPE GOVERNANCE_KIND AS ENUM ('pgf_steward', 'pgf_funding', 'default', 'default_with_wasm');
CREATE TYPE GOVERNANCE_RESULT AS ENUM ('passed', 'rejected', 'pending', 'unknown', 'voting_period');
CREATE TYPE GOVERNANCE_TALLY_TYPE AS ENUM ('two_thirds', 'one_half_over_one_third', 'less_one_half_over_one_third_nay');

CREATE TABLE governance_proposals (
  id INT PRIMARY KEY,
  content VARCHAR NOT NULL,
  data VARCHAR,
  kind GOVERNANCE_KIND NOT NULL,
  tally_type GOVERNANCE_TALLY_TYPE NOT NULL,
  author VARCHAR NOT NULL,
  start_epoch INT NOT NULL,
  end_epoch INT NOT NULL,
  activation_epoch INT NOT NULL,
  result GOVERNANCE_RESULT NOT NULL DEFAULT 'pending',
  yay_votes VARCHAR NOT NULL DEFAULT '0',
  nay_votes VARCHAR NOT NULL DEFAULT '0',
  abstain_votes VARCHAR NOT NULL DEFAULT '0'
);

CREATE INDEX index_governance_proposals_kind ON governance_proposals USING HASH  (kind);
CREATE INDEX index_governance_proposals_result ON governance_proposals USING HASH  (result);
