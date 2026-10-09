//! Public tests/unit_tests/runtime/script_signature_test.cpp at f40c9dfe.
//! Guest execution and AOT rejection remain parked under UNIV-3728; the
//! non-Wasm import/signature branches are translated here.
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{
    File, RuntimeFactoryHandle,
    source::{
        assets::{
            script_asset::ScriptAsset, script_module_asset::ScriptModuleAsset,
            shader_asset::ShaderAsset,
        },
        core::CoreHandle,
        file::{ImportResult, RuntimeFileHandle},
        lua::scripting_vm::RuntimeScriptingVmHandle,
        scripted::decoded_file::import_decoded_file,
    },
};
use nuxie_scripting::vm::{ScriptExecutionLimits, ScriptVm};

const SAMPLE_KEY_VERIFIES: bool = cfg!(feature = "test-script-signature");
const PRODUCTION_KEY_VERIFIES: bool = !SAMPLE_KEY_VERIFIES;
const UNSIGNED_ACCEPTED: bool = cfg!(feature = "tools");
const UNSIGNED: &str = "animascript.riv";
const SHADER_SIGNED: &str = "animascript_shader_signed.riv";
const LUAU_SHADER: &str = "luau_shader_signed.riv";

fn fixture(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .expect("RIVE_RUNTIME_DIR must name the pinned upstream checkout");
    std::fs::read(
        std::path::PathBuf::from(root)
            .join("tests/unit_tests/assets")
            .join(name),
    )
    .unwrap()
}

fn import(bytes: &[u8], require_signed: bool) -> RuntimeFileHandle {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let factory = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
    let mut result = ImportResult::Malformed;
    // The existing inverted dependency API creates a Luau VM lazily, just as
    // C++ File::registerScripts does. Ordinary imports here need only assets.
    let file = if require_signed {
        let mut make_vm = |_| {
            RuntimeScriptingVmHandle::new(Box::new(
                ScriptVm::new_with_execution_limits(ScriptExecutionLimits::default()).unwrap(),
            ))
        };
        import_decoded_file(bytes, factory, Some(&mut result), Some(&mut make_vm))
    } else {
        File::import_with_script_policy(bytes, factory, Some(&mut result), None, None, false)
    };
    assert_eq!(result, ImportResult::Success);
    file.expect("upstream fixture imports")
}

fn shaders(file: &RuntimeFileHandle) -> Vec<CoreHandle> {
    file.with_file(|file| {
        file.assets()
            .iter()
            .filter(|asset| asset.with_downcast::<ShaderAsset, _>(|_| ()).is_some())
            .cloned()
            .collect()
    })
}
fn scripts(file: &RuntimeFileHandle) -> Vec<CoreHandle> {
    file.with_file(|file| {
        file.assets()
            .iter()
            .filter(|asset| asset.with_downcast::<ScriptAsset, _>(|_| ()).is_some())
            .cloned()
            .collect()
    })
}
fn shader_used(shader: &ShaderAsset) -> bool {
    let exposed = !shader.rstb().is_empty();
    if !exposed {
        assert!(shader.texture_sampler_pairs().is_empty());
        for target in 0..=16 {
            assert!(shader.find_shader(target).is_empty());
        }
    }
    exposed
}
fn shaders_used(file: &RuntimeFileHandle, used: bool) {
    let shaders = shaders(file);
    assert!(!shaders.is_empty());
    for shader in shaders {
        shader
            .with_downcast::<ShaderAsset, _>(|shader| assert_eq!(shader_used(shader), used))
            .unwrap();
    }
}
fn find(bytes: &[u8], needle: &[u8]) -> usize {
    bytes
        .windows(needle.len())
        .position(|candidate| candidate == needle)
        .expect("upstream fixture marker")
}
fn signature_at(bytes: &[u8]) -> usize {
    assert_eq!(nuxie_runtime::source::generated::assets::file_asset_contents_base::FileAssetContentsBase::SIGNATURE_PROPERTY_KEY, 0x38f);
    let at = find(bytes, &[0x8f, 7, 64]);
    assert_eq!(bytes[at + 3 + 64], 0);
    at
}
fn luau_runs(file: &RuntimeFileHandle) -> bool {
    file.with_file(|file| file.scripting_vm().is_some())
}

#[test]
fn an_unsigned_animascript_module_imports_without_joining_a_signed_group() {
    for path in [UNSIGNED, SHADER_SIGNED] {
        for require_signed in [false, true] {
            let file = import(&fixture(path), require_signed);
            let count = file.with_file(|file| {
                file.assets()
                    .iter()
                    .filter(|asset| {
                        asset
                            .with_downcast::<ScriptModuleAsset, _>(|_| ())
                            .is_some()
                    })
                    .count()
            });
            assert_eq!(count, 1);
            // Upstream checkModuleRuns' VM checks are WITH_RIVE_SCRIPTING_WASM
            // only. This assertion proves import, not guest execution.
        }
    }
}

#[test]
fn the_signature_beside_an_animascript_module_covers_the_shader_alone() {
    let file = import(&fixture(SHADER_SIGNED), false);
    let shader_assets = shaders(&file);
    assert_eq!(shader_assets.len(), 1);
    shader_assets[0]
        .with_downcast::<ShaderAsset, _>(|shader| {
            assert_eq!(shader.base.verified(), SAMPLE_KEY_VERIFIES)
        })
        .unwrap();
    for script in scripts(&file) {
        script
            .with_downcast::<ScriptAsset, _>(|script| assert!(!script.verified()))
            .unwrap();
    }
    shaders_used(&file, UNSIGNED_ACCEPTED || SAMPLE_KEY_VERIFIES);
    shaders_used(&import(&fixture(SHADER_SIGNED), true), false);
}

#[test]
fn a_signed_files_shaders_are_used() {
    shaders_used(
        &import(&fixture(LUAU_SHADER), false),
        UNSIGNED_ACCEPTED || PRODUCTION_KEY_VERIFIES,
    );
    shaders_used(
        &import(&fixture(LUAU_SHADER), true),
        PRODUCTION_KEY_VERIFIES,
    );
}

#[test]
fn a_tampered_shader_is_refused_with_the_luau_it_was_signed_with() {
    for path in [SHADER_SIGNED, LUAU_SHADER] {
        let mut bytes = fixture(path);
        let at = find(&bytes, b"VertexOut");
        bytes[at] ^= 1;
        for required in [false, true] {
            let file = import(&bytes, required);
            for shader in shaders(&file) {
                shader
                    .with_downcast::<ShaderAsset, _>(|shader| assert!(!shader.base.verified()))
                    .unwrap();
            }
            shaders_used(&file, UNSIGNED_ACCEPTED && !required);
            for script in scripts(&file) {
                script
                    .with_downcast::<ScriptAsset, _>(|script| assert!(!script.verified()))
                    .unwrap();
            }
            if path == LUAU_SHADER && required {
                assert!(!luau_runs(&file));
            }
        }
    }
}

#[test]
fn a_shader_without_its_signature_is_refused_unless_unsigned_content_is_accepted() {
    for path in [SHADER_SIGNED, LUAU_SHADER] {
        let mut bytes = fixture(path);
        let at = signature_at(&bytes);
        bytes.drain(at..at + 3 + 64);
        for required in [false, true] {
            let file = import(&bytes, required);
            for shader in shaders(&file) {
                shader
                    .with_downcast::<ShaderAsset, _>(|shader| assert!(!shader.base.verified()))
                    .unwrap();
            }
            shaders_used(&file, UNSIGNED_ACCEPTED && !required);
        }
    }
}

#[test]
fn a_referenced_shader_verifies_its_own_signature() {
    let bytes = fixture(SHADER_SIGNED);
    let file = import(&bytes, false);
    let shader_assets = shaders(&file);
    assert_eq!(shader_assets.len(), 1);
    let rstb = shader_assets[0]
        .with_downcast::<ShaderAsset, _>(|shader| shader.rstb().to_vec())
        .unwrap();
    // Like upstream, this fixture construction requires tools or the sample key.
    assert!(!rstb.is_empty());
    let at = signature_at(&bytes) + 3;
    let mut envelope = vec![0x80];
    envelope.extend_from_slice(&bytes[at..at + 64]);
    envelope.extend_from_slice(&rstb);
    let mut tampered = envelope.clone();
    *tampered.last_mut().unwrap() ^= 1;
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let factory = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
    for required in [false, true] {
        let mut referenced = ShaderAsset::default();
        referenced.imported_with(required);
        assert!(referenced.decode(&envelope, &factory));
        assert_eq!(referenced.base.verified(), SAMPLE_KEY_VERIFIES);
        assert_eq!(
            shader_used(&referenced),
            !required && (UNSIGNED_ACCEPTED || SAMPLE_KEY_VERIFIES)
        );
        let mut forged = ShaderAsset::default();
        forged.imported_with(required);
        assert!(forged.decode(&tampered, &factory));
        assert!(!forged.base.verified());
        assert_eq!(shader_used(&forged), !required && UNSIGNED_ACCEPTED);
    }
}

#[test]
fn a_luau_and_shader_file_signed_together_verifies_its_luau() {
    let file = import(&fixture(LUAU_SHADER), false);
    let script_assets = scripts(&file);
    assert!(!script_assets.is_empty());
    for script in script_assets {
        script
            .with_downcast::<ScriptAsset, _>(|script| {
                assert_eq!(script.verified(), PRODUCTION_KEY_VERIFIES)
            })
            .unwrap();
    }
    for shader in shaders(&file) {
        shader
            .with_downcast::<ShaderAsset, _>(|shader| {
                assert_eq!(shader.base.verified(), PRODUCTION_KEY_VERIFIES)
            })
            .unwrap();
    }
    assert_eq!(
        luau_runs(&import(&fixture(LUAU_SHADER), true)),
        PRODUCTION_KEY_VERIFIES
    );
}

#[test]
fn a_tampered_luau_script_is_refused() {
    let mut bytes = fixture(LUAU_SHADER);
    let at = find(&bytes, b"add WGSL assets");
    bytes[at] ^= 1;
    let file = import(&bytes, true);
    for script in scripts(&file) {
        script
            .with_downcast::<ScriptAsset, _>(|script| assert!(!script.verified()))
            .unwrap();
    }
    shaders_used(&file, false);
    assert!(!luau_runs(&file));
}

#[test]
fn a_refused_shader_is_reported_once_its_file_is_read_never_on_reads() {
    const CHILD: &str = "NUXIE_SIGNATURE_LOG_CHILD";
    const MARKER: &str = "NUXIE_SIGNATURE_ACCESSORS_BEGIN";
    const REFUSAL: &str = "is unavailable: its signature did not verify";
    if std::env::var_os(CHILD).is_some() {
        let mut bytes = fixture(LUAU_SHADER);
        let at = find(&bytes, b"VertexOut");
        bytes[at] ^= 1;
        let file = import(&bytes, true);
        eprintln!("{MARKER}:{}", shaders(&file).len());
        shaders_used(&file, false);
        return;
    }
    // Isolated process captures actual stderr without changing process-wide
    // descriptors while unrelated Rust tests run in parallel.
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "a_refused_shader_is_reported_once_its_file_is_read_never_on_reads",
            "--nocapture",
        ])
        .env(CHILD, "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8(output.stderr).unwrap();
    let (imported, reads) = stderr.split_once(MARKER).unwrap();
    let count: usize = reads
        .trim_start_matches(':')
        .lines()
        .next()
        .unwrap()
        .parse()
        .unwrap();
    assert_eq!(imported.matches(REFUSAL).count(), count);
    assert_eq!(reads.matches(REFUSAL).count(), 0);
}
