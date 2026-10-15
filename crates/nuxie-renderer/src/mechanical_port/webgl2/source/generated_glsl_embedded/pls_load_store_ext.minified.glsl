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
#ifdef BF
#if __VERSION__>=310
layout(binding=0,std140) uniform Mk{uniform highp vec4 Si;}Ti;
#else
uniform mediump vec4 CF;
#endif
#endif
#ifdef GL_EXT_shader_pixel_local_storage
#ifdef IE
__pixel_local_inEXT h2
#else
__pixel_local_outEXT h2
#endif
{layout(rgba8) mediump vec4 n0;layout(r32ui) highp uint m0;layout(rgba8) mediump vec4 I4;layout(r32ui) highp uint p8;};
#ifndef GL_ARM_shader_framebuffer_fetch
#ifdef DF
layout(location=0) inout mediump vec4 oc;
#endif
#endif
#ifdef IE
layout(location=0) out mediump vec4 oc;
#endif
void main(){
#ifdef BF
#if __VERSION__>=310
n0=Ti.Si;
#else
n0=CF;
#endif
#endif
#ifdef DF
#ifdef GL_ARM_shader_framebuffer_fetch
n0=gl_LastFragColorARM;
#else
n0=oc;
#endif
#endif
#ifdef JE
p8=0u;
#endif
#ifdef BG
m0=0u;
#endif
#ifdef IE
oc=n0;
#endif
}
#else
layout(location=0) out mediump vec4 Ui;void main(){Ui=vec4(0,1,0,1);}
#endif
#endif
