#pragma once

#include "pls_load_store_ext.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char pls_load_store_ext[] = R"===(#ifdef CB
void main(){gl_Position=vec4(mix(vec2(-1,1),vec2(1,-1),equal(gl_VertexID&ivec2(1,2),ivec2(0))),0,1);
#ifdef RC
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
#ifdef WE
#if __VERSION__>=310
layout(binding=0,std140)uniform lj{uniform highp vec4 wh;}xh;
#else
uniform mediump vec4 XE;
#endif
#endif
#ifdef GL_EXT_shader_pixel_local_storage
#ifdef EE
__pixel_local_inEXT U1
#else
__pixel_local_outEXT U1
#endif
{layout(rgba8)mediump vec4 l0;layout(r32ui)highp uint i0;layout(rgba8)mediump vec4 o4;layout(r32ui)highp uint I7;};
#ifndef GL_ARM_shader_framebuffer_fetch
#ifdef YE
layout(location=0)inout mediump vec4 fb;
#endif
#endif
#ifdef EE
layout(location=0)out mediump vec4 fb;
#endif
void main(){
#ifdef WE
#if __VERSION__>=310
l0=xh.wh;
#else
l0=XE;
#endif
#endif
#ifdef YE
#ifdef GL_ARM_shader_framebuffer_fetch
l0=gl_LastFragColorARM;
#else
l0=fb;
#endif
#endif
#ifdef FE
I7=0u;
#endif
#ifdef XF
i0=0u;
#endif
#ifdef EE
fb=l0;
#endif
}
#else
layout(location=0)out mediump vec4 yh;void main(){yh=vec4(0,1,0,1);}
#endif
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive