// Preserved assertions from the previous public facade. These remain test-wired
// while consumers are migrated to the upstream factory-required API.
use crate::*;
use anyhow::Result;
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, RwLock},
};

#[cfg(all(test, feature = "scripting"))]
mod inert_script_import_tests {
    use super::*;
    use nuxie_render_api::{PersistentFactory, RecordingFactory};
    use nuxie_runtime::mechanical_port::source::assets::script_asset::ScriptAsset;
    use nuxie_schema::definition_by_name;

    fn push_var_uint(bytes: &mut Vec<u8>, mut value: u64) {
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            bytes.push(byte);
            if value == 0 {
                break;
            }
        }
    }

    fn property_key(type_name: &str, property_name: &str) -> u16 {
        let definition = definition_by_name(type_name).expect("fixture type exists");
        definition
            .properties
            .iter()
            .chain(definition.ancestors.iter().flat_map(|ancestor| {
                definition_by_name(ancestor)
                    .expect("fixture ancestor exists")
                    .properties
                    .iter()
            }))
            .find(|property| property.name == property_name)
            .expect("fixture property exists")
            .key
            .int
    }

    fn push_object(bytes: &mut Vec<u8>, type_name: &str, properties: impl FnOnce(&mut Vec<u8>)) {
        push_var_uint(
            bytes,
            u64::from(
                definition_by_name(type_name)
                    .expect("fixture type exists")
                    .type_key
                    .int,
            ),
        );
        properties(bytes);
        push_var_uint(bytes, 0);
    }

    fn push_uint(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: u64) {
        push_var_uint(bytes, u64::from(property_key(type_name, name)));
        push_var_uint(bytes, value);
    }

    fn push_blob(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: &[u8]) {
        push_var_uint(bytes, u64::from(property_key(type_name, name)));
        push_var_uint(bytes, value.len() as u64);
        bytes.extend_from_slice(value);
    }

    fn imported_script_assets_bytes(payloads: &[&[u8]]) -> Vec<u8> {
        let mut bytes = b"RIVE".to_vec();
        push_var_uint(&mut bytes, 7);
        push_var_uint(&mut bytes, 0);
        push_var_uint(&mut bytes, 991);
        push_var_uint(&mut bytes, 0);
        push_object(&mut bytes, "Backboard", |_| {});
        for (ordinal, payload) in payloads.iter().enumerate() {
            push_object(&mut bytes, "ScriptAsset", |bytes| {
                push_uint(bytes, "ScriptAsset", "assetId", ordinal as u64);
            });
            push_object(&mut bytes, "FileAssetContents", |bytes| {
                push_blob(bytes, "FileAssetContents", "bytes", payload);
            });
        }
        bytes
    }

    fn imported_script_asset_bytes() -> Vec<u8> {
        imported_script_assets_bytes(&[&[0, 1, 2, 3]])
    }

    #[cfg(feature = "ore-metal-authored-msl")]
    fn imported_shader_asset_bytes(payload: &[u8]) -> Vec<u8> {
        let mut bytes = b"RIVE".to_vec();
        push_var_uint(&mut bytes, 7);
        push_var_uint(&mut bytes, 0);
        push_var_uint(&mut bytes, 991);
        push_var_uint(&mut bytes, 0);
        push_object(&mut bytes, "Backboard", |_| {});
        push_object(&mut bytes, "ShaderAsset", |bytes| {
            push_uint(bytes, "ShaderAsset", "assetId", 0);
        });
        push_object(&mut bytes, "FileAssetContents", |bytes| {
            push_blob(bytes, "FileAssetContents", "bytes", payload);
        });
        bytes
    }

    fn external_fixture(relative: &str) -> Vec<u8> {
        let path = std::path::PathBuf::from(
            std::env::var_os("RIVE_RUNTIME_DIR")
                .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into()),
        )
        .join("tests/unit_tests/assets")
        .join(relative);
        std::fs::read(path).expect("read external fixture")
    }

    fn import_native_with_limits(
        bytes: &[u8],
        limits: FileImportLimits,
    ) -> Result<nuxie_runtime::mechanical_port::source::file::RuntimeFileHandle> {
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        import_native(bytes, &mut factory, None, limits)
    }

    fn imported_image_asset_bytes(count: usize) -> Vec<u8> {
        let mut bytes = b"RIVE".to_vec();
        push_var_uint(&mut bytes, 7);
        push_var_uint(&mut bytes, 0);
        push_var_uint(&mut bytes, 992);
        push_var_uint(&mut bytes, 0);
        push_object(&mut bytes, "Backboard", |_| {});
        for asset_id in 0..count {
            push_object(&mut bytes, "ImageAsset", |bytes| {
                push_uint(bytes, "ImageAsset", "assetId", asset_id as u64);
            });
        }
        bytes
    }

    fn imported_manifest_asset_bytes() -> Vec<u8> {
        // One name entry: section=0, section bytes=[count=1, id=7,
        // string-length=1, 'a']. The parser budget charges the ManifestAsset
        // and FileAssetContents properties plus this declared entry.
        let manifest = [0, 4, 1, 7, 1, b'a'];
        let mut bytes = b"RIVE".to_vec();
        push_var_uint(&mut bytes, 7);
        push_var_uint(&mut bytes, 0);
        push_var_uint(&mut bytes, 993);
        push_var_uint(&mut bytes, 0);
        push_object(&mut bytes, "Backboard", |_| {});
        push_object(&mut bytes, "ManifestAsset", |bytes| {
            push_uint(bytes, "ManifestAsset", "assetId", 0);
        });
        push_object(&mut bytes, "FileAssetContents", |bytes| {
            push_blob(bytes, "FileAssetContents", "bytes", &manifest);
        });
        bytes
    }

    #[test]
    fn bounded_import_rejects_file_assets_before_owned_graph_construction() {
        let bytes = imported_image_asset_bytes(2);
        let limits = FileImportLimits::new().with_max_imported_file_assets(1);

        let error = import_native_with_limits(&bytes, limits)
            .err()
            .expect("the parsed file exceeds its pre-graph asset limit");
        assert!(
            error.to_string().contains("imports more than 1 FileAssets"),
            "{error:#}"
        );
        import_native_with_limits(
            &bytes,
            FileImportLimits::new().with_max_imported_file_assets(2),
        )
        .expect("the exact bound admits graph construction");
    }

    #[test]
    fn bounded_import_rejects_input_before_binary_parser_allocation() {
        let bytes = imported_script_asset_bytes();
        let error = import_native_with_limits(
            &bytes,
            FileImportLimits::new().with_max_input_bytes(bytes.len() - 1),
        )
        .err()
        .expect("an oversized input must be rejected before parsing");
        assert!(error.to_string().contains("import limit"), "{error:#}");

        import_native_with_limits(
            &bytes,
            FileImportLimits::new().with_max_input_bytes(bytes.len()),
        )
        .expect("the exact input-byte bound admits parsing");
    }

    #[test]
    fn bounded_import_rejects_runtime_object_and_asset_content_growth() {
        let bytes = imported_script_asset_bytes();

        let object_error =
            import_native_with_limits(&bytes, FileImportLimits::new().with_max_runtime_objects(2))
                .err()
                .expect("the fixture has three runtime objects");
        assert!(
            format!("{object_error:#}").contains("runtime objects"),
            "{object_error:#}"
        );

        let per_content_error = import_native_with_limits(
            &bytes,
            FileImportLimits::new().with_max_file_asset_content_bytes(3),
        )
        .err()
        .expect("the script payload is four bytes");
        assert!(
            per_content_error
                .to_string()
                .contains("per-content import limit"),
            "{per_content_error:#}"
        );

        let aggregate_error = import_native_with_limits(
            &bytes,
            FileImportLimits::new().with_max_total_file_asset_content_bytes(3),
        )
        .err()
        .expect("the aggregate payload is four bytes");
        assert!(
            aggregate_error
                .to_string()
                .contains("aggregate content bytes"),
            "{aggregate_error:#}"
        );
    }

    #[test]
    fn native_object_limit_precedes_next_record_decode_and_unbounded_stays_available() {
        let mut bytes = b"RIVE".to_vec();
        push_var_uint(&mut bytes, 7);
        push_var_uint(&mut bytes, 0);
        push_var_uint(&mut bytes, 991);
        push_var_uint(&mut bytes, 0);
        push_object(&mut bytes, "Backboard", |_| {});
        bytes.push(0x80);

        let bounded =
            import_native_with_limits(&bytes, FileImportLimits::new().with_max_runtime_objects(1))
                .err()
                .expect("the parser must reject before decoding compact object two");
        assert!(
            format!("{bounded:#}").contains("more than 1 runtime objects"),
            "{bounded:#}"
        );

        let unbounded = import_native_with_limits(&bytes, FileImportLimits::unbounded())
            .err()
            .expect("the explicit unbounded reader reaches the malformed second record");
        assert!(
            !format!("{unbounded:#}").contains("more than 1 runtime objects"),
            "{unbounded:#}"
        );
    }

    #[test]
    fn native_property_limit_covers_values_and_manifest_declared_work() {
        let node_type = definition_by_name("Node")
            .expect("Node schema")
            .type_key
            .int;
        let x_key = property_key("Node", "x");
        let mut malformed_value = b"RIVE".to_vec();
        push_var_uint(&mut malformed_value, 7);
        push_var_uint(&mut malformed_value, 0);
        push_var_uint(&mut malformed_value, 994);
        push_var_uint(&mut malformed_value, 0);
        push_object(&mut malformed_value, "Backboard", |_| {});
        push_var_uint(&mut malformed_value, u64::from(node_type));
        for _ in 0..2 {
            push_var_uint(&mut malformed_value, u64::from(x_key));
            malformed_value.extend_from_slice(&1.0f32.to_le_bytes());
        }
        push_var_uint(&mut malformed_value, u64::from(x_key));

        let error = import_native_with_limits(
            &malformed_value,
            FileImportLimits::new().with_max_runtime_properties(2),
        )
        .err()
        .expect("property N+1 must be rejected before its missing value is decoded");
        assert!(
            format!("{error:#}").contains("runtime object properties"),
            "{error:#}"
        );

        let manifest = imported_manifest_asset_bytes();
        import_native_with_limits(
            &manifest,
            FileImportLimits::new().with_max_runtime_properties(3),
        )
        .expect("two object properties plus one manifest name fit the exact boundary");
        let error = import_native_with_limits(
            &manifest,
            FileImportLimits::new().with_max_runtime_properties(2),
        )
        .err()
        .expect("manifest declarations share the native import property budget");
        assert!(
            format!("{error:#}").contains("manifest name entries"),
            "{error:#}"
        );
    }

    #[test]
    fn ordinary_native_import_keeps_scripts_inert_and_retains_shared_owners() {
        let bytes = imported_script_asset_bytes();
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let inert = import_native(&bytes, &mut factory, None, FileImportLimits::new())
            .expect("ordinary import remains available");
        let inert_asset = inert.with_file(|file| {
            assert!(file.scripting_vm().is_none());
            assert_eq!(file.assets().len(), 1);
            file.assets()[0].clone()
        });
        inert_asset
            .with_downcast::<ScriptAsset, _>(|script| {
                assert!(!script.verified());
                assert!(script.scripting_vm().is_none());
            })
            .expect("fixture owns one ScriptAsset");
        let cloned = inert.clone();
        assert_eq!(
            inert_asset,
            cloned.with_file(|file| file.assets()[0].clone()),
            "cloning the native File handle retains the same ScriptAsset owner"
        );

        let trusted = import_unsigned_scripted(
            &bytes,
            &mut factory,
            None,
            FileImportLimits::new(),
            ScriptExecutionLimits::new(),
        )
        .expect("explicit trust admits the exact script bytes");
        trusted.native_file().with_file(|file| {
            file.assets()[0]
                .with_downcast::<ScriptAsset, _>(|script| {
                    assert!(script.verified());
                    assert!(script.scripting_vm().is_some());
                    assert!(
                        !script.has_generator(),
                        "malformed bytecode must not install a protocol generator"
                    );
                })
                .expect("trusted fixture remains a ScriptAsset");
        });
    }

    #[cfg(feature = "ore-metal-authored-msl")]
    #[test]
    fn native_shader_authority_is_bound_to_the_exact_imported_artifact() {
        let shader_payload = [0, 1, 2, 3];
        let bytes = imported_shader_asset_bytes(&shader_payload);
        let config = HostCommandImportConfig::new(
            "bridge",
            ScriptExecutionLimits::new(),
            HostCommandLimits::new(),
        )
        .expect("host command config");

        // SAFETY: this synthetic fixture stands in for output from the trusted
        // native-shader exporter; the test never submits its payload to Metal.
        let capability = unsafe {
            ScriptExecutionCapability::for_verified_native_shader_artifact_unchecked(
                &bytes,
                config.extension(),
            )
            .expect("test native-shader authority")
        };
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let trusted = import_scripted(
            &bytes,
            &mut factory,
            None,
            FileImportLimits::new(),
            capability.clone(),
            ScriptExecutionLimits::new(),
            None,
        )
        .expect("trusted-exporter exact-artifact boundary imports");
        assert!(
            trusted
                .native_file()
                .with_file(|file| file.scripting_vm().is_some())
        );

        let mut changed = bytes.clone();
        changed.push(0);
        let error = import_scripted(
            &changed,
            &mut factory,
            None,
            FileImportLimits::new(),
            capability,
            ScriptExecutionLimits::new(),
            None,
        )
        .err()
        .expect("authority for one artifact must not authorize changed bytes");
        assert!(
            format!("{error:#}").contains("does not match the exact artifact bytes"),
            "{error:#}"
        );
    }

    #[test]
    fn import_log_sink_observes_script_registration_failure() {
        let bytes = imported_script_asset_bytes();
        let lines = Arc::new(RwLock::new(Vec::new()));
        let captured = Arc::clone(&lines);
        let config = HostCommandImportConfig::new(
            "bridge",
            ScriptExecutionLimits::new(),
            HostCommandLimits::new(),
        )
        .expect("host command config");
        // SAFETY: this test admits only its exact synthetic fixture bytes.
        let capability = unsafe {
            ScriptExecutionCapability::for_verified_artifact_unchecked(&bytes, config.extension())
                .expect("test authority")
        };
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let _scripted = import_scripted(
            &bytes,
            &mut factory,
            None,
            FileImportLimits::new(),
            capability,
            ScriptExecutionLimits::new(),
            Some(Arc::new(move |level, line| {
                captured.write().unwrap().push((level, line.to_vec()));
            })),
        )
        .expect("a script registration failure does not discard the imported native File");

        let lines = lines.read().unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].0, ScriptingLogLevel::Error);
        let line = String::from_utf8_lossy(&lines[0].1);
        assert!(
            line.contains("malformed Luau bytecode") && line.contains("outside supported range"),
            "unexpected host log line: {line}"
        );
    }

    #[test]
    fn upstream_script_ownership_fixtures_remain_on_the_luau_vm_path() {
        let root = std::path::PathBuf::from(
            std::env::var_os("RIVE_RUNTIME_DIR")
                .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into()),
        )
        .join("tests/unit_tests/assets");

        for fixture in [
            "script_inputs_test_1.riv",
            "script_string_converter_test.riv",
            "scripted_data_converter_bound_input.riv",
            "scripted_memory_leak.riv",
        ] {
            let bytes = std::fs::read(root.join(fixture))
                .unwrap_or_else(|error| panic!("failed to read {fixture}: {error}"));
            let mut factory = PersistentFactory::new(RecordingFactory::new());
            let scripted = import_unsigned_scripted(
                &bytes,
                &mut factory,
                None,
                FileImportLimits::new(),
                ScriptExecutionLimits::new(),
            )
            .unwrap_or_else(|error| panic!("failed to import {fixture}: {error:#}"));
            let protocols = scripted
                .native_file()
                .with_file(|file| file.assets().to_vec())
                .iter()
                .filter_map(|asset| {
                    asset.with_downcast::<ScriptAsset, _>(|script| {
                        script.is_protocol_script().then(|| {
                            assert!(script.verified(), "{fixture} protocol was not admitted");
                            assert!(
                                script.scripting_vm().is_some(),
                                "{fixture} protocol does not retain the file VM"
                            );
                            assert!(
                                script.has_generator(),
                                "{fixture} protocol did not register its Luau generator"
                            );
                        })
                    })?
                })
                .collect::<Vec<_>>();
            assert!(
                !protocols.is_empty(),
                "{fixture} has no protocol ScriptAsset fixture witness"
            );
        }
    }

    #[test]
    fn scripted_import_installs_one_runtime_before_native_occurrences_are_created() {
        let bytes = external_fixture("script_inputs_test_1.riv");
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let scripted = import_unsigned_scripted(
            &bytes,
            &mut factory,
            None,
            FileImportLimits::new(),
            ScriptExecutionLimits::new(),
        )
        .expect("trusted script fixture imports");
        let native = scripted.native_file();
        assert!(native.with_file(|file| file.scripting_vm().is_some()));
        let script_assets = native
            .with_file(|file| file.assets().to_vec())
            .into_iter()
            .filter(|asset| {
                asset
                    .with_downcast::<ScriptAsset, _>(|script| {
                        assert!(script.verified());
                        assert!(script.scripting_vm().is_some());
                        true
                    })
                    .unwrap_or(false)
            })
            .collect::<Vec<_>>();
        assert!(!script_assets.is_empty());

        let artboard = native
            .with_file(File::artboard_default)
            .expect("script fixture has a default artboard");
        artboard.advance_default(0.0);

        let cloned_native = native.clone();
        assert_eq!(
            script_assets[0],
            cloned_native.with_file(|file| {
                file.assets()
                    .iter()
                    .find(|asset| asset.with_downcast::<ScriptAsset, _>(|_| ()).is_some())
                    .expect("cloned handle sees ScriptAsset")
                    .clone()
            }),
            "native File clones retain the installed VM's source owners"
        );
    }

    #[test]
    fn native_import_leaves_unauthenticated_scripts_inert() {
        let bytes = external_fixture("script_inputs_test_1.riv");
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = import_native(&bytes, &mut factory, None, FileImportLimits::new())
            .expect("visual-only script fixture imports");
        assert!(file.with_file(|file| file.scripting_vm().is_none()));
        let scripts = file
            .with_file(|file| file.assets().to_vec())
            .into_iter()
            .filter_map(|asset| {
                asset.with_downcast::<ScriptAsset, _>(|script| {
                    assert!(script.scripting_vm().is_none());
                    assert!(!script.has_generator());
                })
            })
            .count();
        assert!(scripts > 0, "fixture must witness inert ScriptAssets");
    }

    #[test]
    fn native_import_leaves_a_scriptless_file_without_a_vm() {
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = import_native(
            &external_fixture("dependency_test.riv"),
            &mut factory,
            None,
            FileImportLimits::new(),
        )
        .expect("scriptless fixture imports");
        assert!(file.with_file(|file| file.scripting_vm().is_none()));
        assert!(
            file.with_file(|file| file
                .assets()
                .iter()
                .all(|asset| { asset.with_downcast::<ScriptAsset, _>(|_| ()).is_none() })),
            "scriptless fixture unexpectedly owns a ScriptAsset"
        );
    }
}

