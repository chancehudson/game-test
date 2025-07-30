# keind

A simple, deterministic, game engine. Designed for use in multiplayer 2d platformers.

## Design

Each engine represents a 2d space containing entities. Entities contain systems which store data and logic for the entity. Entities and systems are copy on write, and decide each step whether to copy and mutate.


