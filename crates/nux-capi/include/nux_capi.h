#ifndef NUX_CAPI_H
#define NUX_CAPI_H

/* Public umbrella header for the Nuxie runtime C ABI.
 *
 * Declarations live in nux_capi.generated.h, which is generated from the
 * Rust exports by cbindgen and byte-verified during every nux-capi build.
 *
 * LIFETIME AND OWNERSHIP CONTRACT
 *
 * SAFETY PRECONDITIONS
 *
 * Every non-NULL pointer must be valid and correctly aligned for its declared
 * pointee and for the complete synchronous call. Writable outputs must point
 * to initialized storage of the declared size, must not overlap each other or
 * live input storage, and borrowed byte/string ranges must remain readable for
 * their declared length. Violating these C pointer preconditions is undefined
 * caller behavior; status codes validate semantic errors, not arbitrary memory.
 *
 * 1. Every non-NULL handle is owned by the caller and must be passed exactly
 *    once to its matching `_free` function. Every `_free(NULL)` returns OK.
 *    Once `_free` passes validation and begins destruction it consumes the
 *    handle, including when a contained destructor panic returns RUNTIME_ERROR.
 *    WRONG_THREAD, REENTRANT_CALL, and HANDLE_MISMATCH do not consume it.
 *    Every later use of a consumed pointer (including a second free) is outside
 *    this API's contract. Registry validation is only a best-effort diagnostic
 *    and is not an ABA-safe stale-pointer capability.
 * 2. File, artboard, player, state-machine, view-model, and renderer handles may be
 *    released in any order. An artboard occurrence retains its imported file;
 *    a player retains its artboard occurrence and renderer binding. A legacy
 *    state-machine operation that needs an artboard still requires the exact
 *    originating live artboard handle and reports HANDLE_MISMATCH otherwise.
 * 3. NuxStringView values borrow their documented owner unless their field is
 *    explicitly documented as process-static. Copy the bytes when a longer
 *    lifetime is needed; views are not NUL-terminated.
 * 4. Every handle is affine to its creation thread. Use or free from another
 *    thread returns WRONG_THREAD without consuming the handle. Concurrent or
 *    callback-time reentrant use returns REENTRANT_CALL without consuming it.
 * 5. NuxViewModelInstance retains its imported file and shared runtime graph.
 *    A handle returned by `_share` has the same stable identity and observes
 *    the same values. Generic file-level instances may bind to a compatible
 *    artboard from that file; legacy artboard-created instances retain their
 *    exact-occurrence restriction.
 * 6. The first accepted draw attempt binds one NuxRenderCallbacks descriptor
 *    address to the occurrence, including when drawing later returns an error;
 *    later draws must pass that same descriptor address.
 *    Its callback functions and user_data must remain valid until the last
 *    artboard or player retaining that occurrence is freed. Calling back into
 *    that active occurrence from a renderer callback returns REENTRANT_CALL.
 * 7. Every caller-sized ABI-v3 struct must set struct_size. A size smaller than
 *    its documented NUX_*_V3_MIN_SIZE is rejected; additive tail bytes from a
 *    larger caller are ignored and preserved.
 * 8. NuxCapiResult owns its bounded diagnostic bytes. Diagnostic views borrow
 *    that result and expire when nux_capi_result_free succeeds.
 * 9. NuxPlayerStep input/pointer arrays and their input-name bytes are borrowed
 *    only for the synchronous call. NuxPlayerStepResult owns every returned
 *    event, property, state-change, pointer outcome, host command, flattened
 *    host value, and diagnostic. Indexed views expire when
 *    nux_player_step_result_free succeeds. Copy any command/value bytes before
 *    freeing the result or handing them to another thread or asynchronous
 *    task. This includes every view-model change and its bytes_value/list-item
 *    views. Optional string fields use NULL+0 for absent and non-NULL+0 for
 *    authored present-empty.
 * 10. nux_player_step fully validates the bounded batch before mutation and
 *    executes under the shared artboard-occurrence gate. Reentrant access from
 *    any callback returns REENTRANT_CALL. An unexpected post-mutation failure
 *    rolls back pending external host effects and terminally poisons that
 *    occurrence; every later read, mutate, or draw fails with RUNTIME_ERROR,
 *    while matching frees remain allowed. On success, state-machine inputs and
 *    pointer events are applied in array order, pointer-authored events are
 *    copied before advancement-authored events, and host commands become
 *    observable only after commit in their script FIFO. Hosts that combine
 *    the separate arrays deliver authored events before host commands.
 * 11. nux_file_import_trusted_with_host_commands is an explicit trust boundary
 *    for the exact byte range passed to that call. Its caller-sized config and
 *    module name are copied synchronously. It installs no foreign callback;
 *    scripts only enqueue bounded owned values for the active player step.
 *    Ordinary nux_file_import remains script-inert.
 * 12. On Apple, NuxRenderer owns the wgpu/Metal device domain; it never owns a
 *    CAMetalLayer or acquires a drawable. nux_renderer_copy_metal_device gives
 *    the caller Objective-C +1 ownership. A non-NULL AVAILABLE drawable is
 *    borrowed only for the synchronous render call. TIMEOUT and OCCLUDED are
 *    explicit caller-reported states and require a NULL drawable. A valid
 *    completion pair is consumed before later validation and runs exactly once
 *    on a system dispatch queue, never inline. A too-short operation prefix
 *    cannot transfer a callback that the runtime cannot safely read.
 * 13. The first drawable-backed Metal draw binds the retained artboard
 *    occurrence to that renderer's durable domain and current generation.
 *    Timeout, occlusion, and zero-size skips do not bind it. Another renderer,
 *    or a reattached generation, returns HANDLE_MISMATCH until
 *    nux_renderer_reset_player_domain succeeds on a healthy attached renderer.
 *    The player retains that binding after the public renderer handle is freed.
 * 14. nux_renderer_render_player treats out_result as optional and failure-only:
 *    a supplied slot is cleared on entry, remains NULL on success, and owns one
 *    bounded diagnostic on failure. Renderer control APIs require out_result
 *    and publish one result on every outcome.
 * 15. View-model catalogs and snapshots own all returned bytes and flat
 *    tables; indexed views remain valid until their catalog/snapshot is freed.
 *    Mutation input arrays and byte/string views are borrowed only for the
 *    synchronous call. Mutation-result code/message views borrow that result
 *    and expire when nux_view_model_mutation_result_free succeeds. Each
 *    mutation-result change view and its bytes_value/list-item views borrow
 *    that same result and expire with it. A failed batch reports
 *    applied_count=0 and leaves the live view-model graph
 *    observationally unchanged: exact retained cells/topology are restored
 *    and buffered dirt, listener, and binding notifications are discarded.
 *    Observer callbacks run only during final publication and are individually
 *    panic-isolated; a panicking observer falls back to ordinary dirt delivery
 *    and does not change a successfully committed batch into a failure.
 * 16. Apple asset hooks are borrowed only for the synchronous import call.
 *    Request byte/string views expire when their callback returns and callbacks
 *    must not call back into any nux-capi export. Every successful callback
 *    output whose struct_size covers a complete retain/release pair transfers
 *    ownership: the runtime calls retain exactly once before inspecting the
 *    buffer and release exactly once after copying or rejecting it.
 *    Failure/unknown outcomes with a complete readable pair are balanced the
 *    same way; short prefixes and incomplete pairs transfer no ownership and
 *    neither function is called. Zero-length data may be NULL. Rust preflights
 *    encoded dimensions and supplies the exact remaining per-item/aggregate
 *    decode ceiling before invoking the host. Decoded images must be RGBA8
 *    premultiplied-sRGB within every advertised bound.
 *    Canonical CPU pixels remain file/occurrence-owned across renderer resets;
 *    only renderer-domain GPU resources are invalidated and recreated.
 *
 * PANIC SAFETY
 *
 * Every exported entry point has a panic firewall. An unwind never crosses
 * this ABI. Status-returning calls report NUX_STATUS_RUNTIME_ERROR; scalar or
 * void calls return their documented safe fallback. A panic poisons an active
 * handle/occurrence; later operations fail with RUNTIME_ERROR, but its matching
 * free remains permitted on the creation thread unless that panic occurred
 * during destruction itself, in which case destruction already consumed it.
 * The narrow exception is a runtime panic while staging or publishing the
 * result of a transactional view-model/text-run batch: its runtime-owned RAII
 * transaction restores the exact graph silently, returns RUNTIME_ERROR (and
 * applied_count=0 when a result exists), and leaves the otherwise healthy
 * handle usable. Panics from observer callbacks are contained as described
 * above and therefore do not fail or poison the committed batch.
 */

#include "nux_capi.generated.h"

/* ABI-v3 prefixes. These use the last field that belongs to v3 so future
 * generated headers may append fields without changing the accepted prefix. */
#define NUX_RUNTIME_INFO_V3_MIN_SIZE                                      \
    (offsetof(NuxRuntimeInfo, source_revision) +                          \
     sizeof(((NuxRuntimeInfo*)0)->source_revision))
#define NUX_PLAYER_INFO_V3_MIN_SIZE                                       \
    (offsetof(NuxPlayerInfo, name) + sizeof(((NuxPlayerInfo*)0)->name))
#define NUX_CAPI_DIAGNOSTIC_VIEW_V3_MIN_SIZE                              \
    (offsetof(NuxCapiDiagnosticView, message) +                           \
     sizeof(((NuxCapiDiagnosticView*)0)->message))
#define NUX_FILE_ASSET_DESCRIPTOR_VIEW_V3_MIN_SIZE                        \
    (offsetof(NuxFileAssetDescriptorView, required_provider_flags) +      \
     sizeof(((NuxFileAssetDescriptorView*)0)->required_provider_flags))
#define NUX_RENDER_CALLBACKS_V3_MIN_SIZE                                  \
    (offsetof(NuxRenderCallbacks, modulate_opacity) +                     \
     sizeof(((NuxRenderCallbacks*)0)->modulate_opacity))