#[cfg(test)]
mod owned_instance_tests {
    use super::*;
    use nuxie_render_api::{PersistentFactory, RecordingFactory};
    use nuxie_runtime::source::{
        assets::{font_asset::FontAsset, image_asset::ImageAsset},
        file::ImportResult,
        text::font_hb::HbFont,
        viewmodel::{
            viewmodel_instance::ViewModelInstance,
            viewmodel_instance_asset_font::ViewModelInstanceAssetFont,
        },
    };
    use nuxie_runtime::{FileAssetLoader, FileAssetLoaderRef, RuntimeFactoryHandle};

    const FIXTURE: &[u8] = include_bytes!("../../../fixtures/graph/dependency_test.riv");

    fn external_fixture(relative: &str) -> Vec<u8> {
        let path = std::path::PathBuf::from(
            std::env::var_os("RIVE_RUNTIME_DIR")
                .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into()),
        )
        .join("tests/unit_tests/assets")
        .join(relative);
        std::fs::read(path).expect("read external fixture")
    }

    fn import_native(
        bytes: &[u8],
        loader: Option<FileAssetLoaderRef>,
    ) -> nuxie_runtime::RuntimeFileHandle {
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let retained = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
        let mut result = ImportResult::Malformed;
        let file = File::import(bytes, retained, Some(&mut result), loader, None)
            .expect("native File imports");
        assert_eq!(result, ImportResult::Success);
        file
    }

    type LoaderAttempt = (bool, String, usize);

    struct InBandLoader {
        claim: bool,
        attempted: Rc<RefCell<Option<LoaderAttempt>>>,
    }

    impl FileAssetLoader for InBandLoader {
        fn load_contents(
            &mut self,
            asset: CoreHandle,
            in_band: &[u8],
            _factory: &RuntimeFactoryHandle,
        ) -> bool {
            let metadata = asset
                .with(|object| {
                    let image = object
                        .as_any()
                        .downcast_ref::<ImageAsset>()
                        .expect("ImageAsset");
                    let asset = object.as_file_asset().expect("FileAsset").file_asset_base();
                    (asset.unique_filename(image.file_extension()), in_band.len())
                })
                .expect("retained ImageAsset");
            *self.attempted.borrow_mut() = Some((
                asset.is_type_of(<ImageAsset as nuxie_runtime::source::core::CoreType>::TYPE_KEY),
                metadata.0,
                metadata.1,
            ));
            self.claim
        }
    }

    #[test]
    fn general_loader_claims_in_band_image_and_suppresses_fallback() {
        let attempted = Rc::new(RefCell::new(None));
        let file = import_native(
            &external_fixture("in_band_asset.riv"),
            Some(FileAssetLoaderRef::new(Box::new(InBandLoader {
                claim: true,
                attempted: attempted.clone(),
            }))),
        );
        let image = file.with_file(|file| file.asset(0)).expect("ImageAsset");

        assert_eq!(
            *attempted.borrow(),
            Some((true, "1x1-45022.png".to_owned(), 308))
        );
        assert!(
            image
                .with_downcast::<ImageAsset, _>(|image| image.render_image().is_none())
                .unwrap_or(false),
            "a claiming loader suppresses in-band image decode"
        );
    }

    #[test]
    fn general_loader_rejection_decodes_in_band_image_fallback() {
        let attempted = Rc::new(RefCell::new(None));
        let file = import_native(
            &external_fixture("in_band_asset.riv"),
            Some(FileAssetLoaderRef::new(Box::new(InBandLoader {
                claim: false,
                attempted,
            }))),
        );
        let image = file.with_file(|file| file.asset(0)).expect("ImageAsset");
        assert!(
            image
                .with_downcast::<ImageAsset, _>(|image| image.render_image().is_some())
                .unwrap_or(false)
        );
    }

    #[test]
    fn hosted_image_cdn_descriptor_matches_pinned_cpp() {
        let file = import_native(&external_fixture("hosted_image_file.riv"), None);
        let assets = file.with_file(|file| file.assets().to_vec());
        assert_eq!(assets.len(), 1);
        assets[0]
            .with(|object| {
                let image = object
                    .as_any()
                    .downcast_ref::<ImageAsset>()
                    .expect("ImageAsset");
                let asset = object.as_file_asset().expect("FileAsset").file_asset_base();
                assert_eq!(asset.cdn_uuid().len(), 16);
                assert_eq!(asset.cdn_uuid_str(), "edcb1816-8405-4983-acd2-16db48d85df4");
                assert_eq!(asset.cdn_base_url(), "https://public.uat.rive.app/cdn/uuid");
                assert_eq!(
                    asset.unique_filename(image.file_extension()),
                    "one-45008.png"
                );
                assert_eq!(image.file_extension(), "png");
            })
            .expect("retained ImageAsset");
    }

    #[test]
    fn hosted_font_cdn_descriptor_matches_pinned_cpp() {
        let file = import_native(&external_fixture("hosted_font_file.riv"), None);
        let assets = file.with_file(|file| file.assets().to_vec());
        assert_eq!(assets.len(), 1);
        assets[0]
            .with(|object| {
                let font = object
                    .as_any()
                    .downcast_ref::<FontAsset>()
                    .expect("FontAsset");
                let asset = object.as_file_asset().expect("FileAsset").file_asset_base();
                assert_eq!(asset.cdn_uuid().len(), 16);
                assert_eq!(asset.cdn_base_url(), "https://public.uat.rive.app/cdn/uuid");
                assert_eq!(
                    asset.unique_filename(font.file_extension()),
                    "Inter-43276.ttf"
                );
                assert_eq!(font.file_extension(), "ttf");
            })
            .expect("retained FontAsset");
    }

    #[test]
    fn upstream_file_artboards_can_be_counted_and_accessed_via_index_or_name() {
        let file = import_native(&external_fixture("dependency_test.riv"), None);

        assert_eq!(file.with_file(File::artboard_count), 1);
        assert!(file.with_file(|file| file.artboard_at_source(0)).is_some());
        assert!(
            file.with_file(|file| file.artboard_named_source("Blue"))
                .is_some()
        );
    }

    #[test]
    fn upstream_file_can_be_read() {
        let file = import_native(&external_fixture("two_artboards.riv"), None);

        assert_eq!(
            file.with_file(File::artboard).and_then(|artboard| {
                artboard.with_downcast::<Artboard, _>(|artboard| artboard.base.name().to_owned())
            }),
            Some("Two".to_owned())
        );
        assert!(
            file.with_file(|file| file.artboard_named_source("One"))
                .is_some()
        );
    }

    #[test]
    fn upstream_file_with_bad_blend_mode_fails_to_load() {
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let mut result = ImportResult::Success;
        let file = File::import(
            &external_fixture("solar-system.riv"),
            RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory"),
            Some(&mut result),
            None,
            None,
        );
        assert!(file.is_none());
        assert_eq!(result, ImportResult::Malformed);
    }

    #[test]
    fn data_bind_font_fixture_applies_a_live_font_on_advance_and_draw() {
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = File::import(
            &external_fixture("data_bind_font_test.riv"),
            RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory"),
            None,
            None,
            None,
        )
        .expect("data-bind font fixture imports");
        let artboard = file
            .with_file(File::artboard_default)
            .expect("default artboard");
        let machine = artboard.state_machine_at(0).expect("fixture state machine");
        let view_model = file
            .with_file_mut(|file| {
                file.create_default_view_model_instance_for_artboard(artboard.core_handle())
            })
            .expect("default view model");
        let property = view_model
            .with_downcast::<ViewModelInstance, _>(|instance| {
                instance.property_value_named("fontProperty")
            })
            .flatten()
            .expect("fontProperty");
        let font = HbFont::decode(&external_fixture("kablammo.ttf")).expect("kablammo decoded");
        property
            .with_downcast_mut::<ViewModelInstanceAssetFont, _>(|property| {
                property.set_value(Some(font))
            })
            .expect("native font property");
        machine.with_instance_mut(|machine| machine.bind_view_model_instance(view_model));
        machine.advance_and_apply(0.0);
        let mut renderer = factory.borrow().make_renderer();
        artboard.draw(&mut renderer);
        assert!(factory.borrow().stream().contains("drawPath"));
    }

    #[test]
    fn font_data_bind_stores_replaces_and_clears_its_private_font() {
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = File::import(
            &external_fixture("data_bind_font_test.riv"),
            RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory"),
            None,
            None,
            None,
        )
        .expect("data-bind font fixture imports");
        let artboard = file
            .with_file(File::artboard_default)
            .expect("default artboard");
        let machine = artboard.state_machine_at(0).expect("fixture state machine");
        let view_model = file
            .with_file_mut(|file| {
                file.create_default_view_model_instance_for_artboard(artboard.core_handle())
            })
            .expect("default view model");
        machine.with_instance_mut(|machine| machine.bind_view_model_instance(view_model.clone()));
        machine.advance_and_apply(0.0);
        let property = view_model
            .with_downcast::<ViewModelInstance, _>(|instance| {
                instance.property_value_named("fontProperty")
            })
            .flatten()
            .expect("fontProperty");

        let kablammo = HbFont::decode(&external_fixture("kablammo.ttf")).expect("kablammo decoded");
        property
            .with_downcast_mut::<ViewModelInstanceAssetFont, _>(|property| {
                property.set_value(Some(kablammo.clone()))
            })
            .expect("native font property");
        machine.advance_and_apply(0.0);
        let installed = property
            .with_downcast::<ViewModelInstanceAssetFont, _>(|property| property.asset().font())
            .flatten()
            .expect("backing FontAsset retains kablammo");
        assert!(Rc::ptr_eq(&installed, &kablammo));

        let nabla = HbFont::decode(&external_fixture("nabla.ttf")).expect("nabla decoded");
        property
            .with_downcast_mut::<ViewModelInstanceAssetFont, _>(|property| {
                property.set_value(Some(nabla.clone()))
            })
            .expect("native font property");
        machine.advance_and_apply(0.0);
        let installed = property
            .with_downcast::<ViewModelInstanceAssetFont, _>(|property| property.asset().font())
            .flatten()
            .expect("backing FontAsset retains nabla");
        assert!(Rc::ptr_eq(&installed, &nabla));
        assert!(!Rc::ptr_eq(&installed, &kablammo));

        property
            .with_downcast_mut::<ViewModelInstanceAssetFont, _>(|property| property.set_value(None))
            .expect("native font property");
        machine.advance_and_apply(0.0);
        assert!(
            property
                .with_downcast::<ViewModelInstanceAssetFont, _>(|property| property.asset().font())
                .flatten()
                .is_none()
        );
    }

    #[test]
    fn retained_native_artboard_outlives_the_importing_file_handle() {
        let instance = {
            let file = import_native(FIXTURE, None);
            file.with_file(File::artboard_default)
                .expect("instantiate default artboard")
        };
        instance.advance_default(0.016);
        assert!(instance.with_artboard(|artboard| !artboard.base.objects().is_empty()));
    }
}
