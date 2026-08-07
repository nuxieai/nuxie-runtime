# Nuxie vendored luaur-rt 0.1.8 (fork baseline)

This directory vendors the crates.io `luaur-rt` 0.1.8 package byte-for-byte
as the baseline of the in-house luaur fork (see `docs/luau-fork.md`). Local
changes are enumerated in `NUXIE_PATCH.md`.

Provenance:

- Package: crates.io `luaur-rt` 0.1.8
- Original package checksum:
  `e2f75240c12ca15167dae8542d7d477d28a58491e33a752c4e160f424dc6e588`
- Upstream repository: `https://github.com/pjankiewicz/luaur`
- Upstream commit: `f0eac7f7cce691d0cdb0b93c3eef9d599f71d739`
  (`crates/luaur-rt`, per `.cargo_vcs_info.json`)
- Luau base: 0.724-era (upstream Luau commit `8f33df9` per the luaur README)
- Patches: see `NUXIE_PATCH.md`
