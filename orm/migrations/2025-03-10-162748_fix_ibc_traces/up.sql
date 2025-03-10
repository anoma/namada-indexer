-- Your SQL goes here

DELETE FROM ibc_token WHERE
  ibc_token.ibc_trace = 'transfer/channel-2/transfer/channel-1317/transfer/channel-1/uosmo';

DELETE FROM token WHERE
  token.address = 'tnam1ph6xwxde08cth5vvees72uwklafy3d9t5s7ckvcs';
