#ifndef NUX_CAPI_H
#define NUX_CAPI_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ==========================================================================
 * LIFETIME AND OWNERSHIP CONTRACT (applies to every function below)
 *
 * Handles form a strict parent/child hierarchy and must be freed strictly
 * child-first:
 *
 *   NuxFile
 *     |- NuxArtboardInstance      (borrows the NuxFile)
 *          |- NuxStateMachineInstance
 *          |- NuxRenderCache
 *
 * 1. A NuxFile must stay alive (not passed to nux_file_free) for as long
 *    as ANY NuxArtboardInstance created from it exists. Freeing the file
 *    first leaves every instance dangling; any later use of an instance --
 *    including nux_artboard_instance_free -- is undefined behavior. Debug
 *    builds of the library detect this misuse inside nux_file_free and
 *    abort with a diagnostic instead of corrupting memory; release builds
 *    do not check.
 * 2. A NuxStateMachineInstance must be freed before the
 *    NuxArtboardInstance it was created from, and must only ever be
 *    advanced or handed pointer events through that same artboard instance.
 * 3. NuxStringView results borrow the NuxFile and are valid until the
 *    file is freed. Copy the bytes out if you need them longer.
 * 4. Handles are not thread-safe: never use a handle (or its parent
 *    hierarchy) from two threads at once.
 * 5. NuxViewModelInstance is the exception to the hierarchy: it owns a
 *    private copy of the view model's values and borrows nothing, so it has
 *    no free-ordering constraint relative to the file or artboard instance.
 *    It is only useful while bound to the artboard instance it was created
 *    from, which must be alive at bind time.
 * 6. A NuxRenderCache is bound to the NuxArtboardInstance and render
 *    callbacks used to create it. Free it before that artboard instance and
 *    keep its callback user_data valid until nux_render_cache_free returns.
 *
 * PANIC SAFETY: no function ever unwinds across this ABI. When an internal
 * error is caught, functions returning NuxStatus report
 * NUX_STATUS_RUNTIME_ERROR, nux_file_artboard_count returns 0, and void
 * functions return normally.
 * ========================================================================== */

typedef enum NuxStatus
{
    NUX_STATUS_OK = 0,
    NUX_STATUS_NULL_ARGUMENT = 1,
    NUX_STATUS_IMPORT_ERROR = 2,
    NUX_STATUS_NOT_FOUND = 3,
    NUX_STATUS_RUNTIME_ERROR = 4,
    NUX_STATUS_INVALID_ARGUMENT = 5,
} NuxStatus;

typedef struct NuxFile NuxFile;
typedef struct NuxArtboardInstance NuxArtboardInstance;
typedef struct NuxStateMachineInstance NuxStateMachineInstance;
typedef struct NuxViewModelInstance NuxViewModelInstance;
typedef struct NuxRenderCache NuxRenderCache;

typedef struct NuxStringView
{
    const char* data;
    size_t len;
} NuxStringView;

/* Enum encodings used by the render callback vtable. */

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

/* Borrowed view of a path: `verbs` holds NuxPathVerb values, `points` holds
 * `point_count` interleaved x,y pairs. Only valid during the callback. */
typedef struct NuxRawPathView
{
    const uint8_t* verbs;
    size_t verb_count;
    const float* points;
    size_t point_count;
} NuxRawPathView;

typedef struct NuxImageSampler
{
    uint8_t wrap_x; /* NuxImageWrap */
    uint8_t wrap_y; /* NuxImageWrap */
    uint8_t filter; /* NuxImageFilter */
} NuxImageSampler;

/* Caller-provided render vtable mirroring the nuxie-render-api traits.
 *
 * Object handles are opaque uint64_t values chosen by the caller from its
 * make_* callbacks; later mutation/draw callbacks receive them back and the
 * matching release_* callback fires exactly once per created object. Handle 0
 * means "no object" (for example a cleared shader). Transform pointers
 * reference six floats ordered [xx, yx, xy, yy, tx, ty]. Every callback may
 * be NULL; missing callbacks degrade to no-ops. */
