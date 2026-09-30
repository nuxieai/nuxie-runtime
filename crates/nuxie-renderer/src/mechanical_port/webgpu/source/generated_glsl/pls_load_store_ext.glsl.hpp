#pragma once

#include "pls_load_store_ext.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char pls_load_store_ext[] = R"===(#ifdef CB
void main(){gl_Position=vec4(mix(vec2(-1,1),vec2(1,-1),equal(gl_VertexID&ivec2(1,2),ivec2(0))),0,1);
#ifdef SC
gl_Position.y=-gl_Position.y;
#endif
}
#endif
#ifdef EB
#extension GL_EXT_shader_pixel_local_storage:require
#ifdef GL_ARM_shader_framebuffer_fetch
#extension GL_ARM_shader_framebuffer_fetch:require
#else
#extension GL_EXT_shader_framebuffer_fetch:require
#endif
#ifdef XE
#if __VERSION__>=310
layout(binding=0,std140) uniform sj{uniform highp vec4 zh;}Ah;
#else
uniform mediump vec4 YE;
#endif
#endif
#ifdef GL_EXT_shader_pixel_local_storage
#ifdef FE
__pixel_local_inEXT V1
#else
__pixel_local_outEXT V1
#endif
{layout(rgba8) mediump vec4 l0;layout(r32ui) highp uint i0;layout(rgba8) mediump vec4 q4;layout(r32ui) highp uint F7;};
#ifndef GL_ARM_shader_framebuffer_fetch
#ifdef ZE
layout(location=0) inout mediump vec4 cb;
#endif
#endif
#ifdef FE
layout(location=0) out mediump vec4 cb;
#endif
void main(){
#ifdef XE
#if __VERSION__>=310
l0=Ah.zh;
#else
l0=YE;
#endif
#endif
#ifdef ZE
#ifdef GL_ARM_shader_framebuffer_fetch
l0=gl_LastFragColorARM;
#else
l0=cb;
#endif
#endif
#ifdef GE
F7=0u;
#endif
#ifdef XF
i0=0u;
#endif
#ifdef FE
cb=l0;
#endif
}
#else
layout(location=0) out mediump vec4 Bh;void main(){Bh=vec4(0,1,0,1);}
#endif
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive