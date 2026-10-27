//! Public artboard_instance_sharing_test.cpp at 73d7b1c2.
#![cfg(feature = "test-support")]
#[path = "support/import_439.rs"]
mod support;
use support::*;

fn load() -> RuntimeFileHandle {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    import(
        &std::fs::read(
            std::path::PathBuf::from(root)
                .join("tests/unit_tests/assets/multiple_state_machines.riv"),
        )
        .unwrap(),
    )
}

#[test]
fn instance_reports_its_sources_animations_and_state_machines() {
    let file = load();
    let source = artboard(&file);
    let instance = file.with_file(File::artboard_default).unwrap();
    source
        .with_downcast::<Artboard, _>(|source_board| {
            assert!(source_board.state_machine_count() > 0);
            instance.with_artboard(|instance| {
                assert!(instance.is_instance());
                assert_eq!(instance.artboard_source_handle(), Some(source.clone()));
                assert_eq!(instance.animation_count(), source_board.animation_count());
                assert_eq!(
                    instance.state_machine_count(),
                    source_board.state_machine_count()
                );
                for i in 0..source_board.animation_count() {
                    assert_eq!(
                        instance.animation_handle_at(i),
                        source_board.animation_handle_at(i)
                    );
                    assert_eq!(
                        instance.animation_name_at(i),
                        source_board.animation_name_at(i)
                    );
                    assert_eq!(
                        instance
                            .base
                            .animation_named(&source_board.animation_name_at(i)),
                        source_board.animation_handle_at(i)
                    );
                }
                for i in 0..source_board.state_machine_count() {
                    assert_eq!(
                        instance.state_machine_handle_at(i),
                        source_board.state_machine_handle_at(i)
                    );
                    assert_eq!(
                        instance.state_machine_name_at(i),
                        source_board.state_machine_name_at(i)
                    );
                    assert_eq!(
                        instance
                            .base
                            .state_machine_named(&source_board.state_machine_name_at(i)),
                        source_board.state_machine_handle_at(i)
                    );
                }
                assert_eq!(
                    instance.animation_handle_at(source_board.animation_count()),
                    None
                );
                assert_eq!(
                    instance.state_machine_handle_at(source_board.state_machine_count()),
                    None
                );
                assert_eq!(
                    instance.default_state_machine_index(),
                    source_board.default_state_machine_index()
                );
            });
        })
        .unwrap();
}

#[test]
fn instance_of_instance_reaches_original_source() {
    let file = load();
    let source = artboard(&file);
    let instance = file.with_file(File::artboard_default).unwrap();
    let nested = instance.instance().unwrap();
    source
        .with_downcast::<Artboard, _>(|source_board| {
            nested.with_artboard(|nested| {
                assert_eq!(nested.artboard_source_handle(), Some(source.clone()));
                assert_eq!(
                    nested.state_machine_count(),
                    source_board.state_machine_count()
                );
                assert_eq!(nested.animation_count(), source_board.animation_count());
            })
        })
        .unwrap();
}

#[test]
fn instance_object_list_carries_no_spare_capacity() {
    let file = load();
    let instance = file.with_file(File::artboard_default).unwrap();
    instance.with_artboard(|instance| {
        assert!(instance.objects().len() > 1);
        assert_eq!(instance.objects_capacity(), instance.objects().len());
    });
}
