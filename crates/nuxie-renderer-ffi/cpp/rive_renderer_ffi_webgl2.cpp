#include "rive_renderer_ffi_private.hpp"

#include "rive/renderer/gl/render_context_gl_impl.hpp"
#include "rive/renderer/gl/render_target_gl.hpp"

#include <emscripten/html5.h>

#include <cstring>
#include <string>

namespace
{
class rive_ffi_webgl2_context final : public rive_ffi_context
{
public:
    explicit rive_ffi_webgl2_context(EMSCRIPTEN_WEBGL_CONTEXT_HANDLE handle) :
        m_handle(handle)
    {}

    ~rive_ffi_webgl2_context() override
    {
        renderer.reset();
        target.reset();
        context.reset();
        if (m_handle > 0)
        {
            emscripten_webgl_destroy_context(m_handle);
        }
    }

    bool ensureTarget(uint32_t nextWidth, uint32_t nextHeight) override
    {
        if (context == nullptr || nextWidth == 0 || nextHeight == 0)
        {
            return false;
        }
        if (target != nullptr && width == nextWidth && height == nextHeight)
        {
            return true;
        }
        if (emscripten_set_canvas_element_size(
                "#canvas",
                static_cast<int>(nextWidth),
                static_cast<int>(nextHeight)) != EMSCRIPTEN_RESULT_SUCCESS)
        {
            return false;
        }
        width = nextWidth;
        height = nextHeight;
        glBindFramebuffer(GL_FRAMEBUFFER, 0);
        glViewport(0, 0, width, height);
        GLint samples = 0;
        glGetIntegerv(GL_SAMPLES, &samples);
        target = rive::make_rcp<rive::gpu::FramebufferRenderTargetGL>(
            width,
            height,
            0,
            static_cast<uint32_t>(samples));
        context->static_impl_cast<rive::gpu::RenderContextGLImpl>()
            ->invalidateGLState();
        return target != nullptr;
    }

    bool afterFlush() override
    {
        auto* glContext =
            context->static_impl_cast<rive::gpu::RenderContextGLImpl>();
        glContext->unbindGLInternalResources();
        static_cast<rive::gpu::RenderTargetGL*>(target.get())
            ->bindDestinationFramebuffer(GL_READ_FRAMEBUFFER);
        glFinish();
        return glGetError() == GL_NO_ERROR;
    }

    const char* adapterName() const override { return m_adapterName.c_str(); }

    size_t readPixels(uint8_t* out, size_t len) override
    {
        const size_t required = static_cast<size_t>(width) * height * 4;
        if (out == nullptr || len < required || target == nullptr)
        {
            return 0;
        }
        static_cast<rive::gpu::RenderTargetGL*>(target.get())
            ->bindDestinationFramebuffer(GL_READ_FRAMEBUFFER);
        glPixelStorei(GL_PACK_ALIGNMENT, 1);
        glReadPixels(0,
                     0,
                     static_cast<GLsizei>(width),
                     static_cast<GLsizei>(height),
                     GL_RGBA,
                     GL_UNSIGNED_BYTE,
                     out);
        return glGetError() == GL_NO_ERROR ? required : 0;
    }

    void captureAdapterName()
    {
        const auto* rendererString = glGetString(GL_RENDERER);
        m_adapterName = rendererString == nullptr
                            ? ""
                            : reinterpret_cast<const char*>(rendererString);
    }

private:
    EMSCRIPTEN_WEBGL_CONTEXT_HANDLE m_handle;
    std::string m_adapterName;
};
} // namespace

extern "C" rive_ffi_context* rive_ffi_context_make_webgl2(uint32_t width,
                                                           uint32_t height)
{
    if (width == 0 || height == 0 ||
        emscripten_set_canvas_element_size(
            "#canvas", static_cast<int>(width), static_cast<int>(height)) !=
            EMSCRIPTEN_RESULT_SUCCESS)
    {
        return nullptr;
    }

    EmscriptenWebGLContextAttributes attributes;
    emscripten_webgl_init_context_attributes(&attributes);
    attributes.alpha = true;
    attributes.depth = false;
    attributes.stencil = false;
    attributes.antialias = false;
    attributes.premultipliedAlpha = true;
    attributes.preserveDrawingBuffer = true;
    attributes.enableExtensionsByDefault = true;
    attributes.majorVersion = 2;
    attributes.minorVersion = 0;
    attributes.powerPreference = EM_WEBGL_POWER_PREFERENCE_HIGH_PERFORMANCE;

    const EMSCRIPTEN_WEBGL_CONTEXT_HANDLE handle =
        emscripten_webgl_create_context("#canvas", &attributes);
    if (handle <= 0 ||
        emscripten_webgl_make_context_current(handle) !=
            EMSCRIPTEN_RESULT_SUCCESS)
    {
        if (handle > 0)
        {
            emscripten_webgl_destroy_context(handle);
        }
        return nullptr;
    }

    auto* ctx = new rive_ffi_webgl2_context(handle);
    ctx->context = rive::gpu::RenderContextGLImpl::MakeContext({});
    if (ctx->context == nullptr || !ctx->ensureTarget(width, height))
    {
        delete ctx;
        return nullptr;
    }
    ctx->captureAdapterName();
    if (std::strlen(ctx->adapterName()) == 0)
    {
        delete ctx;
        return nullptr;
    }
    return ctx;
}
