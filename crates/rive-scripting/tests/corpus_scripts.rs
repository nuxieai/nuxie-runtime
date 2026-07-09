//! Reality check against real Rive corpus files: pull `ScriptAsset` payloads
//! out of `.riv` binaries with rive-binary's public reader and prove luaur
//! can load and execute them.
//!
//! Findings these tests pin down:
//! - `.riv` files carry Luau **bytecode** (version 6), not source, wrapped
//!   in the 1-byte signed-content envelope (`rive_scripting::envelope`) —
//!   exactly what C++ `ScriptAsset::decode` + `ScriptingVM::loadModule`
//!   (`luau_load`) consume.
//! - luaur loads that bytecode directly, executes the chunks, resolves the
//!   corpus `require` dependency chains, and runs a real protocol script's
//!   generator end-to-end given two stub globals (`Vector`, `late`).

use rive_binary::read_runtime_file;
use rive_scripting::envelope::SignedContent;

/// Scripting corpus files live in the C++ runtime checkout's unit-test
/// assets; located the same way as rive-binary's tests (`RIVE_RUNTIME_DIR`
/// env var with the workstation default as fallback).
fn asset_path(name: &str) -> std::path::PathBuf {
    std::env::var_os("RIVE_RUNTIME_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("/Users/levi/dev/oss/rive-runtime"))
        .join("tests/unit_tests/assets")
        .join(name)
}

/// A ScriptAsset and its in-band payload extracted from a `.riv` file.
struct ExtractedScript {
    name: String,
    payload: Vec<u8>,
}

/// Walk the object stream pairing each `ScriptAsset` with the
/// `FileAssetContents` that follows it (the importer contract: contents
/// attach to the latest FileAsset, mirroring C++ `FileAssetContentsImporter`).
fn extract_scripts(riv_bytes: &[u8]) -> Vec<ExtractedScript> {
    let file = read_runtime_file(riv_bytes).expect("corpus file parses");
    let mut scripts = Vec::new();
    let mut pending_script: Option<String> = None;

    for id in 0..file.object_count() {
        let Some(object) = file.object(id) else {
            continue;
        };
        match object.type_name {
            "ScriptAsset" => {
                pending_script = Some(
                    object
                        .string_property("name")
                        .unwrap_or("unnamed")
                        .to_string(),
                );
            }
            "FileAssetContents" => {
                if let Some(name) = pending_script.take() {
                    let payload = object
                        .bytes_property("bytes")
                        .expect("in-band script asset has bytes")
                        .to_vec();
                    scripts.push(ExtractedScript { name, payload });
                }
            }
            // Any other asset type between a ScriptAsset and its contents
            // re-targets the importer; drop the pending association.
            _ if object.type_name.ends_with("Asset") => {
                pending_script = None;
            }
            _ => {}
        }
    }
    scripts
}

fn read_corpus(name: &str) -> Vec<u8> {
    let path = asset_path(name);
    std::fs::read(&path).unwrap_or_else(|e| panic!("missing corpus file {}: {e}", path.display()))
}

#[test]
fn corpus_script_assets_carry_luau_bytecode_not_source() {
    let scripts = extract_scripts(&read_corpus("script_artboard_test.riv"));
    assert!(
        !scripts.is_empty(),
        "script_artboard_test.riv should contain ScriptAssets"
    );

    for script in &scripts {
        let envelope = SignedContent::parse(&script.payload).expect("valid envelope");
        // Luau bytecode starts with a version byte in luaur's accepted range
        // (3..=11, target 6); Luau *source* would start with printable text.
        // This is the source-vs-bytecode determination.
        let first = envelope.content[0];
        assert!(
            (3..=11).contains(&first),
            "script '{}' payload does not look like Luau bytecode \
             (first content byte {first:#x})",
            script.name
        );
        eprintln!(
            "script '{}': envelope v{} signed={} content={} bytes, \
             luau bytecode version {first}",
            script.name,
            envelope.version,
            envelope.is_signed(),
            envelope.content.len(),
        );
    }
}

#[cfg(feature = "luau")]
mod luau {
    use super::*;
    use luaur_rt::{Function, Table, Value};
    use rive_runtime::{
        NoopScriptHost, ScriptMethod, ScriptValue, ScriptingVm as RuntimeScriptingVm,
    };
    use rive_scripting::vm::ScriptVm;

    #[test]
    fn luaur_loads_real_corpus_bytecode() {
        let vm = ScriptVm::new();
        let scripts = extract_scripts(&read_corpus("script_artboard_test.riv"));
        assert!(!scripts.is_empty());

        for script in &scripts {
            vm.load_script_asset_payload(&script.name, &script.payload)
                .unwrap_or_else(|e| panic!("luaur failed to load '{}': {e}", script.name));
        }
    }

    #[test]
    fn malformed_bytecode_is_rejected_before_luau_load() {
        let vm = ScriptVm::new();
        for bytecode in [Vec::new(), vec![6], vec![6, 3], vec![6, 3, 0xff]] {
            let error = vm
                .load_bytecode("malformed", &bytecode)
                .expect_err("malformed bytecode should be a regular error");
            assert!(
                error.to_string().contains("malformed Luau bytecode"),
                "unexpected error: {error}"
            );
        }
    }

    #[test]
    fn truncated_corpus_bytecode_is_rejected_before_luau_load() {
        let scripts = extract_scripts(&read_corpus("script_artboard_test.riv"));
        let script = &scripts[0];
        let content = SignedContent::parse(&script.payload).unwrap().content;
        let cuts = [
            0usize,
            1,
            2,
            3,
            content.len() / 2,
            content.len().saturating_sub(1),
        ];
        let vm = ScriptVm::new();

        for cut in cuts {
            let error = vm
                .load_bytecode(&format!("{}-truncated-{cut}", script.name), &content[..cut])
                .expect_err("truncated corpus bytecode should be a regular error");
            assert!(
                error.to_string().contains("malformed Luau bytecode"),
                "unexpected error for cut {cut}: {error}"
            );
        }
    }

    /// Load every ScriptAsset across a sample of the scripting corpus to
    /// measure luaur bytecode compatibility breadth, not just one file.
    #[test]
    fn luaur_loads_bytecode_across_scripting_corpus() {
        let corpus = [
            "script_artboard_test.riv",
            "scripted_graph.riv",
            "script_paths_test.riv",
            "scripted_color.riv",
            "script_inputs_test_1.riv",
            "scripting_linear_animation.riv",
            "script_dependency_test.riv",
        ];
        let vm = ScriptVm::new();
        let mut loaded = 0usize;

        for file_name in corpus {
            for script in extract_scripts(&read_corpus(file_name)) {
                vm.load_script_asset_payload(&script.name, &script.payload)
                    .unwrap_or_else(|e| {
                        panic!(
                            "luaur failed to load '{}' from {file_name}: {e}",
                            script.name
                        )
                    });
                loaded += 1;
            }
        }
        assert!(loaded >= 10, "expected scripts across the corpus sample");
        eprintln!("loaded {loaded} corpus scripts as Luau bytecode");
    }

    /// script_dependency_test.riv stores its modules *out of dependency
    /// order* (ChainedConverter first; it requires Transform1 -> 2 -> 3 ->
    /// 4 -> 5). The C++ runtime handles this with a registration retry loop
    /// (`ScriptingContext::performRegistration`); prove the Rust twin
    /// converges and that in-VM `require` resolves the registered modules.
    #[test]
    fn corpus_require_dependency_chain_registers_and_resolves() {
        let scripts = extract_scripts(&read_corpus("script_dependency_test.riv"));
        assert_eq!(scripts.len(), 6, "expected the 6-script dependency chain");

        let vm = ScriptVm::new();
        let failures = vm.perform_registration(
            scripts
                .iter()
                .map(|s| (s.name.as_str(), s.payload.as_slice())),
        );
        assert!(
            failures.is_empty(),
            "registration did not converge: {:?}",
            failures
                .iter()
                .map(|(n, e)| format!("{n}: {e}"))
                .collect::<Vec<_>>()
        );

        // Utility modules resolve through require; the protocol script's
        // chunk produced its generator function.
        let transform: Value = vm.eval("return require('Transform1')").unwrap();
        assert!(matches!(transform, Value::Table(_)));
        assert!(matches!(
            vm.registered_module("ChainedConverter").unwrap(),
            Value::Function(_)
        ));
    }

    /// Execute a real protocol script end-to-end the way C++
    /// `ScriptedObject::ensureScriptInitialized` does: run the chunk to get
    /// the generator, call `generator(context)`, and get back the instance
    /// table carrying the scripted-drawable lifecycle methods. Only two
    /// host globals are needed at construction time: `Vector` (Rive's
    /// vector constructor) and `late` (Rive's late-bound-input marker,
    /// which just returns nil — `lua_late` in src/lua/rive_lua_libs.cpp).
    #[test]
    fn corpus_protocol_script_generator_runs_end_to_end() {
        let scripts = extract_scripts(&read_corpus("script_artboard_test.riv"));
        let script = &scripts[0];
        assert_eq!(script.name, "ArtboardGrid");

        let vm = ScriptVm::new();
        vm.eval::<()>(
            "function Vector(x, y) return { x = x or 0, y = y or 0 } end\n\
             function late() return nil end",
        )
        .unwrap();

        let generator: Function = vm
            .run_bytecode(
                &script.name,
                SignedContent::parse(&script.payload).unwrap().content,
            )
            .unwrap();

        // C++ passes a ScriptedContext userdata; a stub table suffices for
        // this script's construction path.
        let context: Table = vm.lua().create_table();
        let instance: Table = generator.call(context).unwrap();

        // The instance implements the ScriptedDrawable protocol surface...
        for method in ["init", "advance", "update", "draw", "pointerDown"] {
            assert!(
                matches!(instance.get::<Value>(method).unwrap(), Value::Function(_)),
                "instance should implement {method}"
            );
        }
        // ...and carries its default input values, readable from Rust.
        assert_eq!(instance.get::<f64>("columns").unwrap(), 3.0);
        assert_eq!(instance.get::<f64>("rows").unwrap(), 2.0);

        // Inputs are plain fields on the instance (C++ setNumberInput).
        instance.set("columns", 5.0).unwrap();
        assert_eq!(instance.get::<f64>("columns").unwrap(), 5.0);
    }

    #[test]
    fn runtime_scripting_seam_instantiates_protocol_script() {
        let scripts = extract_scripts(&read_corpus("script_artboard_test.riv"));
        let script = &scripts[0];
        assert_eq!(script.name, "ArtboardGrid");

        let mut vm = ScriptVm::new();
        let mut host = NoopScriptHost;
        let mut instance = RuntimeScriptingVm::instantiate_script(
            &mut vm,
            &script.name,
            &script.payload,
            &mut host,
        )
        .unwrap();

        for method in [
            ScriptMethod::Init,
            ScriptMethod::Advance,
            ScriptMethod::Update,
            ScriptMethod::Draw,
            ScriptMethod::PointerDown,
        ] {
            assert!(
                instance.has_method(method).unwrap(),
                "instance should implement {}",
                method.as_str()
            );
        }

        assert_eq!(
            instance.get_input("columns").unwrap().as_number(),
            Some(3.0)
        );
        instance
            .set_input("columns", ScriptValue::Number(5.0))
            .unwrap();
        assert_eq!(
            instance.get_input("columns").unwrap().as_number(),
            Some(5.0)
        );

        let vector: Value = vm.eval("return Vector(1, 2)").unwrap();
        assert!(
            matches!(vector, Value::Vector(v) if v == [1.0, 2.0, 0.0]),
            "Vector must be luaur's native vector value, got {vector:?}"
        );

        assert_eq!(
            instance
                .call_method(
                    ScriptMethod::Advance,
                    &[ScriptValue::Number(0.016)],
                    &mut host,
                )
                .unwrap(),
            ScriptValue::Bool(false)
        );
    }
}
