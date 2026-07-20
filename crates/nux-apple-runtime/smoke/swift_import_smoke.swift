import NuxieRuntime

func typecheckNuxieRuntimeModule(bytes: UnsafePointer<UInt8>, count: UInt64) {
    _ = nux_runtime_abi_major()
    _ = nux_runtime_abi_minor()
    _ = nux_runtime_require_abi(1, 1)

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
    var metalDevice: UnsafeMutableRawPointer?
    var result: OpaquePointer?
    var authenticatedKeyID = NuxByteView(data: nil, len: 0)
    _ = nux_apple_surface_copy_metal_device(nil, &metalDevice, &result)
    _ = nux_operation_result_script_authorization(result)
    _ = nux_operation_result_authenticated_key_id(result, &authenticatedKeyID)
    _ = nux_operation_result_diagnostic_count(result)
    _ = nux_operation_result_diagnostic_at(result, 0, &diagnosticView)
    _ = NUX_SURFACE_DISPOSITION_PRESENTED
}
