#ifdef DB
void main(){gl_Position=vec4(mix(vec2(-1,1),vec2(1,-1),equal(gl_VertexID&ivec2(1,2),ivec2(0))),0,1);
#ifdef RC
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
#ifdef RE
#if __VERSION__>=310
layout(binding=0,std140)uniform Ci{uniform highp vec4 Sg;}Tg;
#else
uniform mediump vec4 SE;
#endif
#endif
#ifdef GL_EXT_shader_pixel_local_storage
#ifdef BE
__pixel_local_inEXT R1
#else
__pixel_local_outEXT R1
#endif
{layout(rgba8)mediump vec4 j0;layout(r32ui)highp uint h0;layout(rgba8)mediump vec4 i4;layout(r32ui)highp uint E7;};
#ifndef GL_ARM_shader_framebuffer_fetch
#ifdef TE
layout(location=0)inout mediump vec4 Wa;
#endif
#endif
#ifdef BE
layout(location=0)out mediump vec4 Wa;
#endif
void main(){
#ifdef RE
#if __VERSION__>=310
j0=Tg.Sg;
#else
j0=SE;
#endif
#endif
#ifdef TE
#ifdef GL_ARM_shader_framebuffer_fetch
j0=gl_LastFragColorARM;
#else
j0=Wa;
#endif
#endif
#ifdef CE
E7=0u;
#endif
#ifdef RF
h0=0u;
#endif
#ifdef BE
Wa=j0;
#endif
}
#else
layout(location=0)out mediump vec4 Ug;void main(){Ug=vec4(0,1,0,1);}
#endif
#endif
