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
layout(binding=0,std140)uniform ij{uniform highp vec4 th;}uh;
#else
uniform mediump vec4 XE;
#endif
#endif
#ifdef GL_EXT_shader_pixel_local_storage
#ifdef STORE_COLOR
__pixel_local_inEXT S1
#else
__pixel_local_outEXT S1
#endif
{layout(rgba8)mediump vec4 k0;layout(r32ui)highp uint h0;layout(rgba8)mediump vec4 m4;layout(r32ui)highp uint H7;};
#ifndef GL_ARM_shader_framebuffer_fetch
#ifdef LOAD_COLOR
layout(location=0)inout mediump vec4 db;
#endif
#endif
#ifdef STORE_COLOR
layout(location=0)out mediump vec4 db;
#endif
void main(){
#ifdef CLEAR_COLOR
#if __VERSION__>=310
k0=uh.th;
#else
k0=XE;
#endif
#endif
#ifdef LOAD_COLOR
#ifdef GL_ARM_shader_framebuffer_fetch
k0=gl_LastFragColorARM;
#else
k0=db;
#endif
#endif
#ifdef CLEAR_COVERAGE
H7=0u;
#endif
#ifdef CLEAR_CLIP
h0=0u;
#endif
#ifdef STORE_COLOR
db=k0;
#endif
}
#else
layout(location=0)out mediump vec4 vh;void main(){vh=vec4(0,1,0,1);}
#endif
#endif
