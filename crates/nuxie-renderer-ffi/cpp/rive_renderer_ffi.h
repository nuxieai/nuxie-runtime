#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct rive_ffi_context rive_ffi_context;
typedef struct rive_ffi_renderer rive_ffi_renderer;
typedef struct rive_ffi_render_path rive_ffi_render_path;
typedef struct rive_ffi_render_paint rive_ffi_render_paint;
typedef struct rive_ffi_render_shader rive_ffi_render_shader;
typedef struct rive_ffi_render_image rive_ffi_render_image;
#ifdef RIVE_FFI_DECODE_ORACLE
typedef struct rive_ffi_decoded_bitmap rive_ffi_decoded_bitmap;
#endif
typedef struct rive_ffi_render_buffer rive_ffi_render_buffer;

typedef struct rive_ffi_vec2d
{
    float x;
    float y;
} rive_ffi_vec2d;

typedef struct rive_ffi_mat2d
{
    float values[6];
} rive_ffi_mat2d;

typedef struct rive_ffi_adapter_identity
{
    char name[256];
    char vendor[64];
    char device[64];
    char driver[256];
} rive_ffi_adapter_identity;

rive_ffi_context* rive_ffi_context_make_null(uint32_t width, uint32_t height);
rive_ffi_context* rive_ffi_context_make_metal(uint32_t width, uint32_t height);
rive_ffi_context* rive_ffi_context_make_dawn(uint32_t width, uint32_t height);
void rive_ffi_context_delete(rive_ffi_context*);
int rive_ffi_context_begin_frame(rive_ffi_context*,
                                 uint32_t width,
                                 uint32_t height,
                                 uint32_t clear_color);
int rive_ffi_context_begin_frame_mode(rive_ffi_context*,
                                      uint32_t width,
                                      uint32_t height,
                                      uint32_t clear_color,
                                      uint32_t mode);
int rive_ffi_context_end_frame(rive_ffi_context*);
size_t rive_ffi_context_read_pixels(rive_ffi_context*, uint8_t* out, size_t len);
uint64_t rive_ffi_context_draw_count(const rive_ffi_context*);
uint64_t rive_ffi_context_logical_flush_count(const rive_ffi_context*);
size_t rive_ffi_context_adapter_name(const rive_ffi_context*,
                                     char* out,
                                     size_t len);
int rive_ffi_metal_adapter_identity(rive_ffi_adapter_identity*);
rive_ffi_renderer* rive_ffi_context_renderer(rive_ffi_context*);
void rive_ffi_renderer_delete(rive_ffi_renderer*);

rive_ffi_render_shader* rive_ffi_make_linear_gradient(rive_ffi_context*,
                                                      float sx,
                                                      float sy,
                                                      float ex,
                                                      float ey,
                                                      const uint32_t* colors,
                                                      const float* stops,
                                                      size_t count);
rive_ffi_render_shader* rive_ffi_make_radial_gradient(rive_ffi_context*,
                                                      float cx,
                                                      float cy,
                                                      float radius,
                                                      const uint32_t* colors,
                                                      const float* stops,
                                                      size_t count);
void rive_ffi_render_shader_delete(rive_ffi_render_shader*);

rive_ffi_render_path* rive_ffi_make_render_path(rive_ffi_context*,
                                                const uint8_t* verbs,
                                                size_t verb_count,
                                                const rive_ffi_vec2d* points,
                                                size_t point_count,
                                                uint8_t fill_rule);
rive_ffi_render_path* rive_ffi_make_empty_render_path(rive_ffi_context*);
void rive_ffi_render_path_delete(rive_ffi_render_path*);
void rive_ffi_render_path_rewind(rive_ffi_render_path*);
void rive_ffi_render_path_fill_rule(rive_ffi_render_path*, uint8_t fill_rule);
void rive_ffi_render_path_add_render_path(rive_ffi_render_path*,
                                          const rive_ffi_render_path* source,
                                          rive_ffi_mat2d transform);
void rive_ffi_render_path_add_render_path_backwards(
    rive_ffi_render_path*,
    const rive_ffi_render_path* source,
    rive_ffi_mat2d transform);
void rive_ffi_render_path_add_raw_path(rive_ffi_render_path*,
                                       const uint8_t* verbs,
                                       size_t verb_count,
                                       const rive_ffi_vec2d* points,
                                       size_t point_count);
