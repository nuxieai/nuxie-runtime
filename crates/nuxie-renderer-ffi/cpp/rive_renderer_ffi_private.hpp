#pragma once

#include "rive_renderer_ffi.h"

#include "rive/renderer.hpp"
#ifdef RIVE_FFI_DECODE_ORACLE
#include "rive/decoders/bitmap_decoder.hpp"
#endif
#include "rive/renderer/render_context.hpp"
#include "rive/renderer/render_target.hpp"
#include "rive/span.hpp"

#include <memory>
#include <stddef.h>
#include <stdint.h>
#include <vector>

struct rive_ffi_context
{
    std::unique_ptr<rive::gpu::RenderContext> context;
    rive::rcp<rive::gpu::RenderTarget> target;
    std::unique_ptr<rive::Renderer> renderer;
    uint32_t width = 0;
    uint32_t height = 0;
    uint64_t drawCount = 0;

    virtual ~rive_ffi_context() = default;
    virtual bool ensureTarget(uint32_t width, uint32_t height) = 0;
    virtual void beforeFlush(rive::gpu::RenderContext::FlushResources&) {}
    virtual void afterFlush() {}
    virtual size_t readPixels(uint8_t*, size_t) { return 0; }
};

struct rive_ffi_renderer
{
    rive_ffi_context* context;
};

struct rive_ffi_render_shader
{
    rive::rcp<rive::RenderShader> shader;
};

struct rive_ffi_render_path
{
    rive::rcp<rive::RenderPath> path;
};

struct rive_ffi_render_paint
{
    rive::rcp<rive::RenderPaint> paint;
};

struct rive_ffi_render_image
{
    rive::rcp<rive::RenderImage> image;
};

#ifdef RIVE_FFI_DECODE_ORACLE
struct rive_ffi_decoded_bitmap
{
    std::unique_ptr<Bitmap> bitmap;
};
#endif

struct rive_ffi_render_buffer
{
    rive::rcp<rive::RenderBuffer> buffer;
    std::vector<uint8_t> shadow;
};
