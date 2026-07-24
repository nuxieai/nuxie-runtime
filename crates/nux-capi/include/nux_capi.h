#ifndef NUX_CAPI_H
#define NUX_CAPI_H

/* Public umbrella header for the Nuxie runtime C ABI.
 *
 * Declarations live in nux_capi.generated.h, which is generated from the
 * Rust exports by cbindgen and byte-verified during every nux-capi build.
 *
 * LIFETIME AND OWNERSHIP CONTRACT
 *
 * Handles form a strict parent/child hierarchy and must be freed child-first:
 *
 *   NuxFile
 *     |- NuxArtboardInstance
 *          |- NuxStateMachineInstance
 *
 * 1. NuxFile must outlive every NuxArtboardInstance created from it.
 * 2. NuxStateMachineInstance must be freed before its NuxArtboardInstance.
 * 3. NuxStringView values borrow their documented owner unless their field is
 *    explicitly documented as process-static. Copy the bytes when a longer
 *    lifetime is needed; views are not NUL-terminated.
 * 4. Handles are not thread-safe and may not be used concurrently.
 * 5. NuxViewModelInstance owns its value copy and does not borrow NuxFile, but
 *    it is only meaningful when bound to the artboard that created it.
 * 6. NuxArtboardInstance retains callback-created renderer objects. The first
 *    draw's callback functions and user_data must remain valid until
 *    nux_artboard_instance_free returns.
 *
 * PANIC SAFETY
 *
 * Every exported entry point has a panic firewall. An unwind never crosses
 * this ABI. Status-returning calls report NUX_STATUS_RUNTIME_ERROR; scalar or
 * void calls return their documented safe fallback.
 */

#include "nux_capi.generated.h"

/* Stable encodings used by the generic callback-renderer test surface. The
 * Apple product renderer does not expose per-primitive callbacks. */
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
