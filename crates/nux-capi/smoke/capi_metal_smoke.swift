import Foundation
import CoreGraphics
import ImageIO
import Metal
import QuartzCore
import NuxieRuntimeC

final class CompletionBox {
    let semaphore = DispatchSemaphore(value: 0)
}

final class DecodeBox {
    var pixels: UnsafeMutableRawPointer?
    var pixelCount = 0
    var calls = 0
    var retains = 0
    var releases = 0
    var nestedABI = UInt32(NUX_CAPI_ABI_VERSION)

    deinit { free(pixels) }
}

func retainPixels(_ context: UnsafeMutableRawPointer?) {
    guard let context else { return }
    let box = Unmanaged<DecodeBox>.fromOpaque(context).takeUnretainedValue()
    box.retains += 1
    box.nestedABI = nux_capi_abi_version()
}

func releasePixels(_ context: UnsafeMutableRawPointer?) {
    guard let context else { return }
    Unmanaged<DecodeBox>.fromOpaque(context).takeUnretainedValue().releases += 1
}

func decodeImage(
    _ context: UnsafeMutableRawPointer?,
    _ request: UnsafePointer<NuxImageDecodeRequest>?,
    _ outImage: UnsafeMutablePointer<NuxDecodedImage>?
) -> UInt32 {
    guard let context, let request, let outImage,
          let encodedPointer = request.pointee.encoded.data else {
        return UInt32(NUX_ASSET_CALLBACK_STATUS_FAILED)
    }
    let box = Unmanaged<DecodeBox>.fromOpaque(context).takeUnretainedValue()
    let encoded = Data(bytes: encodedPointer, count: request.pointee.encoded.len)
    guard let source = CGImageSourceCreateWithData(encoded as CFData, nil),
          let image = CGImageSourceCreateImageAtIndex(source, 0, nil) else {
        return UInt32(NUX_ASSET_CALLBACK_STATUS_FAILED)
    }
    let width = image.width
    let height = image.height
    let rowBytes = width * 4
    let count = rowBytes * height
    guard width > 0, height > 0,
          width <= Int(request.pointee.maximum_dimension),
          height <= Int(request.pointee.maximum_dimension),
          count <= request.pointee.maximum_decoded_bytes else {
        return UInt32(NUX_ASSET_CALLBACK_STATUS_FAILED)
    }
    free(box.pixels)
    box.pixels = calloc(1, count)
    box.pixelCount = count
    guard let pixels = box.pixels,
          let colors = CGColorSpace(name: CGColorSpace.sRGB),
          let bitmap = CGContext(
            data: pixels, width: width, height: height, bitsPerComponent: 8,
            bytesPerRow: rowBytes, space: colors,
            bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue |
                CGBitmapInfo.byteOrder32Big.rawValue
          ) else {
        return UInt32(NUX_ASSET_CALLBACK_STATUS_FAILED)
    }
    bitmap.draw(image, in: CGRect(x: 0, y: 0, width: width, height: height))
    box.calls += 1
    var retained = NuxRetainedBytes()
    retained.struct_size = UInt32(MemoryLayout<NuxRetainedBytes>.size)
    retained.data = UnsafePointer(pixels.assumingMemoryBound(to: UInt8.self))
    retained.len = count
    retained.owner = context
    retained.retain = retainPixels
    retained.release = releasePixels
    var decoded = NuxDecodedImage()
    decoded.struct_size = UInt32(MemoryLayout<NuxDecodedImage>.size)
    decoded.width = UInt32(width)
    decoded.height = UInt32(height)
    decoded.row_bytes = UInt32(rowBytes)
    decoded.pixel_format = UInt32(NUX_PIXEL_FORMAT_RGBA8_PREMULTIPLIED_SRGB)
    decoded.pixels = retained
    outImage.pointee = decoded
    return UInt32(NUX_ASSET_CALLBACK_STATUS_OK)
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

func stringView(_ pointer: UnsafePointer<CChar>, count: Int) -> NuxStringView {
    var view = NuxStringView()
    view.data = pointer
    view.len = count
    return view
}

func copiedString(_ view: NuxStringView) -> String {
    guard let data = view.data else { return "" }
    return String(
        decoding: UnsafeBufferPointer(
            start: UnsafeRawPointer(data).assumingMemoryBound(to: UInt8.self),
            count: view.len
        ),
        as: UTF8.self
    )
}

func importConfigured(
    bytes: Data,
    hooks: inout NuxAppleAssetHooks,
    composed: Bool,
    file: inout OpaquePointer?,
    result: inout OpaquePointer?
) -> UInt32 {
    func call(
        _ config: inout NuxFileImportConfig
    ) -> UInt32 {
        bytes.withUnsafeBytes { rawBytes in
            nux_file_import_configured(
                rawBytes.bindMemory(to: UInt8.self).baseAddress,
                rawBytes.count,
                &config,
                &file,
                &result
            )
        }
    }

    guard composed else {
        return withUnsafePointer(to: &hooks) { hooksPointer in
            var config = NuxFileImportConfig()
            config.struct_size = UInt32(MemoryLayout<NuxFileImportConfig>.size)
            config.apple_assets = hooksPointer
            return call(&config)
        }
    }
    return "bridge".withCString { moduleName in
        "GenericHostChanges".withCString { scriptName in
            "lua".withCString { luaExtension in
                "pixel.png".withCString { imageName in
                    "png".withCString { pngExtension in
                        var host = NuxHostCommandImportConfig()
                        host.struct_size = UInt32(
                            MemoryLayout<NuxHostCommandImportConfig>.size
                        )
                        host.module_name = stringView(moduleName, count: 6)
                        host.max_script_memory_bytes = 64 * 1024 * 1024
                        host.max_script_interrupts_per_callback = 50_000
                        host.max_commands_per_step = 256
                        host.max_value_depth = 32
                        host.max_value_nodes = 4_096
                        host.max_identifier_bytes = 4_096
                        host.max_string_bytes = 1024 * 1024
                        host.max_value_bytes = 4 * 1024 * 1024
                        host.max_command_bytes_per_step = 4 * 1024 * 1024

                        var script = NuxExpectedFileAssetDescriptor()
                        script.struct_size = UInt32(
                            MemoryLayout<NuxExpectedFileAssetDescriptor>.size
                        )
                        script.ordinal = 0
                        script.kind = NUX_FILE_ASSET_KIND_SCRIPT.rawValue
                        script.has_authored_id = 1
                        script.authored_id = 0
                        script.name = stringView(scriptName, count: 18)
                        script.file_extension = stringView(luaExtension, count: 3)
                        script.is_embedded = 1
                        script.has_contents_record = 1

                        var image = NuxExpectedFileAssetDescriptor()
                        image.struct_size = UInt32(
                            MemoryLayout<NuxExpectedFileAssetDescriptor>.size
                        )
                        image.ordinal = 1
                        image.kind = NUX_FILE_ASSET_KIND_IMAGE.rawValue
                        image.has_authored_id = 1
                        image.authored_id = 7
                        image.name = stringView(imageName, count: 9)
                        image.file_extension = stringView(pngExtension, count: 3)
                        image.is_embedded = 1
                        image.has_contents_record = 1
                        image.required_provider_flags = UInt32(
                            NUX_FILE_ASSET_PROVIDER_IMAGE_DECODE
                        )
                        let expected = [script, image]
                        return expected.withUnsafeBufferPointer { expectedBuffer in
                            withUnsafePointer(to: &host) { hostPointer in
                                withUnsafePointer(to: &hooks) { hooksPointer in
                                    var config = NuxFileImportConfig()
                                    config.struct_size = UInt32(
                                        MemoryLayout<NuxFileImportConfig>.size
                                    )
                                    config.host_commands = hostPointer
                                    config.apple_assets = hooksPointer
                                    config.expected_assets = expectedBuffer.baseAddress
                                    config.expected_asset_count = expectedBuffer.count
                                    return call(&config)
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

guard CommandLine.arguments.count == 2 ||
        (CommandLine.arguments.count == 3 && CommandLine.arguments[2] == "--composed") else {
    FileHandle.standardError.write(Data("usage: capi_metal_smoke <file.riv> [--composed]\n".utf8))
    exit(2)
}

let composed = CommandLine.arguments.count == 3

let bytes = try Data(contentsOf: URL(fileURLWithPath: CommandLine.arguments[1]))
var file: OpaquePointer?
let decoder = DecodeBox()
var hooks = NuxAppleAssetHooks()
hooks.struct_size = UInt32(MemoryLayout<NuxAppleAssetHooks>.size)
hooks.context = Unmanaged.passUnretained(decoder).toOpaque()
hooks.decode_image = decodeImage
hooks.maximum_external_asset_bytes = 64 * 1024 * 1024
hooks.maximum_total_external_asset_bytes = 256 * 1024 * 1024
hooks.maximum_image_dimension = 8192
hooks.maximum_decoded_image_bytes = 256 * 1024 * 1024
hooks.maximum_total_decoded_image_bytes = 512 * 1024 * 1024
var result: OpaquePointer?
check(
    importConfigured(
        bytes: bytes,
        hooks: &hooks,
        composed: composed,
        file: &file,
        result: &result
    ) == NUX_STATUS_OK.rawValue,
    "import"
)
freeResult(result, NUX_STATUS_OK.rawValue)
check(decoder.calls == 1, "one image decode")
check(decoder.retains == 1 && decoder.releases == 1, "balanced decoded pixels")
check(decoder.nestedABI == 0, "callback reentry rejected")
var assetCount = 0
check(nux_file_asset_count(file, &assetCount) == NUX_STATUS_OK.rawValue, "asset count")
check(assetCount == (composed ? 2 : 1), "file asset catalog")
var asset = NuxFileAssetDescriptorView()
asset.struct_size = UInt32(MemoryLayout<NuxFileAssetDescriptorView>.size)
check(nux_file_asset_descriptor(file, 0, &asset) == NUX_STATUS_OK.rawValue, "asset descriptor")
check(
    asset.kind == (composed ? NUX_FILE_ASSET_KIND_SCRIPT.rawValue :
                              NUX_FILE_ASSET_KIND_IMAGE.rawValue),
    "first catalog kind"
)
if composed {
    check(asset.required_provider_flags == 0, "script provider requirements")
    asset.struct_size = UInt32(MemoryLayout<NuxFileAssetDescriptorView>.size)
    check(nux_file_asset_descriptor(file, 1, &asset) == NUX_STATUS_OK.rawValue, "image descriptor")
    check(asset.kind == NUX_FILE_ASSET_KIND_IMAGE.rawValue, "image catalog kind")
}
check(
    asset.required_provider_flags == UInt32(NUX_FILE_ASSET_PROVIDER_IMAGE_DECODE),
    "image decode provider requirement"
)
var artboard: OpaquePointer?
check(nux_artboard_instance_new(file, 0, &artboard) == NUX_STATUS_OK.rawValue, "artboard")
var viewModel: OpaquePointer?
if composed {
    check(
        nux_view_model_instance_new_authored(file, 0, 0, &viewModel) == NUX_STATUS_OK.rawValue,
        "authored view model"
    )
    check(
        nux_artboard_instance_bind_view_model(artboard, viewModel) == NUX_STATUS_OK.rawValue,
        "bind view model"
    )
}
var player: OpaquePointer?
if composed {
    check(
        "HostCommands".withCString { name in
            let view = stringView(name, count: 12)
            return nux_player_new_state_machine_named(artboard, view, &player)
        } == NUX_STATUS_OK.rawValue,
        "state-machine player"
    )
} else {
    check(nux_player_new_static(artboard, &player) == NUX_STATUS_OK.rawValue, "player")
}
check(nux_file_free(file) == NUX_STATUS_OK.rawValue, "release file before player")
check(nux_artboard_instance_free(artboard) == NUX_STATUS_OK.rawValue, "release artboard before player")

var renderer: OpaquePointer?
result = nil
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

if composed {
    var mutationResult: OpaquePointer?
    let mutationStatus = "amount".withCString { path in
        var mutation = NuxViewModelMutation()
        mutation.kind = NUX_VIEW_MODEL_MUTATION_KIND_SET_NUMBER.rawValue
        mutation.instance = viewModel
        mutation.path = stringView(path, count: 6)
        mutation.number_value = 5
        return withUnsafePointer(to: &mutation) { mutationPointer in
            var batch = NuxViewModelMutationBatch()
            batch.struct_size = UInt32(MemoryLayout<NuxViewModelMutationBatch>.size)
            batch.mutations = mutationPointer
            batch.mutation_count = 1
            batch.correlation_id = 43
            return nux_view_model_mutate(&batch, &mutationResult)
        }
    }
    check(mutationStatus == NUX_STATUS_OK.rawValue, "caller mutation")
    var mutationInfo = NuxViewModelMutationResultInfo()
    mutationInfo.struct_size = UInt32(MemoryLayout<NuxViewModelMutationResultInfo>.size)
    check(
        nux_view_model_mutation_result_info(mutationResult, &mutationInfo) == NUX_STATUS_OK.rawValue,
        "mutation info"
    )
    check(mutationInfo.applied_count == 1, "one applied caller mutation")
    check(mutationInfo.correlation_id == 43, "caller correlation")
    check(mutationInfo.change_count == 1, "one caller journal entry")
    var callerChange = NuxViewModelChangeView()
    callerChange.struct_size = UInt32(MemoryLayout<NuxViewModelChangeView>.size)
    check(
        nux_view_model_mutation_result_change(mutationResult, 0, &callerChange) == NUX_STATUS_OK.rawValue,
        "caller journal"
    )
    check(callerChange.origin == NUX_VIEW_MODEL_CHANGE_ORIGIN_CALLER.rawValue, "caller origin")
    check(callerChange.correlation_id == 43, "caller journal correlation")
    check(callerChange.kind == NUX_VIEW_MODEL_VALUE_KIND_NUMBER.rawValue, "caller number kind")
    check(callerChange.number_value == 5, "caller number value")
    check(
        nux_view_model_mutation_result_free(mutationResult) == NUX_STATUS_OK.rawValue,
        "free mutation journal"
    )

    var initial = NuxPlayerStep()
    initial.struct_size = UInt32(MemoryLayout<NuxPlayerStep>.size)
    var stepResult: OpaquePointer?
    check(nux_player_step(player, &initial, &stepResult) == NUX_STATUS_OK.rawValue, "initialize player")
    check(nux_player_step_result_free(stepResult) == NUX_STATUS_OK.rawValue, "free initial step")

    var down = NuxPlayerPointerEvent()
    down.kind = NUX_PLAYER_POINTER_KIND_DOWN.rawValue
    down.x = 50
    down.y = 50
    var up = NuxPlayerPointerEvent()
    up.kind = NUX_PLAYER_POINTER_KIND_UP.rawValue
    up.x = 50
    up.y = 50
    let pointers = [down, up]
    stepResult = nil
    let stepStatus = pointers.withUnsafeBufferPointer { pointerBuffer in
        var step = NuxPlayerStep()
        step.struct_size = UInt32(MemoryLayout<NuxPlayerStep>.size)
        step.pointers = pointerBuffer.baseAddress
        step.pointer_count = pointerBuffer.count
        step.elapsed_seconds = 0.016
        step.correlation_id = 44
        return nux_player_step(player, &step, &stepResult)
    }
    check(stepStatus == NUX_STATUS_OK.rawValue, "scripted player step")
    var stepInfo = NuxPlayerStepInfo()
    stepInfo.struct_size = UInt32(MemoryLayout<NuxPlayerStepInfo>.size)
    check(
        nux_player_step_result_info(stepResult, &stepInfo) == NUX_STATUS_OK.rawValue,
        "step info"
    )
    check(stepInfo.host_command_count == 1, "one host command")
    check(stepInfo.view_model_change_count == 2, "two ordered runtime changes")
    var command = NuxHostCommandView()
    command.struct_size = UInt32(MemoryLayout<NuxHostCommandView>.size)
    check(
        nux_player_step_result_host_command(stepResult, 0, &command) == NUX_STATUS_OK.rawValue,
        "host command"
    )
    check(copiedString(command.name) == "performed", "host command name")
    for (index, expected) in [Float(10), Float(20)].enumerated() {
        var change = NuxViewModelChangeView()
        change.struct_size = UInt32(MemoryLayout<NuxViewModelChangeView>.size)
        check(
            nux_player_step_result_view_model_change(stepResult, index, &change) == NUX_STATUS_OK.rawValue,
            "runtime journal \(index)"
        )
        check(change.origin == NUX_VIEW_MODEL_CHANGE_ORIGIN_RUNTIME.rawValue, "runtime origin")
        check(change.correlation_id == 44, "runtime correlation")
        check(change.kind == NUX_VIEW_MODEL_VALUE_KIND_NUMBER.rawValue, "runtime number kind")
        check(change.number_value == expected, "ordered runtime number \(index)")
    }
    check(nux_player_step_result_free(stepResult) == NUX_STATUS_OK.rawValue, "free scripted step")
}

check(nux_renderer_free(renderer) == NUX_STATUS_OK.rawValue, "free renderer")
check(nux_player_free(player) == NUX_STATUS_OK.rawValue, "free player")
check(nux_view_model_instance_free(viewModel) == NUX_STATUS_OK.rawValue, "free view model")
check(decoder.calls == 1, "render does not call host decoder again")
print("swift-metal-smoke ok")
