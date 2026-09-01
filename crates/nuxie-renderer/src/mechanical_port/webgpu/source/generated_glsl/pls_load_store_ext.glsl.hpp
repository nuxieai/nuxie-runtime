#pragma once

#include "pls_load_store_ext.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char pls_load_store_ext[] = R"===(#ifdef DB
void main(){gl_Position=vec4(mix(vec2(-1,1),vec2(1,-1),equal(gl_VertexID&ivec2(1,2),ivec2(0))),0,1);
#ifdef SC
gl_Position.y=-gl_Position.y;
#endif
}
#endif
#ifdef GB
#extension GL_EXT_shader_pixel_local_storage:require
#ifdef GL_ARM_shader_framebuffer_fetch
#extension GL_ARM_shader_framebuffer_fetch:require
#else
#extension GL_EXT_shader_framebuffer_fetch:require
#endif
#ifdef SE
#if __VERSION__>=310
layout(binding=0,std140)uniform Hi{uniform highp vec4 Xg;}Yg;
#else
uniform mediump vec4 TE;
#endif
#endif
#ifdef GL_EXT_shader_pixel_local_storage
#ifdef CE
__pixel_local_inEXT R1
#else
__pixel_local_outEXT R1
#endif
{layout(rgba8)mediump vec4 j0;layout(r32ui)highp uint h0;layout(rgba8)mediump vec4 k4;layout(r32ui)highp uint G7;};
#ifndef GL_ARM_shader_framebuffer_fetch
#ifdef UE
layout(location=0)inout mediump vec4 Wa;
#endif
#endif
#ifdef CE
layout(location=0)out mediump vec4 Wa;
#endif
void main(){
#ifdef SE
#if __VERSION__>=310
j0=Yg.Xg;
#else
j0=TE;
#endif
#endif
#ifdef UE
#ifdef GL_ARM_shader_framebuffer_fetch
j0=gl_LastFragColorARM;
#else
j0=Wa;
#endif
#endif
#ifdef DE
G7=0u;
#endif
#ifdef SF
h0=0u;
#endif
#ifdef CE
Wa=j0;
#endif
}
#else
layout(location=0)out mediump vec4 Zg;void main(){Zg=vec4(0,1,0,1);}
#endif
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive