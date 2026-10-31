#pragma once

#include "pls_load_store_ext.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char pls_load_store_ext[] = R"===(#ifdef BB
void main(){gl_Position=vec4(mix(vec2(-1,1),vec2(1,-1),equal(gl_VertexID&ivec2(1,2),ivec2(0))),0,1);
#ifdef OC
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
layout(binding=0,std140) uniform al{uniform highp vec4 nj;}oj;
#else
uniform mediump vec4 EF;
#endif
#endif
#ifdef GL_EXT_shader_pixel_local_storage
#ifdef KE
__pixel_local_inEXT V1
#else
__pixel_local_outEXT V1
#endif
{layout(rgba8) mediump vec4 p0;layout(r32ui) highp uint n0;layout(rgba8) mediump vec4 S4;layout(r32ui) highp uint x8;};
#ifndef GL_ARM_shader_framebuffer_fetch
#ifdef FF
layout(location=0) inout mediump vec4 Bc;
#endif
#endif
#ifdef KE
layout(location=0) out mediump vec4 Bc;
#endif
void main(){
#ifdef DF
#if __VERSION__>=310
p0=oj.nj;
#else
p0=EF;
#endif
#endif
#ifdef FF
#ifdef GL_ARM_shader_framebuffer_fetch
p0=gl_LastFragColorARM;
#else
p0=Bc;
#endif
#endif
#ifdef LE
x8=0u;
#endif
#ifdef DG
n0=0u;
#endif
#ifdef KE
Bc=p0;
#endif
}
#else
layout(location=0) out mediump vec4 pj;void main(){pj=vec4(0,1,0,1);}
#endif
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive