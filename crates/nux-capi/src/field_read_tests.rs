use super::*;

#[test]
fn published_f3_field_read_uses_delivered_native_name() {
    unsafe {
        let bytes = include_bytes!("../tests/fixtures/published-input/screen.riv");
        let table: serde_json::Value = serde_json::from_str(include_str!(
            "../tests/fixtures/published-input/text-inputs.json"
        ))
        .unwrap();
        let name = table[0]["textInputName"].as_str().unwrap();
        let view = |s: &str| NuxStringView {
            data: s.as_ptr().cast(),
            len: s.len(),
        };
        let mut file = ptr::null_mut();
        assert_eq!(
            nux_file_import(
                bytes.as_ptr(),
                bytes.len(),
                &NuxRenderCallbacks::default(),
                &mut file
            ),
            NuxStatus::Ok
        );
        let mut catalog = ptr::null_mut();
        assert_eq!(
            nux_file_view_model_catalog(file, &mut catalog),
            NuxStatus::Ok
        );
        let mut catalog_info = NuxViewModelCatalogInfo::default();
        catalog_info.struct_size = std::mem::size_of::<NuxViewModelCatalogInfo>() as u32;
        assert_eq!(
            nux_view_model_catalog_info(catalog, &mut catalog_info),
            NuxStatus::Ok
        );
        let schema = |name: &str| {
            (0..catalog_info.schema_count)
                .find(|&index| {
                    let mut schema = NuxViewModelSchemaView::default();
                    schema.struct_size = std::mem::size_of::<NuxViewModelSchemaView>() as u32;
                    assert_eq!(
                        nux_view_model_catalog_schema(catalog, index, &mut schema),
                        NuxStatus::Ok
                    );
                    std::slice::from_raw_parts(schema.name.data.cast::<u8>(), schema.name.len)
                        == name.as_bytes()
                })
                .unwrap()
        };
        let mut experience = ptr::null_mut();
        let mut root = ptr::null_mut();
        assert_eq!(
            nux_view_model_instance_new_authored(file, schema("Experience"), 0, &mut experience),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_view_model_instance_new_authored(
                file,
                schema("Runtime input scr_screens_sinput"),
                0,
                &mut root
            ),
            NuxStatus::Ok
        );
        let mutation = NuxViewModelMutation {
            kind: NUX_VIEW_MODEL_MUTATION_KIND_SET_VIEW_MODEL,
            instance: root,
            path: view("experience"),
            related_instance: experience,
            ..Default::default()
        };
        let batch = NuxViewModelMutationBatch {
            struct_size: std::mem::size_of::<NuxViewModelMutationBatch>() as u32,
            mutations: &mutation,
            mutation_count: 1,
            ..Default::default()
        };
        let mut result = ptr::null_mut();
        assert_eq!(nux_view_model_mutate(&batch, &mut result), NuxStatus::Ok);
        let mut mutation_info = NuxViewModelMutationResultInfo::default();
        mutation_info.struct_size = std::mem::size_of::<NuxViewModelMutationResultInfo>() as u32;
        assert_eq!(
            nux_view_model_mutation_result_info(result, &mut mutation_info),
            NuxStatus::Ok
        );
        assert_eq!(mutation_info.status, NuxStatus::Ok);
        assert_eq!(nux_view_model_mutation_result_free(result), NuxStatus::Ok);
        let mut artboard = ptr::null_mut();
        assert_eq!(
            nux_artboard_instance_new(file, 1, &mut artboard),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_artboard_instance_bind_view_model(artboard, root),
            NuxStatus::Ok
        );
        let mut player = ptr::null_mut();
        assert_eq!(nux_player_new_default(artboard, &mut player), NuxStatus::Ok);
        assert_eq!(nux_player_enable_semantics(player), NuxStatus::Ok);
        let focus = NuxPlayerFocusInput {
            kind: NUX_PLAYER_FOCUS_KIND_NEXT,
            ..Default::default()
        };
        let step = NuxPlayerStep {
            struct_size: std::mem::size_of::<NuxPlayerStep>() as u32,
            focus_inputs: &focus,
            focus_input_count: 1,
            ..Default::default()
        };
        let mut result = ptr::null_mut();
        assert_eq!(nux_player_step(player, &step, &mut result), NuxStatus::Ok);
        let mut info = NuxPlayerSchedulingInfo {
            struct_size: std::mem::size_of::<NuxPlayerSchedulingInfo>() as u32,
            ..Default::default()
        };
        assert_eq!(
            nux_player_step_result_scheduling(result, &mut info),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_player_acknowledge_presented(player, info.render_revision),
            NuxStatus::Ok
        );
        assert_eq!(nux_player_step_result_free(result), NuxStatus::Ok);
        let mut snapshot = ptr::null_mut();
        assert_eq!(
            nux_player_semantic_snapshot(player, &mut snapshot),
            NuxStatus::Ok
        );
        let fields: Vec<_> = (&(*snapshot).nodes)
            .iter()
            .filter(|n| n.role == NUX_SEMANTIC_ROLE_TEXT_FIELD)
            .collect();
        assert_eq!(fields.len(), 1);
        let mut length = 0;
        assert_eq!(
            nux_player_field_string_copy(
                player,
                snapshot,
                fields[0].id,
                view(name),
                ptr::null_mut(),
                0,
                &mut length
            ),
            NuxStatus::Ok
        );
        let mut value = vec![0; length];
        assert_eq!(
            nux_player_field_string_copy(
                player,
                snapshot,
                fields[0].id,
                view(name),
                value.as_mut_ptr(),
                value.len(),
                &mut length
            ),
            NuxStatus::Ok
        );
        assert!(value == b"Ada", "authored initial text is preserved");
        let mut short = [0xab; 2];
        assert_eq!(
            nux_player_field_string_copy(
                player,
                snapshot,
                fields[0].id,
                view(name),
                short.as_mut_ptr(),
                short.len(),
                &mut length
            ),
            NuxStatus::LimitExceeded
        );
        assert_eq!(short, [0xab; 2]);
        let native = (&(*player).artboard).instance.borrow().native_handle();
        let manager = native.with_artboard(|a| a.semantic_manager()).unwrap();
        let node = manager
            .with_semantic_manager(|m| m.node_by_id(fields[0].id))
            .unwrap();
        // A pending numeric draft must be copied from native storage, without
        // substituting the bound model value or normalizing it to a number.
        let property = field_string_property(&node, name).unwrap();
        assert!(
            nuxie::runtime::generated::core_registry::CoreRegistry::set_string_handle(
                &property,
                817,
                "21.".into()
            )
        );
        let mut draft = [0; 3];
        assert_eq!(
            nux_player_field_string_copy(
                player,
                snapshot,
                fields[0].id,
                view(name),
                draft.as_mut_ptr(),
                draft.len(),
                &mut length
            ),
            NuxStatus::Ok
        );
        assert!(
            draft == *b"21.",
            "native draft retains its trailing decimal point"
        );
        let mut ancestors = vec![node.clone()];
        loop {
            let parent = ancestors.last().unwrap().borrow().parent();
            let Some(parent) = parent else { break };
            // Drop the last node borrow before growing the vector.
            let parent = parent.clone();
            ancestors.push(parent);
        }
        for ancestor in ancestors {
            let original = ancestor.borrow().state_flags;
            for flag in [SemanticState::HIDDEN.0, SemanticState::DISABLED.0] {
                ancestor.borrow_mut().state_flags = original | flag;
                assert!(matches!(field_data(&node), Err(NuxStatus::NotFound)));
            }
            ancestor.borrow_mut().state_flags = original;
            assert!(field_data(&node).is_ok());
        }
        assert_eq!(nux_semantic_snapshot_free(snapshot), NuxStatus::Ok);
        assert_eq!(nux_player_free(player), NuxStatus::Ok);
        assert_eq!(nux_artboard_instance_free(artboard), NuxStatus::Ok);
        assert_eq!(nux_view_model_instance_free(root), NuxStatus::Ok);
        assert_eq!(nux_view_model_instance_free(experience), NuxStatus::Ok);
        assert_eq!(nux_view_model_catalog_free(catalog), NuxStatus::Ok);
        assert_eq!(nux_file_free(file), NuxStatus::Ok);
    }
}
