import Foundation
import CoreGraphics
import ImageIO
import Metal
import QuartzCore
import NuxieRuntimeInternal

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

guard CommandLine.arguments.count == 2 else {
    FileHandle.standardError.write(Data("usage: capi_metal_smoke <in_band_asset.riv>\n".utf8))
    exit(2)
}

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
check(bytes.withUnsafeBytes {
    nux_file_import_with_apple_assets(
        $0.bindMemory(to: UInt8.self).baseAddress, $0.count, &hooks, &file, &result)
} == NUX_STATUS_OK.rawValue, "import")
freeResult(result, NUX_STATUS_OK.rawValue)
check(decoder.calls == 1, "one image decode")
check(decoder.retains == 1 && decoder.releases == 1, "balanced decoded pixels")
check(decoder.nestedABI == 0, "callback reentry rejected")
var artboard: OpaquePointer?
check(nux_artboard_instance_new(file, 0, &artboard) == NUX_STATUS_OK.rawValue, "artboard")
var player: OpaquePointer?
check(nux_player_new_static(artboard, &player) == NUX_STATUS_OK.rawValue, "player")
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

check(nux_renderer_free(renderer) == NUX_STATUS_OK.rawValue, "free renderer")
check(nux_player_free(player) == NUX_STATUS_OK.rawValue, "free player")
check(decoder.calls == 1, "render does not call host decoder again")
print("swift-metal-smoke ok")
