import NuxieRuntime

func typecheckNuxieRuntimeModule(bytes: UnsafePointer<UInt8>, count: UInt64) {
    _ = nux_runtime_abi_major()
    _ = nux_runtime_abi_minor()
    _ = nux_runtime_require_abi(1, 0)

    let byteView = NuxByteView(data: bytes, len: count)
    _ = NuxFlowImportRequest(
        struct_size: UInt32(MemoryLayout<NuxFlowImportRequest>.size),
        artifact_bytes: byteView
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
    _ = nux_apple_surface_copy_metal_device(nil, &metalDevice, &result)
    _ = NUX_SURFACE_DISPOSITION_PRESENTED
}