void rive_ffi_render_path_move_to(rive_ffi_render_path*, float x, float y);
void rive_ffi_render_path_line_to(rive_ffi_render_path*, float x, float y);
void rive_ffi_render_path_cubic_to(rive_ffi_render_path*,
                                   float ox,
                                   float oy,
                                   float ix,
                                   float iy,
                                   float x,
                                   float y);
void rive_ffi_render_path_close(rive_ffi_render_path*);

rive_ffi_render_paint* rive_ffi_make_render_paint(rive_ffi_context*);
void rive_ffi_render_paint_delete(rive_ffi_render_paint*);
void rive_ffi_render_paint_style(rive_ffi_render_paint*, uint8_t style);
void rive_ffi_render_paint_color(rive_ffi_render_paint*, uint32_t color);
void rive_ffi_render_paint_thickness(rive_ffi_render_paint*, float thickness);
void rive_ffi_render_paint_join(rive_ffi_render_paint*, uint32_t join);
void rive_ffi_render_paint_cap(rive_ffi_render_paint*, uint32_t cap);
void rive_ffi_render_paint_feather(rive_ffi_render_paint*, float feather);
void rive_ffi_render_paint_blend_mode(rive_ffi_render_paint*,
                                      uint8_t blend_mode);
void rive_ffi_render_paint_shader(rive_ffi_render_paint*,
                                  const rive_ffi_render_shader*);
void rive_ffi_render_paint_invalidate_stroke(rive_ffi_render_paint*);

rive_ffi_render_image* rive_ffi_decode_image(rive_ffi_context*,
                                             const uint8_t* bytes,
                                             size_t len);
void rive_ffi_render_image_delete(rive_ffi_render_image*);
uint32_t rive_ffi_render_image_width(const rive_ffi_render_image*);
uint32_t rive_ffi_render_image_height(const rive_ffi_render_image*);

#ifdef RIVE_FFI_DECODE_ORACLE
rive_ffi_decoded_bitmap* rive_ffi_decode_bitmap_rgba(const uint8_t* bytes,
                                                     size_t len);
void rive_ffi_decoded_bitmap_delete(rive_ffi_decoded_bitmap*);
uint32_t rive_ffi_decoded_bitmap_width(const rive_ffi_decoded_bitmap*);
uint32_t rive_ffi_decoded_bitmap_height(const rive_ffi_decoded_bitmap*);
size_t rive_ffi_decoded_bitmap_copy_bytes(const rive_ffi_decoded_bitmap*,
                                          uint8_t* out,
                                          size_t len);
#endif

rive_ffi_render_buffer* rive_ffi_make_render_buffer(rive_ffi_context*,
                                                    uint8_t buffer_type,
                                                    uint8_t flags,
                                                    size_t size_in_bytes);
void rive_ffi_render_buffer_delete(rive_ffi_render_buffer*);
void rive_ffi_render_buffer_write(rive_ffi_render_buffer*,
                                  const uint8_t* bytes,
                                  size_t len);

void rive_ffi_renderer_save(rive_ffi_renderer*);
void rive_ffi_renderer_restore(rive_ffi_renderer*);
void rive_ffi_renderer_transform(rive_ffi_renderer*, rive_ffi_mat2d);
void rive_ffi_renderer_draw_path(rive_ffi_renderer*,
                                 rive_ffi_render_path*,
                                 rive_ffi_render_paint*);
void rive_ffi_renderer_clip_path(rive_ffi_renderer*, rive_ffi_render_path*);
void rive_ffi_renderer_draw_image(rive_ffi_renderer*,
                                  const rive_ffi_render_image*,
                                  uint8_t sampler,
                                  uint8_t blend_mode,
                                  float opacity);
void rive_ffi_renderer_draw_image_mesh(rive_ffi_renderer*,
                                       const rive_ffi_render_image*,
                                       uint8_t sampler,
                                       const rive_ffi_render_buffer* vertices,
                                       const rive_ffi_render_buffer* uv_coords,
                                       const rive_ffi_render_buffer* indices,
                                       uint32_t vertex_count,
                                       uint32_t index_count,
                                       uint8_t blend_mode,
                                       float opacity);
void rive_ffi_renderer_modulate_opacity(rive_ffi_renderer*, float opacity);

#ifdef __cplusplus
}
#endif
