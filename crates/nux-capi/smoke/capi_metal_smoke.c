/* Product-neutral Apple renderer C ABI smoke. The C program uses only the C
 * ABI (including the Objective-C C runtime) to configure a CAMetalLayer,
 * render a retained drawable, and exercise the complete handle lifecycle.
 *
 * Usage: capi_metal_smoke <path-to-in_band_asset.riv>
 */

#include "nux_capi_apple.h"

#include <CoreGraphics/CoreGraphics.h>
#include <ImageIO/ImageIO.h>
#include <dispatch/dispatch.h>
#include <objc/message.h>
#include <objc/runtime.h>
#include <stdio.h>
#include <stdatomic.h>
#include <stdlib.h>

/* MTLPixelFormatBGRA8Unorm's stable Objective-C ABI value. Metal's public
 * header is Objective-C-only, while this host intentionally remains C11. */
#define MTL_PIXEL_FORMAT_BGRA8_UNORM 80UL

#define CHECK(condition)                                                      \
    do                                                                        \
    {                                                                         \
        if (!(condition))                                                     \
        {                                                                     \
            fprintf(stderr, "capi-metal-smoke FAILED at %s:%d: %s\n",        \
                    __FILE__, __LINE__, #condition);                          \
            exit(1);                                                          \
        }                                                                     \
    } while (0)

static uint8_t* read_file(const char* path, size_t* out_len)
{
    FILE* file = fopen(path, "rb");
    CHECK(file != NULL);
    CHECK(fseek(file, 0, SEEK_END) == 0);
    long size = ftell(file);
    CHECK(size > 0);
    CHECK(fseek(file, 0, SEEK_SET) == 0);
    uint8_t* bytes = malloc((size_t)size);
    CHECK(bytes != NULL);
    CHECK(fread(bytes, 1, (size_t)size, file) == (size_t)size);
    CHECK(fclose(file) == 0);
    *out_len = (size_t)size;
    return bytes;
}

static void free_result(NuxCapiResult* result, NuxStatus expected)
{
    CHECK(result != NULL);
    NuxStatus status = NUX_STATUS_RUNTIME_ERROR;
    CHECK(nux_capi_result_status(result, &status) == NUX_STATUS_OK);
    CHECK(status == expected);
    CHECK(nux_capi_result_free(result) == NUX_STATUS_OK);
}

typedef void* ObjcObject;

typedef struct DecodeProbe
{
    uint8_t* pixels;
    size_t pixel_len;
    unsigned calls;
    unsigned retains;
    unsigned releases;
    uint32_t nested_abi;
} DecodeProbe;

static void retain_pixels(void* owner)
{
    DecodeProbe* probe = owner;
    probe->retains += 1;
    probe->nested_abi = nux_capi_abi_version();
}

static void release_pixels(void* owner)
{
    DecodeProbe* probe = owner;
    probe->releases += 1;
}

static NuxAssetCallbackStatus decode_image(void* context,
                                           const NuxImageDecodeRequest* request,
                                           NuxDecodedImage* out_image)
{
    DecodeProbe* probe = context;
    CHECK(request != NULL);
    CHECK(request->struct_size >= NUX_IMAGE_DECODE_REQUEST_V3_MIN_SIZE);
    CFDataRef encoded = CFDataCreateWithBytesNoCopy(
        kCFAllocatorDefault, request->encoded.data, (CFIndex)request->encoded.len,
        kCFAllocatorNull);
    CHECK(encoded != NULL);
    CGImageSourceRef source = CGImageSourceCreateWithData(encoded, NULL);
    CHECK(source != NULL);
    CGImageRef image = CGImageSourceCreateImageAtIndex(source, 0, NULL);
    CHECK(image != NULL);
    size_t width = CGImageGetWidth(image);
    size_t height = CGImageGetHeight(image);
    CHECK(width > 0 && height > 0);
    CHECK(width <= request->maximum_dimension &&
          height <= request->maximum_dimension);
    CHECK(width <= UINT32_MAX / 4);
    size_t row_bytes = width * 4;
    CHECK(height <= SIZE_MAX / row_bytes);
    size_t pixel_len = row_bytes * height;
    CHECK(pixel_len <= request->maximum_decoded_bytes);
    free(probe->pixels);
    probe->pixels = calloc(1, pixel_len);
    CHECK(probe->pixels != NULL);
    probe->pixel_len = pixel_len;
    CGColorSpaceRef colors = CGColorSpaceCreateWithName(kCGColorSpaceSRGB);
    CHECK(colors != NULL);
    CGContextRef bitmap = CGBitmapContextCreate(
        probe->pixels, width, height, 8, row_bytes, colors,
        kCGImageAlphaPremultipliedLast | kCGBitmapByteOrder32Big);
    CHECK(bitmap != NULL);
    CGContextDrawImage(bitmap, CGRectMake(0, 0, (CGFloat)width, (CGFloat)height), image);
    CGContextRelease(bitmap);
    CGColorSpaceRelease(colors);
    CGImageRelease(image);
    CFRelease(source);
    CFRelease(encoded);
    probe->calls += 1;
    *out_image = (NuxDecodedImage){
        .struct_size = sizeof(NuxDecodedImage),
        .width = (uint32_t)width,
        .height = (uint32_t)height,
        .row_bytes = (uint32_t)row_bytes,
        .pixel_format = NUX_PIXEL_FORMAT_RGBA8_PREMULTIPLIED_SRGB,
        .pixels = {
            .struct_size = sizeof(NuxRetainedBytes),
            .data = probe->pixels,
            .len = probe->pixel_len,
            .owner = probe,
            .retain = retain_pixels,
            .release = release_pixels,
        },
    };
    return NUX_ASSET_CALLBACK_STATUS_OK;
}

static ObjcObject send_object(ObjcObject receiver, const char* selector)
{
    return ((ObjcObject(*)(ObjcObject, SEL))objc_msgSend)(
        receiver, sel_registerName(selector));
}

static void send_object_argument(ObjcObject receiver,
                                 const char* selector,
                                 ObjcObject argument)
{
    ((void (*)(ObjcObject, SEL, ObjcObject))objc_msgSend)(
        receiver, sel_registerName(selector), argument);
}

static void send_unsigned_argument(ObjcObject receiver,
                                   const char* selector,
                                   unsigned long argument)
{
    ((void (*)(ObjcObject, SEL, unsigned long))objc_msgSend)(
        receiver, sel_registerName(selector), argument);
}

static void send_bool_argument(ObjcObject receiver,
                               const char* selector,
                               signed char argument)
{
    ((void (*)(ObjcObject, SEL, signed char))objc_msgSend)(
        receiver, sel_registerName(selector), argument);
}

static void send_size_argument(ObjcObject receiver,
                               const char* selector,
                               CGSize argument)
{
    ((void (*)(ObjcObject, SEL, CGSize))objc_msgSend)(
        receiver, sel_registerName(selector), argument);
}

static void send_void(ObjcObject receiver, const char* selector)
{
    ((void (*)(ObjcObject, SEL))objc_msgSend)(
        receiver, sel_registerName(selector));
}

typedef struct CompletionProbe
{
    dispatch_semaphore_t semaphore;
    atomic_bool call_is_active;
    atomic_bool called_inline;
    atomic_uint calls;
} CompletionProbe;

static void render_completed(void* context)
{
    CompletionProbe* probe = context;
    if (atomic_load_explicit(&probe->call_is_active, memory_order_acquire))
    {
        atomic_store_explicit(&probe->called_inline, true, memory_order_release);
    }
    atomic_fetch_add_explicit(&probe->calls, 1, memory_order_release);
    dispatch_semaphore_signal(probe->semaphore);
}

int main(int argc, char** argv)
{
    CHECK(argc == 2);
    size_t len = 0;
    uint8_t* bytes = read_file(argv[1], &len);
    DecodeProbe decoder = {0};
    NuxAppleAssetHooks hooks = {
        .struct_size = sizeof(NuxAppleAssetHooks),
        .context = &decoder,
        .decode_image = decode_image,
        .maximum_external_asset_bytes = 64 * 1024 * 1024,
        .maximum_total_external_asset_bytes = 256 * 1024 * 1024,
        .maximum_image_dimension = 8192,
        .maximum_decoded_image_bytes = 256 * 1024 * 1024,
        .maximum_total_decoded_image_bytes = 512 * 1024 * 1024,
    };
    NuxFileImportConfig import_config = {
        .struct_size = sizeof(NuxFileImportConfig),
        .apple_assets = &hooks,
    };
    NuxFile* file = NULL;
    NuxCapiResult* result = NULL;
    CHECK(nux_file_import_configured(
              bytes, len, &import_config, &file, &result) == NUX_STATUS_OK);
    free_result(result, NUX_STATUS_OK);
    CHECK(decoder.calls == 1);
    CHECK(decoder.retains == 1);
    CHECK(decoder.releases == 1);
    CHECK(decoder.nested_abi == 0);
    size_t asset_count = 0;
    CHECK(nux_file_asset_count(file, &asset_count) == NUX_STATUS_OK);
    CHECK(asset_count == 1);
    NuxFileAssetDescriptorView asset = {
        .struct_size = sizeof(NuxFileAssetDescriptorView)};
    CHECK(nux_file_asset_descriptor(file, 0, &asset) == NUX_STATUS_OK);
    CHECK(asset.kind == NUX_FILE_ASSET_KIND_IMAGE);
    CHECK(asset.required_provider_flags == NUX_FILE_ASSET_PROVIDER_IMAGE_DECODE);
    free(bytes);
    NuxArtboardInstance* artboard = NULL;
    CHECK(nux_artboard_instance_new(file, 0, &artboard) == NUX_STATUS_OK);
    NuxPlayer* player = NULL;
    CHECK(nux_player_new_static(artboard, &player) == NUX_STATUS_OK);
    CHECK(nux_file_free(file) == NUX_STATUS_OK);
    CHECK(nux_artboard_instance_free(artboard) == NUX_STATUS_OK);

    NuxRenderer* renderer = NULL;
    result = NULL;
    CHECK(nux_renderer_new_metal(4, 3, &renderer, &result) == NUX_STATUS_OK);
    free_result(result, NUX_STATUS_OK);

    void* device = NULL;
    result = NULL;
    CHECK(nux_renderer_copy_metal_device(renderer, &device, &result) ==
          NUX_STATUS_OK);
    free_result(result, NUX_STATUS_OK);
    CHECK(device != NULL);

    ObjcObject layer_class = (ObjcObject)objc_getClass("CAMetalLayer");
    CHECK(layer_class != NULL);
    ObjcObject layer = send_object(layer_class, "layer");
    CHECK(layer != NULL);
    send_object_argument(layer, "setDevice:", device);
    send_unsigned_argument(
        layer, "setPixelFormat:", MTL_PIXEL_FORMAT_BGRA8_UNORM);
    send_bool_argument(layer, "setFramebufferOnly:", 1);
    send_size_argument(layer, "setDrawableSize:", CGSizeMake(4.0, 3.0));
    send_unsigned_argument(layer, "setMaximumDrawableCount:", 2);
    send_bool_argument(layer, "setAllowsNextDrawableTimeout:", 1);
    send_void(device, "release");

    ObjcObject drawable = send_object(layer, "nextDrawable");
    CHECK(drawable != NULL);
    drawable = send_object(drawable, "retain");
    CHECK(drawable != NULL);
    CompletionProbe completion = {
        .semaphore = dispatch_semaphore_create(0),
        .call_is_active = ATOMIC_VAR_INIT(false),
        .called_inline = ATOMIC_VAR_INIT(false),
        .calls = ATOMIC_VAR_INIT(0),
    };
    CHECK(completion.semaphore != NULL);
    NuxMetalRenderOperation operation = {
        .struct_size = sizeof(NuxMetalRenderOperation),
        .drawable_state = NUX_METAL_DRAWABLE_STATE_AVAILABLE,
        .drawable = drawable,
        .clear_color = 0xff112233,
        .completion_context = &completion,
        .completion_callback = render_completed,
    };
    NuxRendererOutcome outcome = {.struct_size = sizeof(NuxRendererOutcome)};
    result = (NuxCapiResult*)1;
    atomic_store_explicit(
        &completion.call_is_active, true, memory_order_release);
    CHECK(nux_renderer_render_player(
              renderer, player, &operation, &outcome, &result) == NUX_STATUS_OK);
    atomic_store_explicit(
        &completion.call_is_active, false, memory_order_release);
    send_void(drawable, "release");
    CHECK(result == NULL);
    CHECK(outcome.disposition == NUX_RENDERER_DISPOSITION_PRESENTED);
    CHECK(dispatch_semaphore_wait(
              completion.semaphore,
              dispatch_time(DISPATCH_TIME_NOW, 5 * NSEC_PER_SEC)) == 0);
    CHECK(atomic_load_explicit(&completion.calls, memory_order_acquire) == 1);
    CHECK(!atomic_load_explicit(
        &completion.called_inline, memory_order_acquire));

    result = NULL;
    CHECK(nux_renderer_resize(renderer, 0, 0, &outcome, &result) == NUX_STATUS_OK);
    free_result(result, NUX_STATUS_OK);
    CHECK(outcome.disposition == NUX_RENDERER_DISPOSITION_SKIPPED_ZERO_SIZE);
    result = NULL;
    CHECK(nux_renderer_detach(renderer, &outcome, &result) == NUX_STATUS_OK);
    free_result(result, NUX_STATUS_OK);
    result = NULL;
    CHECK(nux_renderer_reattach(renderer, 4, 3, &outcome, &result) == NUX_STATUS_OK);
    free_result(result, NUX_STATUS_OK);
    CHECK(outcome.disposition == NUX_RENDERER_DISPOSITION_RECREATED);

    CHECK(nux_renderer_free(renderer) == NUX_STATUS_OK);
    CHECK(nux_player_free(player) == NUX_STATUS_OK);
    CHECK(decoder.calls == 1);
    free(decoder.pixels);
    puts("capi-metal-smoke ok");
    return 0;
}
