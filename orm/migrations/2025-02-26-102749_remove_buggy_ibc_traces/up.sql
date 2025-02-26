-- Your SQL goes here

-- These are all the tokens enabled by phase 3
-- (https://github.com/anoma/namada-governance-upgrades/blob/a9b40c048c4be587ac567bc65f59ffb5505b692d/phase3/src/lib.rs#L16-L65)
DELETE FROM ibc_token WHERE
  ibc_token.ibc_trace = 'transfer/channel-0/transfer/channel-0/stuosmo'
  OR ibc_token.ibc_trace = 'transfer/channel-0/transfer/channel-0/stuatom'
  OR ibc_token.ibc_trace = 'transfer/channel-0/transfer/channel-0/stutia'
  OR ibc_token.ibc_trace = 'transfer/channel-1/transfer/channel-1/uosmo'
  OR ibc_token.ibc_trace = 'transfer/channel-2/transfer/channel-2/uatom'
  OR ibc_token.ibc_trace = 'transfer/channel-3/transfer/channel-3/utia';
