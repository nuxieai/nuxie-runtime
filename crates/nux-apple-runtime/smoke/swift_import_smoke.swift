import NuxieRuntime

func typecheckNuxieRuntimeModule(bytes: UnsafePointer<UInt8>, count: UInt64) {
    _ = nux_runtime_abi_major()
    _ = nux_runtime_abi_minor()
    _ = nux_runtime_require_abi(1, 5)

    let byteView = NuxByteView(data: bytes, len: count)
    _ = NuxFlowImportRequest(
        struct_size: UInt32(MemoryLayout<NuxFlowImportRequest>.size),
        artifact_bytes: byteView,
        expected_flow_id: NuxByteView(data: nil, len: 0),
        expected_build_id: NuxByteView(data: nil, len: 0),
        manifest_bytes: NuxByteView(data: nil, len: 0),
        signature_envelope_bytes: NuxByteView(data: nil, len: 0),
        selected_key: nil,
        external_assets: nil,
        external_asset_count: 0
    )
    _ = NuxFlowAuthorizationKey(
        struct_size: UInt32(MemoryLayout<NuxFlowAuthorizationKey>.size),
        key_id: NuxByteView(data: nil, len: 0),
        ed25519_public_key: NuxByteView(data: nil, len: 0)
    )
    _ = NuxFlowExternalAsset(
        struct_size: UInt32(MemoryLayout<NuxFlowExternalAsset>.size),
        kind: UInt32(NUX_FLOW_EXTERNAL_ASSET_KIND_IMAGE),
        asset_id: 0,
        required: false,
        provided: false,
        unique_name: NuxByteView(data: nil, len: 0),
        source_key: NuxByteView(data: nil, len: 0),
        expected_sha256: NuxByteView(data: nil, len: 0),
        bytes: NuxByteView(data: nil, len: 0)
    )
    var diagnosticView = NuxDiagnosticView(
        struct_size: UInt32(MemoryLayout<NuxDiagnosticView>.size),
        severity: UInt32(NUX_DIAGNOSTIC_SEVERITY_DEBUG),
        code: NuxByteView(data: nil, len: 0),
        message: NuxByteView(data: nil, len: 0)
    )
    _ = NuxFlowSessionDescriptor(
        struct_size: UInt32(MemoryLayout<NuxFlowSessionDescriptor>.size),
        artboard_name: NuxByteView(data: nil, len: 0),
        state_machine_name: NuxByteView(data: nil, len: 0)
    )
    _ = NuxFrameOperation(
        struct_size: UInt32(MemoryLayout<NuxFrameOperation>.size),
        elapsed_seconds: 0,
        render: false,
        apple_drawable: nil,
        completion_context: nil,
        completion_callback: nil
    )
    _ = NuxAppleSurfaceDescriptor(
        struct_size: UInt32(MemoryLayout<NuxAppleSurfaceDescriptor>.size),
        pixel_width: 1,
        pixel_height: 1
    )
    _ = NuxFlowConfiguredSessionDescriptor(
        struct_size: UInt32(MemoryLayout<NuxFlowConfiguredSessionDescriptor>.size),
        required_abi_major: 1,
        minimum_abi_minor: 5,
        artboard_name: NuxByteView(data: nil, len: 0),
        player_name: NuxByteView(data: nil, len: 0)
    )
    _ = NuxFlowValueNode(
        struct_size: UInt32(MemoryLayout<NuxFlowValueNode>.size),
        kind: UInt32(NUX_FLOW_VALUE_KIND_NULL),
        number_value: 0,
        color_value: 0,
        bool_value: 0,
        first_edge: 0,
        edge_count: 0,
        has_instance_id: 0,
        instance_id: 0,
        identity_value: 0,
        string_value: NuxByteView(data: nil, len: 0),
        schema_id: NuxByteView(data: nil, len: 0)
    )
    _ = NuxFlowPointerEvent(
        struct_size: UInt32(MemoryLayout<NuxFlowPointerEvent>.size),
        kind: UInt32(NUX_FLOW_POINTER_EVENT_KIND_DOWN),
        pointer_id: 1,
        x: 0,
        y: 0,
        timestamp_seconds: 0
    )
    _ = NuxFlowAdvanceOperation(
        struct_size: UInt32(MemoryLayout<NuxFlowAdvanceOperation>.size),
        timestamp_seconds: 0,
        delta_seconds: 0,
        render: 0,
        apple_drawable: nil,
        completion_context: nil,
        completion_callback: nil
    )
    _ = NuxFlowQuery(
        struct_size: UInt32(MemoryLayout<NuxFlowQuery>.size),
        kind: UInt32(NUX_FLOW_QUERY_KIND_PLAYER_INPUTS)
    )
    let zeroInstanceReference = NuxFlowInstanceReference(
        kind: 0,
        local_id: 0,
        instance_id: 0
    )
    _ = NuxFlowStateMutation(
        struct_size: UInt32(MemoryLayout<NuxFlowStateMutation>.size),
        kind: UInt32(NUX_FLOW_STATE_MUTATION_KIND_SET_INPUT_BOOL),
        instance: zeroInstanceReference,
        item: zeroInstanceReference,
        path: NuxByteView(data: nil, len: 0),
        input_name: byteView,
        value_root_index: 0,
        index: 0,
        other_index: 0
    )
    _ = NuxFlowTextRunMutation(
        struct_size: UInt32(MemoryLayout<NuxFlowTextRunMutation>.size),
        name: byteView,
        text: NuxByteView(data: nil, len: 0)
    )
    _ = NuxFlowTextRunBatch(
        struct_size: UInt32(MemoryLayout<NuxFlowTextRunBatch>.size),
        mutations: nil,
        mutation_count: 0
    )
    _ = NuxFlowSessionOperation(
        struct_size: UInt32(MemoryLayout<NuxFlowSessionOperation>.size),
        required_abi_major: 1,
        minimum_abi_minor: 5,
        kind: UInt32(NUX_FLOW_SESSION_OPERATION_KIND_QUERY),
        state_batch: nil,
        pointer_batch: nil,
        advance: nil,
        query_batch: nil,
        text_run_batch: nil
    )
    var playerMetadata = NuxFlowPlayerMetadataView(
        struct_size: UInt32(MemoryLayout<NuxFlowPlayerMetadataView>.size),
        kind: UInt32(NUX_FLOW_PLAYER_KIND_STATIC),
        selection: UInt32(NUX_FLOW_PLAYER_SELECTION_STATIC),
        player_index: UInt32.max,
        artboard_name: NuxByteView(data: nil, len: 0),
        player_name: NuxByteView(data: nil, len: 0),
        min_x: 0,
        min_y: 0,
        max_x: 0,
        max_y: 0
    )
    var outputView = NuxFlowOutputView(
        struct_size: UInt32(MemoryLayout<NuxFlowOutputView>.size),
        phase: UInt32(NUX_FLOW_OUTPUT_PHASE_DELAYED_EVENT_CALLBACKS),
        kind: UInt32(NUX_FLOW_OUTPUT_KIND_REPORTED_EVENT),
        payload_root_index: UInt32.max,
        has_origin_mutation_id: 0,
        has_instance_id: 0,
        sequence: 0,
        cycle: 0,
        origin_mutation_id: 0,
        instance_id: 0,
        event_type: 0,
        first_event_property: 0,
        event_property_count: 0,
        delay_seconds: 0,
        name: NuxByteView(data: nil, len: 0),
        path: NuxByteView(data: nil, len: 0),
        payload: NuxByteView(data: nil, len: 0),
        has_open_url: 0,
        open_url: NuxByteView(data: nil, len: 0),
        open_url_target: NuxByteView(data: nil, len: 0)
    )
    var playerInput = NuxFlowPlayerInputView(
        struct_size: UInt32(MemoryLayout<NuxFlowPlayerInputView>.size),
        kind: UInt32(NUX_FLOW_PLAYER_INPUT_KIND_BOOL),
        value_root_index: 0,
        name: NuxByteView(data: nil, len: 0)
    )
    var metalDevice: UnsafeMutableRawPointer?
    var result: OpaquePointer?
    var sessionResult: OpaquePointer?
    var authenticatedKeyID = NuxByteView(data: nil, len: 0)
    _ = nux_apple_surface_copy_metal_device(nil, &metalDevice, &result)
    _ = nux_operation_result_script_authorization(result)
    _ = nux_operation_result_authenticated_key_id(result, &authenticatedKeyID)
    _ = nux_operation_result_diagnostic_count(result)
    _ = nux_operation_result_diagnostic_at(result, 0, &diagnosticView)
    _ = nux_flow_render_session_create_configured(nil, nil, nil, &sessionResult)
    _ = nux_flow_render_session_perform(nil, nil, &sessionResult)
    _ = nux_flow_session_result_status(sessionResult)
    _ = nux_flow_session_result_player_metadata(sessionResult, &playerMetadata)
    _ = nux_flow_session_result_player_input_count(sessionResult)
    _ = nux_flow_session_result_player_input_at(sessionResult, 0, &playerInput)
    _ = nux_flow_session_result_output_count(sessionResult)
    _ = nux_flow_session_result_output_at(sessionResult, 0, &outputView)
    nux_flow_session_result_free(sessionResult)
    _ = NUX_SURFACE_DISPOSITION_PRESENTED
}