#define NUX_PLAYER_STEP_V3_MIN_SIZE                                       \
    (offsetof(NuxPlayerStep, elapsed_seconds) +                           \
     sizeof(((NuxPlayerStep*)0)->elapsed_seconds))
#define NUX_PLAYER_STEP_INFO_V3_MIN_SIZE                                  \
    (offsetof(NuxPlayerStepInfo, event_count) +                           \
     sizeof(((NuxPlayerStepInfo*)0)->event_count))
#define NUX_HOST_COMMAND_IMPORT_CONFIG_V3_MIN_SIZE                        \
    (offsetof(NuxHostCommandImportConfig, max_command_bytes_per_step) +   \
     sizeof(((NuxHostCommandImportConfig*)0)->max_command_bytes_per_step))
#define NUX_HOST_COMMAND_VIEW_V3_MIN_SIZE                                 \
    (offsetof(NuxHostCommandView, root_value_index) +                     \
     sizeof(((NuxHostCommandView*)0)->root_value_index))
#define NUX_HOST_VALUE_VIEW_V3_MIN_SIZE                                   \
    (offsetof(NuxHostValueView, child_count) +                            \
     sizeof(((NuxHostValueView*)0)->child_count))
#define NUX_HOST_VALUE_CHILD_VIEW_V3_MIN_SIZE                             \
    (offsetof(NuxHostValueChildView, value_index) +                       \
     sizeof(((NuxHostValueChildView*)0)->value_index))
#define NUX_PLAYER_STATE_CHANGE_VIEW_V3_MIN_SIZE                          \
    (offsetof(NuxPlayerStateChangeView, state_global_id) +                \
     sizeof(((NuxPlayerStateChangeView*)0)->state_global_id))
#define NUX_PLAYER_EVENT_VIEW_V3_MIN_SIZE                                 \
    (offsetof(NuxPlayerEventView, property_count) +                       \
     sizeof(((NuxPlayerEventView*)0)->property_count))
#define NUX_PLAYER_EVENT_PROPERTY_VIEW_V3_MIN_SIZE                        \
    (offsetof(NuxPlayerEventPropertyView, integer_value) +                \
     sizeof(((NuxPlayerEventPropertyView*)0)->integer_value))
#define NUX_VIEW_MODEL_CATALOG_INFO_V3_MIN_SIZE                           \
    (offsetof(NuxViewModelCatalogInfo, enum_label_count) +                \
     sizeof(((NuxViewModelCatalogInfo*)0)->enum_label_count))
#define NUX_VIEW_MODEL_SCHEMA_VIEW_V3_MIN_SIZE                            \
    (offsetof(NuxViewModelSchemaView, is_global) +                        \
     sizeof(((NuxViewModelSchemaView*)0)->is_global))
#define NUX_VIEW_MODEL_PROPERTY_VIEW_V3_MIN_SIZE                          \
    (offsetof(NuxViewModelPropertyView, enum_label_count) +               \
     sizeof(((NuxViewModelPropertyView*)0)->enum_label_count))
#define NUX_VIEW_MODEL_AUTHORED_INSTANCE_VIEW_V3_MIN_SIZE                 \
    (offsetof(NuxViewModelAuthoredInstanceView, name) +                   \
     sizeof(((NuxViewModelAuthoredInstanceView*)0)->name))
#define NUX_VIEW_MODEL_SNAPSHOT_INFO_V3_MIN_SIZE                          \
    (offsetof(NuxViewModelSnapshotInfo, list_item_count) +                \
     sizeof(((NuxViewModelSnapshotInfo*)0)->list_item_count))
