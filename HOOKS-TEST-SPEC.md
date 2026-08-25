# Prove the android render-path asset wrap (one finding + a verified fact)

Branch portable-asset-hooks.

## The verified fact you must design around

Mutating android_vulkan.rs to disable the wrap branch
(`.filter(|_| false)` on the asset_hooks check) does NOT fail the current
probe test - verified by running the mutant. Import-time hooks already
decode the image, the runtime retains it, and this render scenario never
consults the wrap. A pixel assertion (stub-white present, PNG-red absent)
was added and ALSO survives the mutant; it stays as a sanity check but
proves nothing about the wrap.

## Task

1. Determine from the engine source when the render-path wrap is
   actually consulted - study how apple_metal's identical wrap gets
   exercised (renderer-domain rebinding? recreation/resize forcing image
   re-upload through the factory? lazily-realized assets like fonts?).
   Write the answer as a comment above the android wrap branch.

2. Extend or add a test that exercises that path for real on the android
   arm (e.g. render, reset the player domain or create a second
   renderer, render again, and assert the hook counters advance or the
   re-uploaded pixels are stub-supplied).

3. Acceptance bar: the new test MUST FAIL against the mutant
   (`.filter(|_| false)` on the wrap branch) and pass on the real code.
   Run both directions and say so in the commit message.

If the investigation concludes the wrap is genuinely unreachable on the
android arm (nothing ever re-realizes through the render factory), say
that plainly instead of writing a vacuous test, delete the dead branch,
and document why apple needs it and android does not.

## Acceptance

export RUSTC=$(rustup which rustc --toolchain stable)
export NUXIE_MOLTENVK_LIBRARY=/Users/levi/dev/oss/rive-runtime/renderer/dependencies/MoltenVK/Package/Release/MoltenVK/dynamic/dylib/macOS/libMoltenVK.dylib
cargo test -p nux-capi --features android-vulkan,scripting
(the pre-existing data_binding red on main, UNIV-2653, is excluded)

Commit on the branch; do not push.
