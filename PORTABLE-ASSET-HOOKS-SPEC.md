# Portable asset hooks (UNIV-2652): external assets reach every platform

## Context

crates/nux-capi/src/apple_assets.rs (1,924 lines) implements external
asset lookup + host image decode + the bounded catalog, and is already
platform-neutral Rust - only its EXPORT is Apple-gated
(NUX_CAPI_APPLE_METAL + __APPLE__ in the generated header, apple-metal
feature in Cargo.toml), and only the apple_metal render path consults the
catalog (player.artboard.apple_assets -> assets.wrap_factory(...) around
artboard.draw). Android acquires and verifies external images, fonts,
and screenBehaviors scripts but has no way to hand them to the engine.

Pre-GA posture: hard cutovers, no compat shims. The shipped iOS SDK pins
release apple-runtime-v0.7.0, so renaming symbols on main breaks nothing
shipped; iOS adopts the portable names on its next roll.

## Work

1. Rename the surface portable, one clean cut (no aliases):
   - module apple_assets -> asset_hooks (file rename included)
   - NuxAppleAssetHooks -> NuxAssetHooks
   - nux_file_import_with_apple_assets -> nux_file_import_with_assets
   - AppleAssetCatalog and every Apple* internal type -> Asset*
   - the `apple_assets` field on file/artboard state -> `asset_hooks`
     (or `assets` - pick one and be consistent)
   Update every reference including apple_metal.rs and tests.

2. Gate the exports for BOTH platform arms: available under apple-metal
   (as today) AND android-vulkan. The header must emit the portable
   symbols for both. Regenerate with NUX_CAPI_UPDATE_HEADER=1.

3. Wire the catalog into the Android render path: in
   crates/nux-capi/src/android_vulkan.rs render_player, mirror
   apple_metal's draw exactly - when the player's artboard carries asset
   hooks, wrap the factory (assets.wrap_factory(&mut state.factory)) and
   draw through it; otherwise draw as today.

4. Tests:
   - Existing apple_assets tests keep passing under the new names.
   - Add an android-vulkan-gated test that imports a file with
     nux_file_import_with_assets using stub hooks and renders through
     the android arm, proving lookup_external_asset is consulted from
     that path (a counting stub suffices; live Vulkan optional with the
     existing NUXIE_REQUIRE_LIVE_VULKAN_TESTS convention).

5. Keep behavior byte-identical for Apple: no changes to hook semantics,
   bounds, or callback contracts - this is a rename + gate-widening +
   android wiring, not a redesign.

## Acceptance

export RUSTC=$(rustup which rustc --toolchain stable)
cargo check -p nux-capi --features apple-metal
cargo test -p nux-capi --features android-vulkan,scripting
export ANDROID_NDK_HOME=/opt/homebrew/share/android-commandlinetools/ndk/27.2.12479018
cargo ndk -t arm64-v8a -t x86_64 build --release -p nux-capi --features android-vulkan,scripting

All green; header regenerated and committed. Commit in logical commits
on this branch; do not push.
