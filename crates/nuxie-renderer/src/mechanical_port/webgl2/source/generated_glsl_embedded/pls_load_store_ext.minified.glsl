#ifdef DB
void main(){gl_Position=vec4(mix(vec2(-1,1),vec2(1,-1),equal(gl_VertexID&ivec2(1,2),ivec2(0))),0,1);
#ifdef SC
gl_Position.y=-gl_Position.y;
#endif
}
#endif
#ifdef FB
#extension GL_EXT_shader_pixel_local_storage:require
#ifdef GL_ARM_shader_framebuffer_fetch
#extension GL_ARM_shader_framebuffer_fetch:require
#else
#extension GL_EXT_shader_framebuffer_fetch:require
#endif
#ifdef XE
#if __VERSION__>=310
layout(binding=0,std140)uniform kj{uniform highp vec4 vh;}wh;
#else
uniform mediump vec4 YE;
#endif
#endif
#ifdef GL_EXT_shader_pixel_local_storage
#ifdef FE
__pixel_local_inEXT S1
#else
__pixel_local_outEXT S1
#endif
{layout(rgba8)mediump vec4 k0;layout(r32ui)highp uint h0;layout(rgba8)mediump vec4 m4;layout(r32ui)highp uint H7;};
#ifndef GL_ARM_shader_framebuffer_fetch
#ifdef ZE
layout(location=0)inout mediump vec4 eb;
#endif
#endif
#ifdef FE
layout(location=0)out mediump vec4 eb;
#endif
void main(){
#ifdef XE
#if __VERSION__>=310
k0=wh.vh;
#else
k0=YE;
#endif
#endif
#ifdef ZE
#ifdef GL_ARM_shader_framebuffer_fetch
k0=gl_LastFragColorARM;
#else
k0=eb;
#endif
#endif
#ifdef GE
H7=0u;
#endif
#ifdef YF
h0=0u;
#endif
#ifdef FE
eb=k0;
#endif
}
#else
layout(location=0)out mediump vec4 xh;void main(){xh=vec4(0,1,0,1);}
#endif
#endif
