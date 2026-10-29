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
#ifdef DB
#extension GL_EXT_shader_pixel_local_storage:require
#ifdef GL_ARM_shader_framebuffer_fetch
#extension GL_ARM_shader_framebuffer_fetch:require
#else
#extension GL_EXT_shader_framebuffer_fetch:require
#endif
#ifdef CF
#if __VERSION__>=310
layout(binding=0,std140) uniform Qk{uniform highp vec4 cj;}dj;
#else
uniform mediump vec4 DF;
#endif
#endif
#ifdef GL_EXT_shader_pixel_local_storage
#ifdef KE
__pixel_local_inEXT k2
#else
__pixel_local_outEXT k2
#endif
{layout(rgba8) mediump vec4 o0;layout(r32ui) highp uint m0;layout(rgba8) mediump vec4 M4;layout(r32ui) highp uint p8;};
#ifndef GL_ARM_shader_framebuffer_fetch
#ifdef EF
layout(location=0) inout mediump vec4 xc;
#endif
#endif
#ifdef KE
layout(location=0) out mediump vec4 xc;
#endif
void main(){
#ifdef CF
#if __VERSION__>=310
o0=dj.cj;
#else
o0=DF;
#endif
#endif
#ifdef EF
#ifdef GL_ARM_shader_framebuffer_fetch
o0=gl_LastFragColorARM;
#else
o0=xc;
#endif
#endif
#ifdef LE
p8=0u;
#endif
#ifdef BG
m0=0u;
#endif
#ifdef KE
xc=o0;
#endif
}
#else
layout(location=0) out mediump vec4 ej;void main(){ej=vec4(0,1,0,1);}
#endif
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive