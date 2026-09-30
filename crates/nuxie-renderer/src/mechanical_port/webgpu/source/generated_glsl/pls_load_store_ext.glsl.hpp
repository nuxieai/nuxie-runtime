#pragma once

#include "pls_load_store_ext.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char pls_load_store_ext[] = R"===(#ifdef BB
void main(){gl_Position=vec4(mix(vec2(-1,1),vec2(1,-1),equal(gl_VertexID&ivec2(1,2),ivec2(0))),0,1);
#ifdef NC
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
#ifdef YE
#if __VERSION__>=310
layout(binding=0,std140) uniform Wj{uniform highp vec4 ei;}fi;
#else
uniform mediump vec4 ZE;
#endif
#endif
#ifdef GL_EXT_shader_pixel_local_storage
#ifdef FE
__pixel_local_inEXT a2
#else
__pixel_local_outEXT a2
#endif
{layout(rgba8) mediump vec4 n0;layout(r32ui) highp uint m0;layout(rgba8) mediump vec4 C4;layout(r32ui) highp uint S7;};
#ifndef GL_ARM_shader_framebuffer_fetch
#ifdef AF
layout(location=0) inout mediump vec4 yb;
#endif
#endif
#ifdef FE
layout(location=0) out mediump vec4 yb;
#endif
void main(){
#ifdef YE
#if __VERSION__>=310
n0=fi.ei;
#else
n0=ZE;
#endif
#endif
#ifdef AF
#ifdef GL_ARM_shader_framebuffer_fetch
n0=gl_LastFragColorARM;
#else
n0=yb;
#endif
#endif
#ifdef GE
S7=0u;
#endif
#ifdef YF
m0=0u;
#endif
#ifdef FE
yb=n0;
#endif
}
#else
layout(location=0) out mediump vec4 gi;void main(){gi=vec4(0,1,0,1);}
#endif
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive