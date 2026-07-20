//! `FValueVersionSetter` — a stateless helper whose constructor stamps a version
//! onto a flag by name (used by `LUAU_FLAGVERSION`). Reference:
//! `luau/Common/include/Luau/Common.h`. The work happens in the constructor
//! ([`FValueVersionSetter::new`]); the value itself carries no state.

pub struct FValueVersionSetter;