#define NUX_VIEW_MODEL_SNAPSHOT_INSTANCE_VIEW_V3_MIN_SIZE                 \
    (offsetof(NuxViewModelSnapshotInstanceView, value_count) +            \
     sizeof(((NuxViewModelSnapshotInstanceView*)0)->value_count))
#define NUX_VIEW_MODEL_SNAPSHOT_VALUE_VIEW_V3_MIN_SIZE                    \
    (offsetof(NuxViewModelSnapshotValueView, list_item_count) +           \
     sizeof(((NuxViewModelSnapshotValueView*)0)->list_item_count))
#define NUX_VIEW_MODEL_MUTATION_BATCH_V3_MIN_SIZE                         \
    (offsetof(NuxViewModelMutationBatch, mutation_count) +                \
     sizeof(((NuxViewModelMutationBatch*)0)->mutation_count))
#define NUX_VIEW_MODEL_MUTATION_RESULT_INFO_V3_MIN_SIZE                   \
    (offsetof(NuxViewModelMutationResultInfo, message) +                  \
     sizeof(((NuxViewModelMutationResultInfo*)0)->message))
#define NUX_VIEW_MODEL_CHANGE_VIEW_V3_MIN_SIZE                            \
    (offsetof(NuxViewModelChangeView, list_item_count) +                  \
     sizeof(((NuxViewModelChangeView*)0)->list_item_count))
#define NUX_TEXT_RUN_MUTATION_BATCH_V3_MIN_SIZE                           \
    (offsetof(NuxTextRunMutationBatch, mutation_count) +                  \
     sizeof(((NuxTextRunMutationBatch*)0)->mutation_count))

#if defined(NUX_CAPI_APPLE_METAL) && defined(__APPLE__)
#define NUX_METAL_RENDER_OPERATION_V3_MIN_SIZE                            \
    (offsetof(NuxMetalRenderOperation, completion_callback) +             \
     sizeof(((NuxMetalRenderOperation*)0)->completion_callback))
#define NUX_RENDERER_OUTCOME_V3_MIN_SIZE                                  \
    (offsetof(NuxRendererOutcome, atomic_strategy_partitions) +           \
     sizeof(((NuxRendererOutcome*)0)->atomic_strategy_partitions))
#define NUX_RENDERER_INFO_V3_MIN_SIZE                                     \
    (offsetof(NuxRendererInfo, generation) +                              \
     sizeof(((NuxRendererInfo*)0)->generation))
#define NUX_RETAINED_BYTES_V3_MIN_SIZE                                    \
    (offsetof(NuxRetainedBytes, release) +                                \
     sizeof(((NuxRetainedBytes*)0)->release))
#define NUX_DECODED_IMAGE_V3_MIN_SIZE                                     \
    (offsetof(NuxDecodedImage, pixels) +                                  \
     sizeof(((NuxDecodedImage*)0)->pixels))
#define NUX_IMAGE_DECODE_REQUEST_V3_MIN_SIZE                              \
    (offsetof(NuxImageDecodeRequest, maximum_decoded_bytes) +             \
     sizeof(((NuxImageDecodeRequest*)0)->maximum_decoded_bytes))
#define NUX_EXTERNAL_ASSET_REQUEST_V3_MIN_SIZE                            \
    (offsetof(NuxExternalAssetRequest, file_extension) +                  \
     sizeof(((NuxExternalAssetRequest*)0)->file_extension))
#define NUX_APPLE_ASSET_HOOKS_V3_MIN_SIZE                                \
    (offsetof(NuxAppleAssetHooks, maximum_total_decoded_image_bytes) +    \
     sizeof(((NuxAppleAssetHooks*)0)->maximum_total_decoded_image_bytes))
#define NUX_FILE_IMPORT_CONFIG_V3_MIN_SIZE                                \
    (offsetof(NuxFileImportConfig, expected_asset_count) +                \
     sizeof(((NuxFileImportConfig*)0)->expected_asset_count))
#endif

#if defined(__cplusplus)
static_assert(NUX_FILE_ASSET_DESCRIPTOR_VIEW_V3_MIN_SIZE <=
              sizeof(NuxFileAssetDescriptorView),
              "NuxFileAssetDescriptorView v3 prefix exceeds its layout");
static_assert(NUX_VIEW_MODEL_CHANGE_VIEW_V3_MIN_SIZE <=
              sizeof(NuxViewModelChangeView),
              "NuxViewModelChangeView v3 prefix exceeds its layout");
