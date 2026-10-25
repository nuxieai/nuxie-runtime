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
#ifdef DF
#if __VERSION__>=310
layout(binding=0,std140) uniform Pk{uniform highp vec4 Zi;}aj;
#else
uniform mediump vec4 EF;
#endif
#endif
#ifdef GL_EXT_shader_pixel_local_storage
#ifdef LE
__pixel_local_inEXT j2
#else
__pixel_local_outEXT j2
#endif
{layout(rgba8) mediump vec4 n0;layout(r32ui) highp uint m0;layout(rgba8) mediump vec4 K4;layout(r32ui) highp uint q8;};
#ifndef GL_ARM_shader_framebuffer_fetch
#ifdef FF
layout(location=0) inout mediump vec4 sc;
#endif
#endif
#ifdef LE
layout(location=0) out mediump vec4 sc;
#endif
void main(){
#ifdef DF
#if __VERSION__>=310
n0=aj.Zi;
#else
n0=EF;
#endif
#endif
#ifdef FF
#ifdef GL_ARM_shader_framebuffer_fetch
n0=gl_LastFragColorARM;
#else
n0=sc;
#endif
#endif
#ifdef ME
q8=0u;
#endif
#ifdef CG
m0=0u;
#endif
#ifdef LE
sc=n0;
#endif
}
#else
layout(location=0) out mediump vec4 bj;void main(){bj=vec4(0,1,0,1);}
#endif
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive