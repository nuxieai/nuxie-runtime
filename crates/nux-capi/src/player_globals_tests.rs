use super::*;

fn import_upstream() -> *mut NuxFile {
    let root = std::env::var_os("RIVE_RUNTIME_DIR").expect("upstream assets");
    let bytes = std::fs::read(
        std::path::Path::new(&root).join("tests/unit_tests/assets/global_variables_test.riv"),
    )
    .unwrap();
    let mut file = ptr::null_mut();
    assert_eq!(
        unsafe {
            nux_file_import(
                bytes.as_ptr(),
                bytes.len(),
                &NuxRenderCallbacks::default(),
                &mut file,
            )
        },
        NuxStatus::Ok
    );
    file
}
fn native_global(player: *mut NuxPlayer, name: &str) -> Option<nuxie::runtime::core::CoreHandle> {
    let player = unsafe { &*player };
    let instance = player.instance.borrow();
    let PlayerInstance::StateMachine(machine) = &*instance else {
        panic!("state machine");
    };
    machine
        .native_handle()
        .with_instance(|machine| machine.global_view_model_instance(name))
}
#[test]
fn global_cpp_slot_identity_and_fresh_default_through_c_api() {
    // 7acbdfecb global_view_model_binding_test.cpp:60-82, 250-262, 265-326:
    // set retains identity; an unknown name is absent; clear/bind creates a new default.
    unsafe {
        let file = import_upstream();
        let names = (*file)
            .file
            .with_file(|file| file.global_view_model_names());
        assert!(!names.is_empty());
        let name = &names[0];
        let view = NuxStringView {
            data: name.as_ptr().cast(),
            len: name.len(),
        };
        let schema = (*file)
            .view_model_catalog
            .global_schema_named(name)
            .unwrap();
        let mut artboard = ptr::null_mut();
        let mut player = ptr::null_mut();
        assert_eq!(
            nux_artboard_instance_new(file, 0, &mut artboard),
            NuxStatus::Ok
        );
        assert_eq!(nux_player_new_default(artboard, &mut player), NuxStatus::Ok);
        let mut instance = ptr::null_mut();
        assert_eq!(
            nux_view_model_instance_new(file, schema, &mut instance),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_player_set_global_view_model(player, view, ptr::null()),
            NuxStatus::Ok
        );
        let original = native_global(player, name).unwrap();
        assert_eq!(
            nux_player_set_global_view_model(player, view, instance),
            NuxStatus::Ok
        );
        assert_eq!(
            native_global(player, name).unwrap(),
            (*instance).instance.native_handle()
        );
        assert_eq!(
            nux_player_set_global_view_model(
                player,
                NuxStringView::from_static("not-a-global"),
                instance
            ),
            NuxStatus::NotFound
        );
        assert!(native_global(player, "not-a-global").is_none());
        assert_eq!(
            nux_player_set_global_view_model(player, view, ptr::null()),
            NuxStatus::Ok
        );
        let fresh = native_global(player, name).unwrap();
        assert_ne!(fresh, original);
        assert_ne!(fresh, (*instance).instance.native_handle());
        // Pinned cpp:329-360 refuses a real model that is not global.
        let mut catalog = ptr::null_mut();
        assert_eq!(
            nux_file_view_model_catalog(file, &mut catalog),
            NuxStatus::Ok
        );
        let mut info = NuxViewModelCatalogInfo::default();
        assert_eq!(
            nux_view_model_catalog_info(catalog, &mut info),
            NuxStatus::Ok
        );
        let mut checked_non_global = false;
        for index in 0..info.schema_count {
            let mut schema = NuxViewModelSchemaView::default();
            assert_eq!(
                nux_view_model_catalog_schema(catalog, index, &mut schema),
                NuxStatus::Ok
            );
            if schema.is_global == 0 {
                assert_eq!(
                    nux_player_set_global_view_model(player, schema.name, instance),
                    NuxStatus::NotFound
                );
                checked_non_global = true;
            }
        }
        assert!(checked_non_global);
        nux_view_model_catalog_free(catalog);
        nux_view_model_instance_free(instance);
        nux_player_free(player);
        nux_artboard_instance_free(artboard);
        nux_file_free(file);
    }
}
#[test]
fn global_wrong_thread_and_reentry_do_not_change_the_slot() {
    unsafe {
        let file = import_upstream();
        let names = (*file)
            .file
            .with_file(|file| file.global_view_model_names());
        let name = &names[0];
        let view = NuxStringView {
            data: name.as_ptr().cast(),
            len: name.len(),
        };
        let schema = (*file)
            .view_model_catalog
            .global_schema_named(name)
            .unwrap();
        let mut a = ptr::null_mut();
        let mut p = ptr::null_mut();
        let mut v = ptr::null_mut();
        assert_eq!(nux_artboard_instance_new(file, 0, &mut a), NuxStatus::Ok);
        assert_eq!(nux_player_new_default(a, &mut p), NuxStatus::Ok);
        assert_eq!(
            nux_view_model_instance_new(file, schema, &mut v),
            NuxStatus::Ok
        );
        assert_eq!(nux_player_set_global_view_model(p, view, v), NuxStatus::Ok);
        let original = native_global(p, name).unwrap();
        let address = p as usize;
        assert_eq!(
            std::thread::spawn(move || nux_player_set_global_view_model(
                address as *mut NuxPlayer,
                NuxStringView::default(),
                ptr::null()
            ))
            .join()
            .unwrap(),
            NuxStatus::WrongThread
        );
        let active = enter_occurrence(&(*p).artboard).unwrap();
        assert_eq!(
            nux_player_set_global_view_model(p, view, ptr::null()),
            NuxStatus::ReentrantCall
        );
        drop(active);
        let active = (*v).instance.borrow_mut();
        assert_eq!(
            nux_player_set_global_view_model(p, view, v),
            NuxStatus::ReentrantCall
        );
        drop(active);
        assert_eq!(native_global(p, name).unwrap(), original);
        nux_player_free(p);
        nux_artboard_instance_free(a);
        nux_view_model_instance_free(v);
        nux_file_free(file);
    }
}
