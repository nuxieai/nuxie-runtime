#ifndef RIVE_CAPI_H
#define RIVE_CAPI_H

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
 *   RiveFile
 *     |- RiveArtboardInstance      (borrows the RiveFile)
 *          |- RiveStateMachineInstance
 *
 * 1. A RiveFile must stay alive (not passed to rive_file_free) for as long
 *    as ANY RiveArtboardInstance created from it exists. Freeing the file
 *    first leaves every instance dangling; any later use of an instance --
 *    including rive_artboard_instance_free -- is undefined behavior. Debug
 *    builds of the library detect this misuse inside rive_file_free and
 *    abort with a diagnostic instead of corrupting memory; release builds
 *    do not check.
 * 2. A RiveStateMachineInstance must be freed before the
 *    RiveArtboardInstance it was created from, and must only ever be
 *    advanced or handed pointer events through that same artboard instance.
 * 3. RiveStringView results borrow the RiveFile and are valid until the
 *    file is freed. Copy the bytes out if you need them longer.
 * 4. Handles are not thread-safe: never use a handle (or its parent
 *    hierarchy) from two threads at once.
 *
 * PANIC SAFETY: no function ever unwinds across this ABI. When an internal
 * error is caught, functions returning RiveStatus report
 * RIVE_STATUS_RUNTIME_ERROR, rive_file_artboard_count returns 0, and void
 * functions return normally.
 * ========================================================================== */

typedef enum RiveStatus
{
    RIVE_STATUS_OK = 0,
    RIVE_STATUS_NULL_ARGUMENT = 1,
    RIVE_STATUS_IMPORT_ERROR = 2,
    RIVE_STATUS_NOT_FOUND = 3,
    RIVE_STATUS_RUNTIME_ERROR = 4,
    RIVE_STATUS_INVALID_ARGUMENT = 5,
} RiveStatus;

typedef struct RiveFile RiveFile;
typedef struct RiveArtboardInstance RiveArtboardInstance;
typedef struct RiveStateMachineInstance RiveStateMachineInstance;

typedef struct RiveStringView
{
    const char* data;
    size_t len;
} RiveStringView;

/* Enum encodings used by the render callback vtable. */

typedef enum RiveFillRule
{
    RIVE_FILL_RULE_NON_ZERO = 0,
    RIVE_FILL_RULE_EVEN_ODD = 1,
    RIVE_FILL_RULE_CLOCKWISE = 2,
} RiveFillRule;

typedef enum RivePathVerb
{
    RIVE_PATH_VERB_MOVE = 0,
    RIVE_PATH_VERB_LINE = 1,
    RIVE_PATH_VERB_QUAD = 2,
    RIVE_PATH_VERB_CUBIC = 4,
    RIVE_PATH_VERB_CLOSE = 5,
} RivePathVerb;

typedef enum RivePaintStyle
{
    RIVE_PAINT_STYLE_STROKE = 0,
    RIVE_PAINT_STYLE_FILL = 1,
} RivePaintStyle;

typedef enum RiveStrokeJoin
{
    RIVE_STROKE_JOIN_MITER = 0,
    RIVE_STROKE_JOIN_ROUND = 1,
    RIVE_STROKE_JOIN_BEVEL = 2,
} RiveStrokeJoin;

typedef enum RiveStrokeCap
{
    RIVE_STROKE_CAP_BUTT = 0,
    RIVE_STROKE_CAP_ROUND = 1,
    RIVE_STROKE_CAP_SQUARE = 2,
} RiveStrokeCap;

typedef enum RiveBlendMode
{
    RIVE_BLEND_MODE_SRC_OVER = 3,
    RIVE_BLEND_MODE_SCREEN = 14,
    RIVE_BLEND_MODE_OVERLAY = 15,
    RIVE_BLEND_MODE_DARKEN = 16,
    RIVE_BLEND_MODE_LIGHTEN = 17,
    RIVE_BLEND_MODE_COLOR_DODGE = 18,
    RIVE_BLEND_MODE_COLOR_BURN = 19,
    RIVE_BLEND_MODE_HARD_LIGHT = 20,
    RIVE_BLEND_MODE_SOFT_LIGHT = 21,
    RIVE_BLEND_MODE_DIFFERENCE = 22,
    RIVE_BLEND_MODE_EXCLUSION = 23,
    RIVE_BLEND_MODE_MULTIPLY = 24,
    RIVE_BLEND_MODE_HUE = 25,
    RIVE_BLEND_MODE_SATURATION = 26,
    RIVE_BLEND_MODE_COLOR = 27,
    RIVE_BLEND_MODE_LUMINOSITY = 28,
} RiveBlendMode;

typedef enum RiveImageWrap
{
    RIVE_IMAGE_WRAP_CLAMP = 0,
    RIVE_IMAGE_WRAP_REPEAT = 1,
    RIVE_IMAGE_WRAP_MIRROR = 2,
} RiveImageWrap;

typedef enum RiveImageFilter
{
    RIVE_IMAGE_FILTER_BILINEAR = 0,
    RIVE_IMAGE_FILTER_NEAREST = 1,
} RiveImageFilter;

typedef enum RiveRenderBufferType
{
    RIVE_RENDER_BUFFER_TYPE_INDEX = 0,
    RIVE_RENDER_BUFFER_TYPE_VERTEX = 1,
} RiveRenderBufferType;

typedef enum RiveRenderBufferFlags
{
    RIVE_RENDER_BUFFER_FLAGS_NONE = 0,
    RIVE_RENDER_BUFFER_FLAGS_MAPPED_ONCE_AT_INITIALIZATION = 1,
} RiveRenderBufferFlags;

/* Borrowed view of a path: `verbs` holds RivePathVerb values, `points` holds
 * `point_count` interleaved x,y pairs. Only valid during the callback. */
typedef struct RiveRawPathView
{
    const uint8_t* verbs;
    size_t verb_count;
    const float* points;
    size_t point_count;
} RiveRawPathView;

typedef struct RiveImageSampler
{
    uint8_t wrap_x; /* RiveImageWrap */
    uint8_t wrap_y; /* RiveImageWrap */
    uint8_t filter; /* RiveImageFilter */
} RiveImageSampler;

/* Caller-provided render vtable mirroring the rive-render-api traits.
 *
 * Object handles are opaque uint64_t values chosen by the caller from its
 * make_* callbacks; later mutation/draw callbacks receive them back and the
 * matching release_* callback fires exactly once per created object. Handle 0
 * means "no object" (for example a cleared shader). Transform pointers
 * reference six floats ordered [xx, yx, xy, yy, tx, ty]. Every callback may
 * be NULL; missing callbacks degrade to no-ops. */
typedef struct RiveRenderCallbacks
{
    void* user_data;

    /* Factory calls. */
    uint64_t (*make_render_path)(void* user_data,
                                 const RiveRawPathView* path,
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
                                     const RiveRawPathView* raw_path);
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
                       RiveImageSampler sampler,
                       uint8_t blend_mode,
                       float opacity);
    void (*draw_image_mesh)(void* user_data,
                            uint64_t image,
                            RiveImageSampler sampler,
                            uint64_t vertices,
                            uint64_t uv_coords,
                            uint64_t indices,
                            uint32_t vertex_count,
                            uint32_t index_count,
                            uint8_t blend_mode,
                            float opacity);
    void (*modulate_opacity)(void* user_data, float opacity);
} RiveRenderCallbacks;

/* File import and metadata. */

RiveStatus rive_file_import(const uint8_t* bytes, size_t len, RiveFile** out_file);

/* Free an imported file. Every RiveArtboardInstance created from this file
 * must already have been freed (see the ownership contract above). */
void rive_file_free(RiveFile* file);