typedef struct NuxRenderCallbacks
{
    void* user_data;

    /* Factory calls. */
    uint64_t (*make_render_path)(void* user_data,
                                 const NuxRawPathView* path,
                                 uint8_t fill_rule);
    uint64_t (*make_empty_render_path)(void* user_data);
    uint64_t (*make_render_paint)(void* user_data);
    uint64_t (*make_linear_gradient)(void* user_data,
                                     float sx,
                                     float sy,
                                     float ex,
                                     float ey,
                                     const uint32_t* colors,
                                     const float* stops,
                                     size_t count);
    uint64_t (*make_radial_gradient)(void* user_data,
                                     float cx,
                                     float cy,
                                     float radius,
                                     const uint32_t* colors,
                                     const float* stops,
                                     size_t count);
    uint64_t (*make_render_buffer)(void* user_data,
                                   uint8_t buffer_type,
                                   uint8_t flags,
                                   size_t size_in_bytes);
    uint64_t (*decode_image)(void* user_data,
                             const uint8_t* bytes,
                             size_t len,
                             uint32_t* out_width,
                             uint32_t* out_height);

    /* Object releases (paired with the factory calls above). */
    void (*release_render_path)(void* user_data, uint64_t path);
    void (*release_render_paint)(void* user_data, uint64_t paint);
    void (*release_render_shader)(void* user_data, uint64_t shader);
    void (*release_render_buffer)(void* user_data, uint64_t buffer);
    void (*release_render_image)(void* user_data, uint64_t image);

    /* RenderPath mutation. */
    void (*render_path_rewind)(void* user_data, uint64_t path);
    void (*render_path_fill_rule)(void* user_data,
                                  uint64_t path,
                                  uint8_t fill_rule);
    void (*render_path_move_to)(void* user_data, uint64_t path, float x, float y);
    void (*render_path_line_to)(void* user_data, uint64_t path, float x, float y);
    void (*render_path_cubic_to)(void* user_data,
                                 uint64_t path,
                                 float ox,
                                 float oy,
                                 float ix,
                                 float iy,
                                 float x,
                                 float y);
    void (*render_path_close)(void* user_data, uint64_t path);
    void (*render_path_add_raw_path)(void* user_data,
                                     uint64_t path,
                                     const NuxRawPathView* raw_path);
    void (*render_path_add_render_path)(void* user_data,
                                        uint64_t path,
                                        uint64_t other_path,
                                        const float* transform);
    void (*render_path_add_render_path_backwards)(void* user_data,
                                                  uint64_t path,
                                                  uint64_t other_path,
                                                  const float* transform);

    /* RenderPaint mutation. */
    void (*render_paint_style)(void* user_data, uint64_t paint, uint8_t style);
    void (*render_paint_color)(void* user_data, uint64_t paint, uint32_t color);
    void (*render_paint_thickness)(void* user_data, uint64_t paint, float value);
    void (*render_paint_join)(void* user_data, uint64_t paint, uint32_t join);
    void (*render_paint_cap)(void* user_data, uint64_t paint, uint32_t cap);
    void (*render_paint_feather)(void* user_data, uint64_t paint, float value);
    void (*render_paint_blend_mode)(void* user_data,
                                    uint64_t paint,
                                    uint8_t blend_mode);
    void (*render_paint_shader)(void* user_data, uint64_t paint, uint64_t shader);
    void (*render_paint_invalidate_stroke)(void* user_data, uint64_t paint);

    /* RenderBuffer unmap: receives the staged bytes for the buffer handle. */
    void (*render_buffer_unmap)(void* user_data,
                                uint64_t buffer,
                                const uint8_t* bytes,
                                size_t len);

    /* Renderer calls. */
    void (*save)(void* user_data);
    void (*restore)(void* user_data);
    void (*transform)(void* user_data, const float* transform);
    void (*draw_path)(void* user_data, uint64_t path, uint64_t paint);
    void (*clip_path)(void* user_data, uint64_t path);
    void (*draw_image)(void* user_data,
                       uint64_t image,
                       NuxImageSampler sampler,
                       uint8_t blend_mode,
                       float opacity);
    void (*draw_image_mesh)(void* user_data,
                            uint64_t image,
                            NuxImageSampler sampler,
                            uint64_t vertices,
                            uint64_t uv_coords,
                            uint64_t indices,
                            uint32_t vertex_count,
                            uint32_t index_count,
                            uint8_t blend_mode,
                            float opacity);
    void (*modulate_opacity)(void* user_data, float opacity);
} NuxRenderCallbacks;

