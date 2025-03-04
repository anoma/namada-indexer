DELETE FROM ibc_token WHERE
  ibc_token.ibc_trace = 'transfer/channel-2/transfer/channel-1/uosmo'
  OR ibc_token.ibc_trace = 'transfer/channel-2/transfer/channel-3/utia';

DELETE FROM token WHERE
  token.address = 'tnam1ph08wfm0yxwqg97ckp8uq528440ng09v7576hcct'
  OR token.address = 'tnam1phw9rpsw0fxu04l4cy84383uszpswfahy54d3f0s'
  OR token.address = 'tnam1p5zt6shz5czmy37w94us08dld927lpxaautc32r8'
  OR token.address = 'tnam1p5f9fcev839rm092pu96zfutkd3p7kp2vcqjl4s0'
  OR token.address = 'tnam1p5sg86y9sasgf36ypstcxdr4z33g7lprtcrf066q'