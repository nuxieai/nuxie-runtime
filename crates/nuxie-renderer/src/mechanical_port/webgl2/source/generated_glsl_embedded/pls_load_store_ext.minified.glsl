#ifdef BB
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
layout(binding=0,std140) uniform Ij{uniform highp vec4 Uh;}Vh;
#else
uniform mediump vec4 ZE;
#endif
#endif
#ifdef GL_EXT_shader_pixel_local_storage
#ifdef FE
__pixel_local_inEXT Z1
#else
__pixel_local_outEXT Z1
#endif
{layout(rgba8) mediump vec4 o0;layout(r32ui) highp uint m0;layout(rgba8) mediump vec4 B4;layout(r32ui) highp uint R7;};
#ifndef GL_ARM_shader_framebuffer_fetch
#ifdef AF
layout(location=0) inout mediump vec4 xb;
#endif
#endif
#ifdef FE
layout(location=0) out mediump vec4 xb;
#endif
void main(){
#ifdef YE
#if __VERSION__>=310
o0=Vh.Uh;
#else
o0=ZE;
#endif
#endif
#ifdef AF
#ifdef GL_ARM_shader_framebuffer_fetch
o0=gl_LastFragColorARM;
#else
o0=xb;
#endif
#endif
#ifdef GE
R7=0u;
#endif
#ifdef YF
m0=0u;
#endif
#ifdef FE
xb=o0;
#endif
}
#else
layout(location=0) out mediump vec4 Wh;void main(){Wh=vec4(0,1,0,1);}
#endif
#endif
