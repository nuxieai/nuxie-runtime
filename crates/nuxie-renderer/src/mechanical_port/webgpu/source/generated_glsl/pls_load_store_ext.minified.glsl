#ifdef VERTEX
void main(){gl_Position=vec4(mix(vec2(-1,1),vec2(1,-1),equal(gl_VertexID&ivec2(1,2),ivec2(0))),0,1);
#ifdef POST_INVERT_Y
gl_Position.y=-gl_Position.y;
#endif
}
#endif
#ifdef FRAGMENT
#extension GL_EXT_shader_pixel_local_storage:require
#ifdef GL_ARM_shader_framebuffer_fetch
#extension GL_ARM_shader_framebuffer_fetch:require
#else
#extension GL_EXT_shader_framebuffer_fetch:require
#endif
#ifdef CLEAR_COLOR
#if __VERSION__>=310
layout(binding=0,std140) uniform Jk{uniform highp vec4 Pi;}Qi;
#else
uniform mediump vec4 AF;
#endif
#endif
#ifdef GL_EXT_shader_pixel_local_storage
#ifdef STORE_COLOR
__pixel_local_inEXT h2
#else
__pixel_local_outEXT h2
#endif
{layout(rgba8) mediump vec4 n0;layout(r32ui) highp uint m0;layout(rgba8) mediump vec4 I4;layout(r32ui) highp uint m8;};
#ifndef GL_ARM_shader_framebuffer_fetch
#ifdef LOAD_COLOR
layout(location=0) inout mediump vec4 nc;
#endif
#endif
#ifdef STORE_COLOR
layout(location=0) out mediump vec4 nc;
#endif
void main(){
#ifdef CLEAR_COLOR
#if __VERSION__>=310
n0=Qi.Pi;
#else
n0=AF;
#endif
#endif
#ifdef LOAD_COLOR
#ifdef GL_ARM_shader_framebuffer_fetch
n0=gl_LastFragColorARM;
#else
n0=nc;
#endif
#endif
#ifdef CLEAR_COVERAGE
m8=0u;
#endif
#ifdef CLEAR_CLIP
m0=0u;
#endif
#ifdef STORE_COLOR
nc=n0;
#endif
}
#else
layout(location=0) out mediump vec4 Ri;void main(){Ri=vec4(0,1,0,1);}
#endif
#endif
