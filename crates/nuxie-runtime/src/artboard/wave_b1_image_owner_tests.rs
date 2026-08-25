#[test]
fn wave_b1_data_binding_images_preserve_exact_root_and_nested_image_asset_owners() {
    let fixture = PathBuf::from(
        std::env::var_os("RIVE_RUNTIME_DIR")
            .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into()),
    )
    .join("tests/unit_tests/assets/data_binding_images_test.riv");
    let file = read_runtime_file(&std::fs::read(&fixture).expect("read image fixture"))
        .expect("import image fixture");
    let graphs = GraphFile::from_runtime_file(&file).expect("build image graphs");
    let graph = graphs
        .artboards
        .iter()
        .find(|graph| graph.name.as_deref() == Some("main"))
        .expect("main artboard graph");
    let mut artboard = ArtboardInstance::from_graph_with_artboards(&file, graph, &graphs.artboards)
        .expect("instantiate main artboard");
    let view_model_index = file
        .resolved_view_model_for_artboard(
            graphs
                .artboards
                .iter()
                .position(|candidate| std::ptr::eq(candidate, graph))
                .expect("main artboard index"),
        )
        .expect("main ViewModel")
        .view_model_index;
    let instance_index = file
        .view_model_default_instance(view_model_index)
        .expect("main default ViewModel instance")
        .instance_index;
    let context = RuntimeOwnedViewModelHandle::new(
        RuntimeOwnedViewModelInstance::from_instance(&file, view_model_index, instance_index)
            .expect("instantiate main default ViewModel"),
    );
    assert!(artboard.bind_owned_view_model_artboard_handle(&file, &context));
    artboard
        .advance(0.0)
        .expect("initial image binding advance");

    let root_image = artboard
        .slots()
        .iter()
        .find(|slot| slot.name.as_deref() == Some("root_img"))
        .expect("root_img Image owner")
        .local_id;
    let nested_host = artboard
        .slots()
        .iter()
        .find(|slot| slot.name.as_deref() == Some("sub_1"))
        .expect("sub_1 NestedArtboard owner")
        .local_id;
    let nested = artboard
        .nested_artboards
        .get(&nested_host)
        .expect("sub_1 mounted artboard");
    let nested_image = nested
        .child
        .slots()
        .iter()
        .find(|slot| slot.name.as_deref() == Some("sub_1_img"))
        .expect("sub_1_img Image owner")
        .local_id;

    let main_initial = context
        .borrow()
        .asset_value_by_property_name_path("main_im")
        .expect("main_im asset index");
    let sub_context = context
        .linked_view_model_by_property_name_path("sub_1")
        .expect("sub_1 ViewModel owner");
    let sub_initial = sub_context
        .borrow()
        .asset_value_by_property_name_path("sub_1_im")
        .expect("sub_1_im asset index");
    let main_initial_global = file
        .file_asset(usize::try_from(main_initial).expect("main asset index fits"))
        .expect("main ImageAsset owner")
        .id;
    let sub_initial_global = file
        .file_asset(usize::try_from(sub_initial).expect("nested asset index fits"))
        .expect("nested ImageAsset owner")
        .id;
    assert_eq!(
        artboard.resolved_image_asset_global(Some(root_image), None),
        Some(main_initial_global)
    );
    assert_eq!(
        nested
            .child
            .resolved_image_asset_global(Some(nested_image), None),
        Some(sub_initial_global)
    );

    assert!(
        context
            .borrow_mut()
            .set_asset_by_property_name_path("main_im", 2)
    );
    assert!(
        sub_context
            .borrow_mut()
            .set_asset_by_property_name_path("sub_1_im", 6)
    );
    artboard
        .advance(0.0)
        .expect("updated image binding advance");
    let updated_main_global = file.file_asset(2).expect("updated main ImageAsset").id;
    let updated_sub_global = file.file_asset(6).expect("updated nested ImageAsset").id;
    assert_ne!(main_initial_global, updated_main_global);
    assert_ne!(sub_initial_global, updated_sub_global);
    assert_eq!(
        artboard.resolved_image_asset_global(Some(root_image), None),
        Some(updated_main_global)
    );
    let nested = artboard
        .nested_artboards
        .get(&nested_host)
        .expect("sub_1 remains mounted");
    assert_eq!(
        nested
            .child
            .resolved_image_asset_global(Some(nested_image), None),
        Some(updated_sub_global)
    );
}
