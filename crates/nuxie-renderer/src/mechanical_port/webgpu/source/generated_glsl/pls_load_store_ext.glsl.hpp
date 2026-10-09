#pragma once

#include "pls_load_store_ext.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char pls_load_store_ext[] = R"===(#ifdef BB
void main(){gl_Position=vec4(mix(vec2(-1,1),vec2(1,-1),equal(gl_VertexID&ivec2(1,2),ivec2(0))),0,1);
#ifdef MC
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
#ifdef ZE
#if __VERSION__>=310
layout(binding=0,std140) uniform Jk{uniform highp vec4 Pi;}Qi;
#else
uniform mediump vec4 AF;
#endif
#endif
#ifdef GL_EXT_shader_pixel_local_storage
#ifdef GE
__pixel_local_inEXT h2
#else
__pixel_local_outEXT h2
#endif
{layout(rgba8) mediump vec4 n0;layout(r32ui) highp uint m0;layout(rgba8) mediump vec4 I4;layout(r32ui) highp uint m8;};
#ifndef GL_ARM_shader_framebuffer_fetch
#ifdef BF
layout(location=0) inout mediump vec4 nc;
#endif
#endif
#ifdef GE
layout(location=0) out mediump vec4 nc;
#endif
void main(){
#ifdef ZE
#if __VERSION__>=310
n0=Qi.Pi;
#else
n0=AF;
#endif
#endif
#ifdef BF
#ifdef GL_ARM_shader_framebuffer_fetch
n0=gl_LastFragColorARM;
#else
n0=nc;
#endif
#endif
#ifdef HE
m8=0u;
#endif
#ifdef ZF
m0=0u;
#endif
#ifdef GE
nc=n0;
#endif
}
#else
layout(location=0) out mediump vec4 Ri;void main(){Ri=vec4(0,1,0,1);}
#endif
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive