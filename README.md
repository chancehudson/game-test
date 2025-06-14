# game-test

A multiplayer 2d platformer.

## Install

`cargo install wasm-server-runner`: if you want to preview wasm

## Run

In 2 separate windows:

Start the server: `cargo run --bin server --features=server --no-default-features`
Start the client: `cargo run --bin=client --features=client --no-default-features`
(optional) Start the wasm client: `WASM_SERVER_RUNNER_ADDRESS=0.0.0.0 cargo run --target wasm32-unknown-unknown --bin=client --features=client`

<!--

## Game ideas

Maplestory + runescape

Maplestory with resources and crafting.

Skill trees:

- strength
- defense
- mining
- smithing
- cooking
- brewing
- magic
- farming

-->