size_t rive_file_artboard_count(const RiveFile* file);
RiveStatus rive_file_artboard_name(
    const RiveFile* file,
    size_t index,
    RiveStringView* out_name);
RiveStatus rive_file_artboard_animation_count(
    const RiveFile* file,
    size_t index,
    size_t* out_count);
RiveStatus rive_file_artboard_state_machine_count(
    const RiveFile* file,
    size_t index,
    size_t* out_count);
RiveStatus rive_file_artboard_state_machine_name(
    const RiveFile* file,
    size_t artboard_index,
    size_t state_machine_index,
    RiveStringView* out_name);

/* Artboard instances. The file must outlive its instances: free every
 * instance with rive_artboard_instance_free BEFORE calling rive_file_free. */

RiveStatus rive_artboard_instance_new(
    const RiveFile* file,
    size_t artboard_index,
    RiveArtboardInstance** out_instance);
void rive_artboard_instance_free(RiveArtboardInstance* instance);

/* Advance the artboard timeline without a state machine. `out_changed` is
 * optional and reports whether anything changed. */
RiveStatus rive_artboard_instance_advance(
    RiveArtboardInstance* instance,
    float elapsed_seconds,
    bool* out_changed);

/* Draw the artboard through the caller-provided render vtable. The callbacks
 * struct only needs to stay valid for the duration of this call. */
RiveStatus rive_artboard_instance_draw(
    RiveArtboardInstance* instance,
    const RiveRenderCallbacks* callbacks);

/* State machine instances. Free them before the artboard instance they were
 * created from. */

RiveStatus rive_state_machine_instance_new(
    const RiveArtboardInstance* instance,
    size_t state_machine_index,
    RiveStateMachineInstance** out_state_machine);

/* Default selection: the state machine flagged in the source file when
 * present, otherwise the first state machine. RIVE_STATUS_NOT_FOUND when the
 * artboard has none. */
RiveStatus rive_state_machine_instance_new_default(
    const RiveArtboardInstance* instance,
    RiveStateMachineInstance** out_state_machine);
void rive_state_machine_instance_free(RiveStateMachineInstance* state_machine);

/* Inputs are addressed by NUL-terminated UTF-8 name. RIVE_STATUS_NOT_FOUND
 * when no input has that name; RIVE_STATUS_INVALID_ARGUMENT when the input
 * has a different kind. */
RiveStatus rive_state_machine_instance_set_bool(
    RiveStateMachineInstance* state_machine,
    const char* name,
    bool value);
RiveStatus rive_state_machine_instance_set_number(
    RiveStateMachineInstance* state_machine,
    const char* name,
    float value);
RiveStatus rive_state_machine_instance_fire_trigger(
    RiveStateMachineInstance* state_machine,
    const char* name);

/* Advance the artboard while driving `state_machine` (which must have been
 * created from `instance`). `out_changed` is optional. */
RiveStatus rive_state_machine_instance_advance(
    RiveArtboardInstance* instance,
    RiveStateMachineInstance* state_machine,
    float elapsed_seconds,
    bool* out_changed);

/* Pointer events. Coordinates are in artboard space. The state machine must
 * have been created from `instance`. `out_hit` is optional and reports
 * whether the event landed on a listener. Effects are applied on the next
 * rive_state_machine_instance_advance. */
RiveStatus rive_state_machine_instance_pointer_down(
    const RiveArtboardInstance* instance,
    RiveStateMachineInstance* state_machine,
    float x,
    float y,
    bool* out_hit);
RiveStatus rive_state_machine_instance_pointer_move(
    const RiveArtboardInstance* instance,
    RiveStateMachineInstance* state_machine,
    float x,
    float y,
    bool* out_hit);
RiveStatus rive_state_machine_instance_pointer_up(
    const RiveArtboardInstance* instance,
    RiveStateMachineInstance* state_machine,
    float x,
    float y,
    bool* out_hit);

#ifdef __cplusplus
}
#endif

#endif
