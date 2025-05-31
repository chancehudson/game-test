# server implementation

The server has a relatively low tickrate and does not simulate accurate physics. Instead it calculates future positions and sends them to the client for interpolation.

The client can then _guess_ if a play hits a mob based on where the mob will roughly be on the server. The server will give some amount of leeway in allowing hits based on packet rtt.

## Mobs

Server determines

- hit numbers
- drops
- positions
