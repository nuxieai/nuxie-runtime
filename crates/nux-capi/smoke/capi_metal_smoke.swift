import Foundation
import Metal
import QuartzCore
import NuxieRuntimeInternal

final class CompletionBox {
    let semaphore = DispatchSemaphore(value: 0)
}

func check(_ condition: @autoclosure () -> Bool, _ message: String) {
    guard condition() else {
        FileHandle.standardError.write(Data("swift-metal-smoke FAILED: \(message)\n".utf8))
        exit(1)
    }
}

func freeResult(_ result: OpaquePointer?, _ expected: UInt32) {
    check(result != nil, "owned result")
    var status = NUX_STATUS_RUNTIME_ERROR.rawValue
    check(nux_capi_result_status(result, &status) == NUX_STATUS_OK.rawValue, "result status")
    check(status == expected, "expected diagnostic status")
    check(nux_capi_result_free(result) == NUX_STATUS_OK.rawValue, "free result")
}

guard CommandLine.arguments.count == 2 else {
    FileHandle.standardError.write(Data("usage: capi_metal_smoke <smi_test.riv>\n".utf8))
    exit(2)
}

let bytes = try Data(contentsOf: URL(fileURLWithPath: CommandLine.arguments[1]))
var file: OpaquePointer?
check(bytes.withUnsafeBytes {
    nux_file_import($0.bindMemory(to: UInt8.self).baseAddress, $0.count, &file)
} == NUX_STATUS_OK.rawValue, "import")
var artboard: OpaquePointer?
check(nux_artboard_instance_new(file, 1, &artboard) == NUX_STATUS_OK.rawValue, "artboard")
var player: OpaquePointer?
check(nux_player_new_static(artboard, &player) == NUX_STATUS_OK.rawValue, "player")
check(nux_file_free(file) == NUX_STATUS_OK.rawValue, "release file before player")
check(nux_artboard_instance_free(artboard) == NUX_STATUS_OK.rawValue, "release artboard before player")

var renderer: OpaquePointer?
var result: OpaquePointer?
check(nux_renderer_new_metal(4, 3, &renderer, &result) == NUX_STATUS_OK.rawValue, "renderer")
freeResult(result, NUX_STATUS_OK.rawValue)

var rawDevice: UnsafeMutableRawPointer?
result = nil
check(nux_renderer_copy_metal_device(renderer, &rawDevice, &result) == NUX_STATUS_OK.rawValue, "device")
freeResult(result, NUX_STATUS_OK.rawValue)
check(rawDevice != nil, "+1 MTLDevice")
let deviceObject = Unmanaged<AnyObject>.fromOpaque(rawDevice!).takeRetainedValue()
guard let device = deviceObject as? MTLDevice else {
    fatalError("copied pointer is not an MTLDevice")
}

let layer = CAMetalLayer()
layer.device = device
layer.pixelFormat = .bgra8Unorm
layer.framebufferOnly = true
layer.drawableSize = CGSize(width: 4, height: 3)
layer.maximumDrawableCount = 2
layer.allowsNextDrawableTimeout = true
guard let drawable = layer.nextDrawable() else {
    fatalError("configured layer did not vend a drawable")
}

var operation = NuxMetalRenderOperation()
operation.struct_size = UInt32(MemoryLayout<NuxMetalRenderOperation>.size)
operation.drawable_state = UInt32(NUX_METAL_DRAWABLE_STATE_AVAILABLE)
operation.drawable = Unmanaged.passUnretained(drawable).toOpaque()
operation.clear_color = 0xff11_2233
let completion = CompletionBox()
operation.completion_context = Unmanaged.passRetained(completion).toOpaque()
operation.completion_callback = { context in
    guard let context else { return }
    let completion = Unmanaged<CompletionBox>.fromOpaque(context).takeRetainedValue()
    completion.semaphore.signal()
}
var outcome = NuxRendererOutcome()
outcome.struct_size = UInt32(MemoryLayout<NuxRendererOutcome>.size)
result = OpaquePointer(bitPattern: 1)
check(
    nux_renderer_render_player(renderer, player, &operation, &outcome, &result) == NUX_STATUS_OK.rawValue,
    "render"
)
check(result == nil, "success render performs no diagnostic allocation")
check(outcome.disposition == UInt32(NUX_RENDERER_DISPOSITION_PRESENTED), "presented")
check(
    completion.semaphore.wait(timeout: .now() + 5) == .success,
    "deferred Metal completion"
)

check(nux_renderer_free(renderer) == NUX_STATUS_OK.rawValue, "free renderer")
check(nux_player_free(player) == NUX_STATUS_OK.rawValue, "free player")
print("swift-metal-smoke ok")
