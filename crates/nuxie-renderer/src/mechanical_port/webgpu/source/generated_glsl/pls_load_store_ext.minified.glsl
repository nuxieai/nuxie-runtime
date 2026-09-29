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
layout(binding=0,std140)uniform Si{uniform highp vec4 bh;}ch;
#else
uniform mediump vec4 SE;
#endif
#endif
#ifdef GL_EXT_shader_pixel_local_storage
#ifdef STORE_COLOR
__pixel_local_inEXT S1
#else
__pixel_local_outEXT S1
#endif
{layout(rgba8)mediump vec4 j0;layout(r32ui)highp uint g0;layout(rgba8)mediump vec4 l4;layout(r32ui)highp uint H7;};
#ifndef GL_ARM_shader_framebuffer_fetch
#ifdef LOAD_COLOR
layout(location=0)inout mediump vec4 Za;
#endif
#endif
#ifdef STORE_COLOR
layout(location=0)out mediump vec4 Za;
#endif
void main(){
#ifdef CLEAR_COLOR
#if __VERSION__>=310
j0=ch.bh;
#else
j0=SE;
#endif
#endif
#ifdef LOAD_COLOR
#ifdef GL_ARM_shader_framebuffer_fetch
j0=gl_LastFragColorARM;
#else
j0=Za;
#endif
#endif
#ifdef CLEAR_COVERAGE
H7=0u;
#endif
#ifdef CLEAR_CLIP
g0=0u;
#endif
#ifdef STORE_COLOR
Za=j0;
#endif
}
#else
layout(location=0)out mediump vec4 dh;void main(){dh=vec4(0,1,0,1);}
#endif
#endif
