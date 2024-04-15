# Shielded Expedition API

- Get scoreboard
    - GET /api/v1/scoreboard/pilots?total=<...>&page=<...>&offset=<...>
    - GET /api/v1/scoreboard/crews?total=<...>&page=<...>&offset=<...>
- Get player by `id`
    - GET /api/v1/player/:id
- Search player by `moniker` or `address` or `public key`
    - GET /api/v1/player/search?moniker=<...>&address=<...>&pk=<...>