#![cfg(feature = "scripting")]

//! Editor-emitted bytecode compatibility matrix acceptance
//! (docs/luau-fork.md, "Editor-emitted bytecode compatibility matrix").
//!
//! The fixture is a real Nuxie Editor scene publish: the e4 scripted-vector
//! snapshot materialized through the production `editor-publisher-wasm`
//! `publish()` path (nuxie-dev `4a63abca`), with the script live-compiled by
//! `scripted-resource-compiler` at its flags-off floor (all Luau FFlags
//! forced off during compile, emitting LBC v7). The recorded hash below is
//! the compiled bytecode's SHA-256 captured BEFORE materialization; the row
//! is accepted only while the ScriptAsset bytes re-extracted from the .riv
//! hash-match it.

use nuxie::{
    File, FileImportLimits, PersistentFactory, RecordingFactory, ScriptExecutionLimits,
    import_unsigned_scripted, runtime::assets::script_asset::ScriptAsset,
};
use sha2::{Digest as _, Sha256};

const EDITOR_SCRIPTED_VECTOR_V7: &[u8] =
    include_bytes!("../../../fixtures/editor/editor_scripted_vector_v7.riv");

/// SHA-256 of `scripted_resource_compiler::compile_luau_bytecode` output for
/// the e4 `scripted-vector.luau` source at the emitter commit — the same
/// pre-materialization hash the compiler crate's own contract test pins.
const EMITTED_BYTECODE_SHA256: &str =
    "50d69e465eb4413f342a38b1c6c3dbb71531559c98d0a0514a0d2b782ed477bd";

const EMITTED_BYTECODE_VERSION: u8 = 7;

#[test]
fn editor_v7_row_script_asset_bytes_hash_match_the_compiler_output() {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let scripted = import_unsigned_scripted(
        EDITOR_SCRIPTED_VECTOR_V7,
        &mut factory,
        None,
        FileImportLimits::new(),
        ScriptExecutionLimits::new(),
    )
    .expect("editor-published scripted .riv must import");
    let file = scripted.native_file();
    let script_assets = file.with_file(|file| {
        (0..)
            .map_while(|index| file.asset(index))
            .filter(|asset| {
                asset
                    .with_downcast::<ScriptAsset, _>(|_| true)
                    .unwrap_or(false)
            })
            .collect::<Vec<_>>()
    });
    let [script_asset] = script_assets.as_slice() else {
        panic!(
            "the matrix fixture embeds exactly one ScriptAsset, found {}",
            script_assets.len()
        );
    };
    let bytecode = script_asset
        .with_downcast::<ScriptAsset, _>(|script| script.module_bytecode().to_vec())
        .expect("actual ScriptAsset");
    assert_eq!(
        format!("{:x}", Sha256::digest(&bytecode)),
        EMITTED_BYTECODE_SHA256,
        "re-extracted ScriptAsset bytes must hash-match the compiler output",
    );

    let version = *bytecode.first().expect("bytecode version byte");
    assert_eq!(version, EMITTED_BYTECODE_VERSION);
    // Live guard against pin drift: the row is only evidence while this VM
    // still accepts what the editor emitted.
    let accepted = luaur_common::enums::luau_bytecode_tag::LBC_VERSION_MIN.0 as u8
        ..=luaur_common::enums::luau_bytecode_tag::LBC_VERSION_MAX.0 as u8;
    assert!(
        accepted.contains(&version),
        "emitted version {version} left the VM's accepted range {accepted:?}",
    );
}

#[test]
fn editor_v7_row_script_paints_through_this_runtime() {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let scripted = import_unsigned_scripted(
        EDITOR_SCRIPTED_VECTOR_V7,
        &mut factory,
        None,
        FileImportLimits::new(),
        ScriptExecutionLimits::new(),
    )
    .expect("editor-published scripted .riv must import");
    let file = scripted.native_file();
    let artboard = file
        .with_file(File::artboard_default)
        .expect("published scripted artboard");
    artboard.advance_default(0.0);
    let mut renderer = factory.borrow().make_renderer();
    artboard.draw(&mut renderer);
    let stream = factory.borrow().stream();
    assert!(stream.contains("color=0xff7f33cc"), "{stream}");
    assert!(stream.contains("drawPath "), "{stream}");
    assert!(
        stream.contains("transform matrix=[1,0,0,1,24,18]"),
        "{stream}"
    );
}