/* File import and metadata. */

NuxStatus nux_file_import(const uint8_t* bytes, size_t len, NuxFile** out_file);

/* Free an imported file. Every NuxArtboardInstance created from this file
 * must already have been freed (see the ownership contract above). */
void nux_file_free(NuxFile* file);

size_t nux_file_artboard_count(const NuxFile* file);
NuxStatus nux_file_artboard_name(
    const NuxFile* file,
    size_t index,
    NuxStringView* out_name);
NuxStatus nux_file_artboard_animation_count(
    const NuxFile* file,
    size_t index,
    size_t* out_count);
NuxStatus nux_file_artboard_state_machine_count(
    const NuxFile* file,
    size_t index,
    size_t* out_count);
NuxStatus nux_file_artboard_state_machine_name(
    const NuxFile* file,
    size_t artboard_index,
    size_t state_machine_index,
    NuxStringView* out_name);

/* Artboard instances. The file must outlive its instances: free every
 * instance with nux_artboard_instance_free BEFORE calling nux_file_free. */

NuxStatus nux_artboard_instance_new(
    const NuxFile* file,
    size_t artboard_index,
    NuxArtboardInstance** out_instance);
void nux_artboard_instance_free(NuxArtboardInstance* instance);

/* Advance the artboard timeline without a state machine. `out_changed` is
 * optional and reports whether anything changed. */
NuxStatus nux_artboard_instance_advance(
    NuxArtboardInstance* instance,
    float elapsed_seconds,
    bool* out_changed);

/* Draw the artboard through the caller-provided render vtable. The callbacks
 * struct only needs to stay valid for the duration of this call. */
NuxStatus nux_artboard_instance_draw(
    NuxArtboardInstance* instance,
    const NuxRenderCallbacks* callbacks);

/* Create a cache that retains render handles across frames. The callbacks and
 * their user_data must remain usable until the cache is freed. */
NuxStatus nux_render_cache_new(
    const NuxArtboardInstance* instance,
    const NuxRenderCallbacks* callbacks,
    NuxRenderCache** out_cache);

/* Draw using a cache created for this exact artboard instance. */
NuxStatus nux_artboard_instance_draw_cached(
    NuxArtboardInstance* instance,
    NuxRenderCache* cache);
void nux_render_cache_free(NuxRenderCache* cache);

/* State machine instances. Free them before the artboard instance they were
 * created from. */

NuxStatus nux_state_machine_instance_new(
    const NuxArtboardInstance* instance,
    size_t state_machine_index,
    NuxStateMachineInstance** out_state_machine);

/* Default selection: the state machine flagged in the source file when
 * present, otherwise the first state machine. NUX_STATUS_NOT_FOUND when the
 * artboard has none. */
NuxStatus nux_state_machine_instance_new_default(
    const NuxArtboardInstance* instance,
    NuxStateMachineInstance** out_state_machine);
void nux_state_machine_instance_free(NuxStateMachineInstance* state_machine);

/* Inputs are addressed by NUL-terminated UTF-8 name. NUX_STATUS_NOT_FOUND
 * when no input has that name; NUX_STATUS_INVALID_ARGUMENT when the input
 * has a different kind. */
