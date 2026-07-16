#include "rive_renderer_ffi_private.hpp"

#include "render_context_null.hpp"
#include "rive/math/raw_path.hpp"
#include "rive/renderer/rive_renderer.hpp"

#include <algorithm>
#include <cstring>

class RenderContextTest
{
public:
    static uint64_t logicalFlushCount(const rive::gpu::RenderContext* context)
    {
        return context == nullptr ? 0 : context->m_logicalFlushes.size();
    }
};

namespace
{
class rive_ffi_null_context : public rive_ffi_context
{
public:
    bool ensureTarget(uint32_t nextWidth, uint32_t nextHeight) override
    {
        if (context == nullptr)
        {
            return false;
        }
        if (target != nullptr && width == nextWidth && height == nextHeight)
        {
            return true;
        }
        width = nextWidth;
        height = nextHeight;
        target = context->static_impl_cast<RenderContextNULL>()->makeRenderTarget(
            width,
            height);
        return target != nullptr;
    }
};
} // namespace

static rive::Mat2D to_mat2d(rive_ffi_mat2d matrix)
{
    return rive::Mat2D(matrix.values[0],
                       matrix.values[1],
                       matrix.values[2],
                       matrix.values[3],
                       matrix.values[4],
                       matrix.values[5]);
}

static rive::FillRule to_fill_rule(uint8_t value)
{
    switch (value)
    {
        case 1:
            return rive::FillRule::evenOdd;
        case 2:
            return rive::FillRule::clockwise;
        default:
            return rive::FillRule::nonZero;
    }
}

static rive::ImageSampler to_image_sampler(uint8_t value)
{
    return rive::ImageSampler::SamplerFromKey(value);
}

static rive::RawPath to_raw_path(const uint8_t* verbs,
                                 size_t verbCount,
                                 const rive_ffi_vec2d* points,
                                 size_t pointCount)
{
    rive::RawPath path;
    size_t pointIndex = 0;
    for (size_t i = 0; i < verbCount; ++i)
    {
        switch (static_cast<rive::PathVerb>(verbs[i]))
        {
            case rive::PathVerb::move:
                if (pointIndex + 1 <= pointCount)
                {
                    path.moveTo(points[pointIndex].x, points[pointIndex].y);
                    pointIndex += 1;
                }
                break;
            case rive::PathVerb::line:
                if (pointIndex + 1 <= pointCount)
                {
                    path.lineTo(points[pointIndex].x, points[pointIndex].y);
                    pointIndex += 1;
                }
                break;
            case rive::PathVerb::quad:
                if (pointIndex + 2 <= pointCount)
                {
                    path.quadTo(points[pointIndex].x,
                                points[pointIndex].y,
                                points[pointIndex + 1].x,
                                points[pointIndex + 1].y);
                    pointIndex += 2;
                }
                break;
            case rive::PathVerb::cubic:
                if (pointIndex + 3 <= pointCount)
                {
                    path.cubicTo(points[pointIndex].x,
                                 points[pointIndex].y,
                                 points[pointIndex + 1].x,
                                 points[pointIndex + 1].y,
                                 points[pointIndex + 2].x,
                                 points[pointIndex + 2].y);
                    pointIndex += 3;
                }
                break;
            case rive::PathVerb::close:
                path.close();
                break;
        }
    }
    return path;
}

extern "C" rive_ffi_context* rive_ffi_context_make_null(uint32_t width,
                                                        uint32_t height)
{
    auto* ctx = new rive_ffi_null_context;
    ctx->context = RenderContextNULL::MakeContext();
    if (!ctx->ensureTarget(width, height))
    {
        delete ctx;
        return nullptr;
    }
    return ctx;
}

#if !defined(__APPLE__) || !defined(RIVE_FFI_HAS_METAL)
extern "C" rive_ffi_context* rive_ffi_context_make_metal(uint32_t, uint32_t)
{
    return nullptr;
}
#endif

#if !defined(RIVE_FFI_HAS_DAWN)
extern "C" rive_ffi_context* rive_ffi_context_make_dawn(uint32_t, uint32_t)
{
    return nullptr;
}
#endif

extern "C" void rive_ffi_context_delete(rive_ffi_context* ctx) { delete ctx; }

extern "C" int rive_ffi_context_begin_frame(rive_ffi_context* ctx,
                                            uint32_t width,
                                            uint32_t height,
                                            uint32_t clearColor)
{
    return rive_ffi_context_begin_frame_mode(ctx,
                                             width,
                                             height,
                                             clearColor,
                                             0);
}

extern "C" int rive_ffi_context_begin_frame_mode(rive_ffi_context* ctx,
                                                 uint32_t width,
                                                 uint32_t height,
                                                 uint32_t clearColor,
                                                 uint32_t mode)
{
    return rive_ffi_context_begin_frame_mode_metrics(
        ctx, width, height, clearColor, mode, 0);
}

extern "C" int rive_ffi_context_begin_frame_mode_metrics(
    rive_ffi_context* ctx,
    uint32_t width,
    uint32_t height,
    uint32_t clearColor,
    uint32_t mode,
    uint32_t collectWorkMetrics)
{
    if (ctx == nullptr || ctx->context == nullptr)
    {
        return 0;
    }
    if (!ctx->ensureTarget(width, height))
    {
        return 0;
    }

    rive::gpu::RenderContext::FrameDescriptor desc;
    desc.renderTargetWidth = width;
    desc.renderTargetHeight = height;
    desc.clearColor = clearColor;
    if (mode == 1)
    {
        desc.msaaSampleCount = 4;
    }
    else if (mode == 2)
    {
        desc.disableRasterOrdering = true;
        desc.clockwiseFillOverride = true;
    }
    else if (mode != 0)
    {
        return 0;
    }
    ctx->context->beginFrame(desc);
    ctx->beginBackendWorkMetrics(collectWorkMetrics != 0);
    ctx->renderer = std::make_unique<rive::RiveRenderer>(ctx->context.get());
    ctx->drawCount = 0;
    ctx->lastLogicalFlushCount = 0;
    return 1;
}

extern "C" int rive_ffi_context_end_frame(rive_ffi_context* ctx)
{
    if (ctx == nullptr || ctx->context == nullptr)
    {
        return 0;
    }
    ctx->renderer.reset();
    ctx->lastLogicalFlushCount =
        RenderContextTest::logicalFlushCount(ctx->context.get());
    rive::gpu::RenderContext::FlushResources resources;
    resources.renderTarget = ctx->target.get();
    ctx->beforeFlush(resources);
    ctx->context->flush(resources);
    return ctx->afterFlush() ? 1 : 0;
}

extern "C" size_t rive_ffi_context_read_pixels(rive_ffi_context* ctx,
                                               uint8_t* out,
                                               size_t len)
{
    if (ctx == nullptr || out == nullptr)
    {
        return 0;
    }
    return ctx->readPixels(out, len);
}

extern "C" uint64_t rive_ffi_context_draw_count(const rive_ffi_context* ctx)
{
    return ctx == nullptr ? 0 : ctx->drawCount;
}

extern "C" uint64_t
rive_ffi_context_logical_flush_count(const rive_ffi_context* ctx)
{
    return ctx == nullptr ? 0 : ctx->lastLogicalFlushCount;
}

extern "C" void rive_ffi_context_backend_work_metrics(
    const rive_ffi_context* ctx,
    rive_ffi_backend_work_metrics* out)
{
    if (out != nullptr)
    {
        *out = ctx == nullptr ? rive_ffi_backend_work_metrics{}
                              : ctx->lastBackendWorkMetrics;
    }
}

extern "C" size_t
rive_ffi_context_adapter_name(const rive_ffi_context* ctx,
                              char* out,
                              size_t len)
{
    const char* name = ctx == nullptr ? "" : ctx->adapterName();
    const size_t required = std::strlen(name);
    if (out != nullptr && len != 0)
    {
        const size_t copied = std::min(required, len - 1);
        std::memcpy(out, name, copied);
        out[copied] = '\0';
    }
    return required;
}

#if !defined(__APPLE__) || !defined(RIVE_FFI_HAS_METAL)
extern "C" int
rive_ffi_metal_adapter_identity(rive_ffi_adapter_identity*)
{
    return 0;
}
#endif

extern "C" rive_ffi_renderer* rive_ffi_context_renderer(rive_ffi_context* ctx)
{
    if (ctx == nullptr || ctx->renderer == nullptr)
    {
        return nullptr;
    }
    return new rive_ffi_renderer{ctx};
}

extern "C" void rive_ffi_renderer_delete(rive_ffi_renderer* renderer)
{
    delete renderer;
}

extern "C" rive_ffi_render_shader* rive_ffi_make_linear_gradient(
    rive_ffi_context* ctx,
    float sx,
    float sy,
    float ex,
    float ey,
    const uint32_t* colors,
    const float* stops,
    size_t count)
{
    if (ctx == nullptr || ctx->context == nullptr)
    {
        return nullptr;
    }
    return new rive_ffi_render_shader{
        ctx->context->makeLinearGradient(sx, sy, ex, ey, colors, stops, count)};
}

extern "C" rive_ffi_render_shader* rive_ffi_make_radial_gradient(
    rive_ffi_context* ctx,
    float cx,
    float cy,
    float radius,
    const uint32_t* colors,
    const float* stops,
    size_t count)
{
    if (ctx == nullptr || ctx->context == nullptr)
    {
        return nullptr;
    }
    return new rive_ffi_render_shader{
        ctx->context->makeRadialGradient(cx, cy, radius, colors, stops, count)};
}

extern "C" void rive_ffi_render_shader_delete(rive_ffi_render_shader* shader)
{
    delete shader;
}

extern "C" rive_ffi_render_path* rive_ffi_make_render_path(
    rive_ffi_context* ctx,
    const uint8_t* verbs,
    size_t verbCount,
    const rive_ffi_vec2d* points,
    size_t pointCount,
    uint8_t fillRule)
{
    if (ctx == nullptr || ctx->context == nullptr)
    {
        return nullptr;
    }
    auto rawPath = to_raw_path(verbs, verbCount, points, pointCount);
    return new rive_ffi_render_path{
        ctx->context->makeRenderPath(rawPath, to_fill_rule(fillRule))};
}

extern "C" rive_ffi_render_path* rive_ffi_make_empty_render_path(
    rive_ffi_context* ctx)
{
    if (ctx == nullptr || ctx->context == nullptr)
    {
        return nullptr;
    }
    return new rive_ffi_render_path{ctx->context->makeEmptyRenderPath()};
}

extern "C" void rive_ffi_render_path_delete(rive_ffi_render_path* path)
{
    delete path;
}

extern "C" void rive_ffi_render_path_rewind(rive_ffi_render_path* path)
{
    if (path != nullptr)
    {
        path->path->rewind();
    }
}

extern "C" void rive_ffi_render_path_fill_rule(rive_ffi_render_path* path,
                                               uint8_t fillRule)
{
    if (path != nullptr)
    {
        path->path->fillRule(to_fill_rule(fillRule));
    }
}

extern "C" void rive_ffi_render_path_add_render_path(
    rive_ffi_render_path* path,
    const rive_ffi_render_path* source,
    rive_ffi_mat2d transform)
{
    if (path != nullptr && source != nullptr)
    {
        path->path->addRenderPath(source->path.get(), to_mat2d(transform));
    }
}

extern "C" void rive_ffi_render_path_add_render_path_backwards(
    rive_ffi_render_path* path,
    const rive_ffi_render_path* source,
    rive_ffi_mat2d transform)
{
    if (path != nullptr && source != nullptr)
    {
        path->path->addRenderPathBackwards(source->path.get(),
                                           to_mat2d(transform));
    }
}

extern "C" void rive_ffi_render_path_add_raw_path(
    rive_ffi_render_path* path,
    const uint8_t* verbs,
    size_t verbCount,
    const rive_ffi_vec2d* points,
    size_t pointCount)
{
    if (path != nullptr)
    {
        auto rawPath = to_raw_path(verbs, verbCount, points, pointCount);
        path->path->addRawPath(rawPath);
    }
}

extern "C" void rive_ffi_render_path_move_to(rive_ffi_render_path* path,
                                             float x,
                                             float y)
{
    if (path != nullptr)
    {
        path->path->moveTo(x, y);
    }
}

extern "C" void rive_ffi_render_path_line_to(rive_ffi_render_path* path,
                                             float x,
                                             float y)
{
    if (path != nullptr)
    {
        path->path->lineTo(x, y);
    }
}

extern "C" void rive_ffi_render_path_cubic_to(rive_ffi_render_path* path,
                                              float ox,
                                              float oy,
                                              float ix,
                                              float iy,
                                              float x,
                                              float y)
{
    if (path != nullptr)
    {
        path->path->cubicTo(ox, oy, ix, iy, x, y);
    }
}

extern "C" void rive_ffi_render_path_close(rive_ffi_render_path* path)
{
    if (path != nullptr)
    {
        path->path->close();
    }
}

extern "C" rive_ffi_render_paint* rive_ffi_make_render_paint(
    rive_ffi_context* ctx)
{
    if (ctx == nullptr || ctx->context == nullptr)
    {
        return nullptr;
    }
    return new rive_ffi_render_paint{ctx->context->makeRenderPaint()};
}

extern "C" void rive_ffi_render_paint_delete(rive_ffi_render_paint* paint)
{
    delete paint;
}

extern "C" void rive_ffi_render_paint_style(rive_ffi_render_paint* paint,
                                            uint8_t style)
{
    if (paint != nullptr)
    {
        paint->paint->style(style == 0 ? rive::RenderPaintStyle::stroke
                                       : rive::RenderPaintStyle::fill);
    }
}

extern "C" void rive_ffi_render_paint_color(rive_ffi_render_paint* paint,
                                            uint32_t color)
{
    if (paint != nullptr)
    {
        paint->paint->color(color);
    }
}

extern "C" void rive_ffi_render_paint_thickness(rive_ffi_render_paint* paint,
                                                float thickness)
{
    if (paint != nullptr)
    {
        paint->paint->thickness(thickness);
    }
}

extern "C" void rive_ffi_render_paint_join(rive_ffi_render_paint* paint,
                                           uint32_t join)
{
    if (paint != nullptr)
    {
        paint->paint->join(static_cast<rive::StrokeJoin>(join));
    }
}

extern "C" void rive_ffi_render_paint_cap(rive_ffi_render_paint* paint,
                                          uint32_t cap)
{
    if (paint != nullptr)
    {
        paint->paint->cap(static_cast<rive::StrokeCap>(cap));
    }
}

extern "C" void rive_ffi_render_paint_feather(rive_ffi_render_paint* paint,
                                              float feather)
{
    if (paint != nullptr)
    {
        paint->paint->feather(feather);
    }
}

extern "C" void rive_ffi_render_paint_blend_mode(rive_ffi_render_paint* paint,
                                                 uint8_t blendMode)
{
    if (paint != nullptr)
    {
        paint->paint->blendMode(static_cast<rive::BlendMode>(blendMode));
    }
}

extern "C" void rive_ffi_render_paint_shader(
    rive_ffi_render_paint* paint,
    const rive_ffi_render_shader* shader)
{
    if (paint != nullptr)
    {
        paint->paint->shader(shader == nullptr ? nullptr : shader->shader);
    }
}

extern "C" void rive_ffi_render_paint_invalidate_stroke(
    rive_ffi_render_paint* paint)
{
    if (paint != nullptr)
    {
        paint->paint->invalidateStroke();
    }
}

extern "C" rive_ffi_render_image* rive_ffi_decode_image(rive_ffi_context* ctx,
                                                        const uint8_t* bytes,
                                                        size_t len)
{
    if (ctx == nullptr || ctx->context == nullptr)
    {
        return nullptr;
    }
    return new rive_ffi_render_image{
        ctx->context->decodeImage(rive::Span<const uint8_t>(bytes, len))};
}

extern "C" void rive_ffi_render_image_delete(rive_ffi_render_image* image)
{
    delete image;
}

extern "C" uint32_t rive_ffi_render_image_width(
    const rive_ffi_render_image* image)
{
    return image == nullptr || image->image == nullptr ? 0 : image->image->width();
}

extern "C" uint32_t rive_ffi_render_image_height(
    const rive_ffi_render_image* image)
{
    return image == nullptr || image->image == nullptr ? 0
                                                       : image->image->height();
}

#ifdef RIVE_FFI_DECODE_ORACLE
extern "C" rive_ffi_decoded_bitmap* rive_ffi_decode_bitmap_rgba(
    const uint8_t* bytes,
    size_t len)
{
    if (bytes == nullptr)
    {
        return nullptr;
    }
    auto bitmap = Bitmap::decode(bytes, len);
    if (bitmap == nullptr)
    {
        return nullptr;
    }
    if (bitmap->pixelFormat() != Bitmap::PixelFormat::RGBAPremul)
    {
        bitmap->pixelFormat(Bitmap::PixelFormat::RGBAPremul);
    }
    return new rive_ffi_decoded_bitmap{std::move(bitmap)};
}

extern "C" void rive_ffi_decoded_bitmap_delete(
    rive_ffi_decoded_bitmap* bitmap)
{
    delete bitmap;
}

extern "C" uint32_t rive_ffi_decoded_bitmap_width(
    const rive_ffi_decoded_bitmap* bitmap)
{
    return bitmap == nullptr || bitmap->bitmap == nullptr
               ? 0
               : bitmap->bitmap->width();
}

extern "C" uint32_t rive_ffi_decoded_bitmap_height(
    const rive_ffi_decoded_bitmap* bitmap)
{
    return bitmap == nullptr || bitmap->bitmap == nullptr
               ? 0
               : bitmap->bitmap->height();
}

extern "C" size_t rive_ffi_decoded_bitmap_copy_bytes(
    const rive_ffi_decoded_bitmap* bitmap,
    uint8_t* out,
    size_t len)
{
    if (bitmap == nullptr || bitmap->bitmap == nullptr || out == nullptr)
    {
        return 0;
    }
    const size_t byteCount = bitmap->bitmap->numBytes();
    if (len < byteCount)
    {
        return byteCount;
    }
    std::copy_n(bitmap->bitmap->bytes(), byteCount, out);
    return byteCount;
}
#endif

extern "C" rive_ffi_render_buffer* rive_ffi_make_render_buffer(
    rive_ffi_context* ctx,
    uint8_t bufferType,
    uint8_t flags,
    size_t sizeInBytes)
{
    if (ctx == nullptr || ctx->context == nullptr)
    {
        return nullptr;
    }
    auto* buffer = new rive_ffi_render_buffer;
    buffer->buffer = ctx->context->makeRenderBuffer(
        static_cast<rive::RenderBufferType>(bufferType),
        static_cast<rive::RenderBufferFlags>(flags),
        sizeInBytes);
    buffer->shadow.resize(sizeInBytes);
    return buffer;
}

extern "C" void rive_ffi_render_buffer_delete(rive_ffi_render_buffer* buffer)
{
    delete buffer;
}

extern "C" void rive_ffi_render_buffer_write(rive_ffi_render_buffer* buffer,
                                             const uint8_t* bytes,
                                             size_t len)
{
    if (buffer == nullptr || buffer->buffer == nullptr)
    {
        return;
    }
    len = std::min(len, buffer->shadow.size());
    std::copy(bytes, bytes + len, buffer->shadow.begin());
    auto* mapped = static_cast<uint8_t*>(buffer->buffer->map());
    std::copy(buffer->shadow.begin(), buffer->shadow.begin() + len, mapped);
    buffer->buffer->unmap();
}

extern "C" void rive_ffi_renderer_save(rive_ffi_renderer* renderer)
{
    if (renderer != nullptr && renderer->context->renderer != nullptr)
    {
        renderer->context->renderer->save();
    }
}

extern "C" void rive_ffi_renderer_restore(rive_ffi_renderer* renderer)
{
    if (renderer != nullptr && renderer->context->renderer != nullptr)
    {
        renderer->context->renderer->restore();
    }
}

extern "C" void rive_ffi_renderer_transform(rive_ffi_renderer* renderer,
                                            rive_ffi_mat2d matrix)
{
    if (renderer != nullptr && renderer->context->renderer != nullptr)
    {
        renderer->context->renderer->transform(to_mat2d(matrix));
    }
}

extern "C" void rive_ffi_renderer_draw_path(rive_ffi_renderer* renderer,
                                            rive_ffi_render_path* path,
                                            rive_ffi_render_paint* paint)
{
    if (renderer != nullptr && renderer->context->renderer != nullptr &&
        path != nullptr && paint != nullptr)
    {
        renderer->context->renderer->drawPath(path->path.get(),
                                              paint->paint.get());
        renderer->context->drawCount += 1;
    }
}

extern "C" void rive_ffi_renderer_clip_path(rive_ffi_renderer* renderer,
                                            rive_ffi_render_path* path)
{
    if (renderer != nullptr && renderer->context->renderer != nullptr &&
        path != nullptr)
    {
        renderer->context->renderer->clipPath(path->path.get());
    }
}

extern "C" void rive_ffi_renderer_draw_image(
    rive_ffi_renderer* renderer,
    const rive_ffi_render_image* image,
    uint8_t sampler,
    uint8_t blendMode,
    float opacity)
{
    if (renderer != nullptr && renderer->context->renderer != nullptr)
    {
        renderer->context->renderer->drawImage(
            image == nullptr ? nullptr : image->image.get(),
            to_image_sampler(sampler),
            static_cast<rive::BlendMode>(blendMode),
            opacity);
        renderer->context->drawCount += 1;
    }
}

extern "C" void rive_ffi_renderer_draw_image_mesh(
    rive_ffi_renderer* renderer,
    const rive_ffi_render_image* image,
    uint8_t sampler,
    const rive_ffi_render_buffer* vertices,
    const rive_ffi_render_buffer* uvCoords,
    const rive_ffi_render_buffer* indices,
    uint32_t vertexCount,
    uint32_t indexCount,
    uint8_t blendMode,
    float opacity)
{
    if (renderer != nullptr && renderer->context->renderer != nullptr)
    {
        renderer->context->renderer->drawImageMesh(
            image == nullptr ? nullptr : image->image.get(),
            to_image_sampler(sampler),
            vertices == nullptr ? nullptr : vertices->buffer,
            uvCoords == nullptr ? nullptr : uvCoords->buffer,
            indices == nullptr ? nullptr : indices->buffer,
            vertexCount,
            indexCount,
            static_cast<rive::BlendMode>(blendMode),
            opacity);
        renderer->context->drawCount += 1;
    }
}

extern "C" void rive_ffi_renderer_modulate_opacity(
    rive_ffi_renderer* renderer,
    float opacity)
{
    if (renderer != nullptr && renderer->context->renderer != nullptr)
    {
        renderer->context->renderer->modulateOpacity(opacity);
    }
}