#if defined(NUX_CAPI_APPLE_METAL) && defined(__APPLE__)
static_assert(NUX_FILE_IMPORT_CONFIG_V3_MIN_SIZE <=
              sizeof(NuxFileImportConfig),
              "NuxFileImportConfig v3 prefix exceeds its layout");
#endif
#else
_Static_assert(NUX_FILE_ASSET_DESCRIPTOR_VIEW_V3_MIN_SIZE <=
               sizeof(NuxFileAssetDescriptorView),
               "NuxFileAssetDescriptorView v3 prefix exceeds its layout");
_Static_assert(NUX_VIEW_MODEL_CHANGE_VIEW_V3_MIN_SIZE <=
               sizeof(NuxViewModelChangeView),
               "NuxViewModelChangeView v3 prefix exceeds its layout");
#if defined(NUX_CAPI_APPLE_METAL) && defined(__APPLE__)
_Static_assert(NUX_FILE_IMPORT_CONFIG_V3_MIN_SIZE <=
               sizeof(NuxFileImportConfig),
               "NuxFileImportConfig v3 prefix exceeds its layout");
#endif
#endif

/* Stable encodings used by the portable callback-renderer surface. Embedders
 * supply these per-primitive callbacks when they choose that rendering path. */
typedef enum NuxFillRule
{
    NUX_FILL_RULE_NON_ZERO = 0,
    NUX_FILL_RULE_EVEN_ODD = 1,
    NUX_FILL_RULE_CLOCKWISE = 2,
} NuxFillRule;

typedef enum NuxPathVerb
{
    NUX_PATH_VERB_MOVE = 0,
    NUX_PATH_VERB_LINE = 1,
    NUX_PATH_VERB_QUAD = 2,
    NUX_PATH_VERB_CUBIC = 4,
    NUX_PATH_VERB_CLOSE = 5,
} NuxPathVerb;

typedef enum NuxPaintStyle
{
    NUX_PAINT_STYLE_STROKE = 0,
    NUX_PAINT_STYLE_FILL = 1,
} NuxPaintStyle;

typedef enum NuxStrokeJoin
{
    NUX_STROKE_JOIN_MITER = 0,
    NUX_STROKE_JOIN_ROUND = 1,
    NUX_STROKE_JOIN_BEVEL = 2,
} NuxStrokeJoin;

typedef enum NuxStrokeCap
{
    NUX_STROKE_CAP_BUTT = 0,
    NUX_STROKE_CAP_ROUND = 1,
    NUX_STROKE_CAP_SQUARE = 2,
} NuxStrokeCap;

typedef enum NuxBlendMode
{
    NUX_BLEND_MODE_SRC_OVER = 3,
    NUX_BLEND_MODE_SCREEN = 14,
    NUX_BLEND_MODE_OVERLAY = 15,
    NUX_BLEND_MODE_DARKEN = 16,
    NUX_BLEND_MODE_LIGHTEN = 17,
    NUX_BLEND_MODE_COLOR_DODGE = 18,
    NUX_BLEND_MODE_COLOR_BURN = 19,
    NUX_BLEND_MODE_HARD_LIGHT = 20,
    NUX_BLEND_MODE_SOFT_LIGHT = 21,
    NUX_BLEND_MODE_DIFFERENCE = 22,
    NUX_BLEND_MODE_EXCLUSION = 23,
    NUX_BLEND_MODE_MULTIPLY = 24,
    NUX_BLEND_MODE_HUE = 25,
    NUX_BLEND_MODE_SATURATION = 26,
    NUX_BLEND_MODE_COLOR = 27,
    NUX_BLEND_MODE_LUMINOSITY = 28,
} NuxBlendMode;

typedef enum NuxImageWrap
{
    NUX_IMAGE_WRAP_CLAMP = 0,
    NUX_IMAGE_WRAP_REPEAT = 1,
    NUX_IMAGE_WRAP_MIRROR = 2,
} NuxImageWrap;

typedef enum NuxImageFilter
{
    NUX_IMAGE_FILTER_BILINEAR = 0,
    NUX_IMAGE_FILTER_NEAREST = 1,
} NuxImageFilter;

typedef enum NuxRenderBufferType
{
    NUX_RENDER_BUFFER_TYPE_INDEX = 0,
    NUX_RENDER_BUFFER_TYPE_VERTEX = 1,
} NuxRenderBufferType;

typedef enum NuxRenderBufferFlags
{
    NUX_RENDER_BUFFER_FLAGS_NONE = 0,
    NUX_RENDER_BUFFER_FLAGS_MAPPED_ONCE_AT_INITIALIZATION = 1,
} NuxRenderBufferFlags;

#endif /* NUX_CAPI_H */