NuxStatus nux_state_machine_instance_set_bool(
    NuxStateMachineInstance* state_machine,
    const char* name,
    bool value);
NuxStatus nux_state_machine_instance_set_number(
    NuxStateMachineInstance* state_machine,
    const char* name,
    float value);
NuxStatus nux_state_machine_instance_fire_trigger(
    NuxStateMachineInstance* state_machine,
    const char* name);

/* Advance the artboard while driving `state_machine` (which must have been
 * created from `instance`). `out_changed` is optional. */
NuxStatus nux_state_machine_instance_advance(
    NuxArtboardInstance* instance,
    NuxStateMachineInstance* state_machine,
    float elapsed_seconds,
    bool* out_changed);

/* Pointer events. Coordinates are in artboard space. The state machine must
 * have been created from `instance`. `out_hit` is optional and reports
 * whether the event landed on a listener. Effects are applied on the next
 * nux_state_machine_instance_advance. */
NuxStatus nux_state_machine_instance_pointer_down(
    const NuxArtboardInstance* instance,
    NuxStateMachineInstance* state_machine,
    float x,
    float y,
    bool* out_hit);
NuxStatus nux_state_machine_instance_pointer_move(
    const NuxArtboardInstance* instance,
    NuxStateMachineInstance* state_machine,
    float x,
    float y,
    bool* out_hit);
NuxStatus nux_state_machine_instance_pointer_up(
    const NuxArtboardInstance* instance,
    NuxStateMachineInstance* state_machine,
    float x,
    float y,
    bool* out_hit);

/* View-model instances. A view-model context drives an artboard's data binds.
 * It owns its values and borrows nothing (see the ownership contract, point 5),
 * so free it with nux_view_model_instance_free whenever you are done with it.
 *
 * Typical use: create a context for the artboard, set properties, bind it with
 * nux_artboard_instance_bind_view_model, then advance and draw the artboard.
 * Because the binding copies the values in, re-bind after every mutation for it
 * to take effect on the next advance. */

/* Default selection: the artboard's view model with generated default values.
 * NUX_STATUS_NOT_FOUND when the artboard declares no view model. */
NuxStatus nux_view_model_instance_new_default(
    const NuxArtboardInstance* instance,
    NuxViewModelInstance** out_view_model);

/* Instance selection: the artboard's view model populated from the source
 * instance at `instance_index`. NUX_STATUS_NOT_FOUND when the artboard
 * declares no view model or the index is out of range. */
NuxStatus nux_view_model_instance_new_instance(
    const NuxArtboardInstance* instance,
    size_t instance_index,
    NuxViewModelInstance** out_view_model);
void nux_view_model_instance_free(NuxViewModelInstance* view_model);

/* Properties are addressed by NUL-terminated UTF-8 name path, using '/' to
 * descend into nested view models (for example "child/width"). Each setter
 * returns NUX_STATUS_NOT_FOUND when no settable property of the matching kind
 * exists at that path. */
NuxStatus nux_view_model_instance_set_number(
    NuxViewModelInstance* view_model,
    const char* name_path,
    float value);
NuxStatus nux_view_model_instance_set_bool(
    NuxViewModelInstance* view_model,
    const char* name_path,
    bool value);
NuxStatus nux_view_model_instance_set_string(
    NuxViewModelInstance* view_model,
    const char* name_path,
    const char* value);

/* Bind `view_model` to `instance`'s own data binds and nested-artboard
 * contexts (mirrors artboard->bindViewModelInstance). `view_model` must have
 * been created from `instance`. The values are copied in at bind time, so
 * re-bind after mutating the context for the change to reach the next advance. */
NuxStatus nux_artboard_instance_bind_view_model(
    NuxArtboardInstance* instance,
    const NuxViewModelInstance* view_model);

#ifdef __cplusplus
}
#endif

#endif
