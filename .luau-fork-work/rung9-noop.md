# Rung 9 no-op rows

## R29 — `lua_newstate` (`VM/src/lstate.cpp`)

No executable change was required. The endpoint diff adds only a comment above
the unconditional `udatadirectfields` initialization loop. The Rust loop was
made unconditional in rung 1, so its behavior already matches the Rive tip.
