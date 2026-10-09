#pragma once

#include "glsl.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char glsl[] = R"===(#define bb
#ifndef KC
#define KC __VERSION__
#endif
#define c vec2
#define M vec3
#define i4 vec3
#define f vec4
#define d mediump float
#define D mediump vec2
#define v mediump vec3
#define i mediump vec4
#define z7 mediump mat3x3
#define A7 mediump mat2x3
#define d5 mediump mat4x4
#define g0 ivec2
#define x6 ivec4
#define R0 uvec2
#define O uvec4
#define P mediump uint
#define c5 bvec2
#define Q6 bvec3
#define d8 bvec4
#define X mat2
#define e
#define k1(D2) out D2
#define i7(D2) inout D2
#ifdef GL_ANGLE_base_vertex_base_instance_shader_builtin
#extension GL_ANGLE_base_vertex_base_instance_shader_builtin:require
#endif
#ifdef NE
#extension GL_KHR_blend_equation_advanced:require
#endif
#ifdef AE
#extension GL_EXT_shader_framebuffer_fetch:require
#elif defined(BE)
#extension GL_EXT_shader_pixel_local_storage:require
#elif defined(EXPORTED_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
#extension GL_ANGLE_shader_pixel_local_storage:require
#elif defined(CE)
#ifdef GL_ARB_shader_image_load_store
#extension GL_ARB_shader_image_load_store:require
#endif
#ifdef GL_OES_shader_image_atomic
#extension GL_OES_shader_image_atomic:require
#endif
#endif
#if defined(CB)&&defined(AB)&&defined(GL_ES)&&!defined(SE)
#ifdef GL_EXT_clip_cull_distance
#extension GL_EXT_clip_cull_distance:require
#elif defined(GL_ANGLE_clip_cull_distance)
#extension GL_ANGLE_clip_cull_distance:require
#endif
#endif
#if KC>=310
#define c8(g,a) layout(binding=g,std140) uniform a{
#else
#define c8(g,a) layout(std140) uniform a{
#endif
#define L9(a) }a;
#define d1(a)
#define K(g,k0,a) layout(location=g) in k0 a
#define e1
#define L(O9,B,a,k0)
#ifdef BB
#if KC>=310
#define W(g,k0,a) layout(location=g) out k0 a
#else
#define W(g,k0,a) out k0 a
#endif
#else
#if KC>=310
#define W(g,k0,a) layout(location=g) in k0 a
#else
#define W(g,k0,a) in k0 a
#endif
#endif
#define g3 flat
#define v2
#define k2
#ifdef DC
#define F0
#else
#ifdef GL_NV_shader_noperspective_interpolation
#extension GL_NV_shader_noperspective_interpolation:require
#define F0 noperspective
#else
#define F0
#endif
#endif
#ifdef BB
#define q4
#define r4
#endif
#ifdef EB
#define V3
#define W3
#endif
#define B5
#define C5
#ifdef DC
#define i5(e0,g,a) layout(set=e0,binding=g) uniform highp utexture2D a
#define R6(e0,g,a) layout(set=e0,binding=g) uniform highp texture2D a
#define p3(e0,g,a) layout(set=e0,binding=g) uniform mediump texture2D a
#define M5(e0,g,a) layout(binding=g) uniform mediump texture2D a
#if defined(EB)&&defined(CB)
#endif
#elif KC>=310
#define i5(e0,g,a) layout(binding=g) uniform highp usampler2D a
#define R6(e0,g,a) layout(binding=g) uniform highp sampler2D a
#define p3(e0,g,a) layout(binding=g) uniform mediump sampler2D a
#define M5(e0,g,a) layout(binding=g) uniform mediump sampler2D a
#else
#define i5(e0,g,a) uniform highp usampler2D a
#define R6(e0,g,a) uniform highp sampler2D a
#define p3(e0,g,a) uniform mediump sampler2D a
#define M5(e0,g,a) uniform mediump sampler2D a
#endif
#ifdef DC
#define S6(e0,g,a) layout(set=e0,binding=g) uniform mediump sampler a;
#ifdef RF
#define a4(Q5,a) layout(set=Ch,binding=Q5) uniform mediump sampler a;
#define J6 a4
#define w4(a) S6(A5,Bh,a)
#else
#define a4(Q5,a) layout(set=q3,binding=Q5) uniform mediump sampler a;
#define J6 a4
#define w4(a) S6(A5,v4,a)
#endif
#define R5(a,p,o) texture(sampler2D(a,p),o)
#define n2(a,p,o,a1) textureLod(sampler2D(a,p),o,a1)
#define S5(a,p,o,f2) texture(sampler2D(a,p),o,f2)
#if defined(EB)&&defined(CB)&&defined(TE)
#extension GL_OES_sample_variables:require
#endif
#else
#define a4(Q5,a)
#define J6(Q5,a)
#define S6(e0,g,a)
#define w4(a)
#define R5(a,p,o) texture(a,o)
#define n2(a,p,o,a1) textureLod(a,o,a1)
#define S5(a,p,o,f2) texture(a,o,f2)
#endif
#define T8(q0,p,o) R5(q0,p,o)
#define D5(q0,p,o,a1) n2(q0,p,o,a1)
#define e8(q0,p,o,f2) S5(q0,p,o,f2)
#define I6(e0,g,a) M5(e0,g,a)
#define y7(a,p,E,T6,Q9,a1) n2(a,p,c(E,Q9),a1)
#define Di(e0,g,a) i5(e0,g,a)
#define c4
#define m1
#define q1(a,o) texelFetch(a,o,0)
#ifdef DC
#elif KC>=310
#else
#endif
#define Y4
#define Z4
#define k4
#define l4
#ifdef SF
#define j6(g,G1,a) i5(q3,g,a)
#define j5(g,G1,a) Di(q3,g,a)
#define k6(g,G1,a) R6(q3,g,a)
#define p0(a,E0) q1(a,g0((E0)&Ud,(E0)>>Td))
#define w5(a,E0) q1(a,g0((E0)&Ud,(E0)>>Td)).xy
#else
#ifdef GL_ARB_shader_storage_buffer_object
#extension GL_ARB_shader_storage_buffer_object:require
#endif
#define j6(g,G1,a) layout(std430,binding=g) readonly buffer G1{R0 r2[];}a
#define j5(g,G1,a) layout(std430,binding=g) readonly buffer G1{O r2[];}a
#define k6(g,G1,a) layout(std430,binding=g) readonly buffer G1{f r2[];}a
#define Yb(g,G1,a) layout(std430,binding=g) buffer G1{uint r2[];}a
#define p0(a,E0) a.r2[E0]
#define w5(a,E0) a.r2[E0]
#define Oe(a,E0) a.r2[E0]
#define g8(a,E0,E) atomicMax(a.r2[E0],E)
#define Zb(a,E0,E) atomicAdd(a.r2[E0],E)
#define Ei(a,E0,E) atomicOr(a.r2[E0],E)
#endif
#ifdef LD
#define X1(a) void main(){g0 G=ivec2(floor(d0));int L0=int(K9(uvec2(G),(j.P6+(Ab-1u))&~(Ab-1u)));
#define o2 }
#define m4 ,int L0
#define Y1 ,L0
#ifdef UE
#define U2(g,a) layout(std430,set=K3,binding=g) buffer a##Pe{uint r2[];}a
#elif defined(DC)
#define U2(g,a) layout(std430,set=K3,binding=g) coherent buffer a##Pe{uint r2[];}a
#else
#define U2(g,a) layout(std430,binding=g) coherent buffer a##Pe{uint r2[];}a
#endif
#define ac U2
#define m3(h) h.r2[L0]
#define n3(h,C) h.r2[L0]=C
#define bc(h) unpackUnorm4x8(m3(h))
#define cc(h,C) n3(h,packUnorm4x8(C))
#define y5(h,E) atomicMax(h.r2[L0],E)
#define z5(h,E) atomicAdd(h.r2[L0],E)
#elif defined(DE)||defined(TF)
#ifdef GL_ARB_shader_image_load_store
#extension GL_ARB_shader_image_load_store:require
#endif
#define X1(a) void main(){g0 G=ivec2(floor(d0));
#define o2 }
#define m4 ,g0 G
#define Y1 ,G
#ifdef DC
#define ac(g,a) layout(set=K3,binding=g,rgba8) uniform mediump coherent image2D a
#define U2(g,a) layout(set=K3,binding=g,r32ui) uniform highp coherent uimage2D a
#define dc(g,a) layout(set=K3,binding=g,rgb10_a2) uniform mediump coherent image2D a
#else
#define ac(g,a) layout(binding=g,rgba8) uniform mediump coherent image2D a
#define U2(g,a) layout(binding=g,r32ui) uniform highp coherent uimage2D a
#define dc(g,a) layout(binding=g,rgb10_a2) uniform mediump coherent image2D a;
#endif
#define m3(h) imageLoad(h,G).x
#define n3(h,C) imageStore(h,G,uvec4(C))
#define bc(h) imageLoad(h,G)
#define cc(h,C) imageStore(h,G,C)
#define y5(h,E) imageAtomicMax(h,G,E)
#define z5(h,E) imageAtomicAdd(h,G,E)
#else
#define X1(a) void main()
#define o2
#define m4
#define Y1
#endif
#ifdef EXPORTED_PLS_IMPL_ANGLE
#extension GL_ANGLE_shader_pixel_local_storage:require
#define U1
#define C0(g,a) layout(binding=g,rgba8) uniform mediump pixelLocalANGLE a
#define p1(g,a) layout(binding=g,r32ui) uniform highp upixelLocalANGLE a
#define V1
#define Q0(h) pixelLocalLoadANGLE(h)
#define j1(h) pixelLocalLoadANGLE(h).x
#define y0(h,C) pixelLocalStoreANGLE(h,C)
#define l1(h,C) pixelLocalStoreANGLE(h,uvec4(C))
#define M2(h)
#define g2(h)
#define N2
#define O2
#endif
#ifdef UF
#ifdef U
#extension GL_EXT_shader_pixel_local_storage2:require
#else
#extension GL_EXT_shader_pixel_local_storage:require
#endif
#define U1 __pixel_localEXT h2{
#define C0(g,a) layout(rgba8) mediump vec4 a
#define ec(g,a) layout(rgb10_a2) mediump vec4 a
#define p1(g,a) layout(r32ui) highp uint a
#define V1 };
#define Q0(h) h
#define j1(h) h
#define y0(h,C) h=(C)
#define l1(h,C) h=(C)
#define M2(h) h=h
#define g2(h) h=h
#define N2
#define O2
#ifdef U
#define G2(a) layout(location=0,rgba8) out i L1;X1(a)
#endif
#endif
#if defined(DE)||defined(LD)
#define U1
#define V1
#define C0 ac
#define p1 U2
#define ec dc
#define Q0 bc
#define y0 cc
#define j1 m3
#define l1 n3
#define M2(h)
#define g2(h)
#if defined(GL_ARB_fragment_shader_interlock)
#extension GL_ARB_fragment_shader_interlock:require
#define N2 beginInvocationInterlockARB()
#define O2 endInvocationInterlockARB()
#elif defined(GL_INTEL_fragment_shader_ordering)
#extension GL_INTEL_fragment_shader_ordering:require
#define N2 beginFragmentShaderOrderingINTEL()
#define O2
#else
#define N2
#define O2
#endif
#endif
#ifdef VF
#define U1
#define P4(g,a) layout(input_attachment_index=g,binding=g,set=K3) uniform mediump subpassInput h8##a
#define Qe(g,a) layout(location=g) out mediump vec4 a
#define C0(g,a) P4(g,a);Qe(g,a)
#define p1(g,a) layout(input_attachment_index=g,binding=g,set=K3) uniform highp usubpassInput h8##a;layout(location=g) out highp uvec4 a
#define V1
#define Q0(h) subpassLoad(h8##h)
#define j1(h) subpassLoad(h8##h).x
#define y0(h,C) h=(C)
#define l1(h,C) h.x=(C)
#define M2(h) y0(h,subpassLoad(h8##h))
#define g2(h) l1(h,subpassLoad(h8##h).x)
#define N2
#define O2
#endif
#ifdef WF
#define U1
#define C0(g,a) layout(location=g) out mediump vec4 a
#define p1(g,a) layout(location=g) out highp uvec4 a
#define V1
#define Q0(h) vec4(0)
#define j1(h) 0u
#define y0(h,C) h=(C)
#define l1(h,C) h.x=(C)
#define M2(h) h=vec4(0)
#define g2(h) h.x=0u
#define N2
#define O2
#endif
#ifndef P4
#define P4 C0
#endif
#ifdef DC
#define fc gl_VertexIndex
#ifdef EE
#define i8 gl_InstanceIndex
#else
#define i8 0
#endif
#else
#ifdef VE
uniform highp int WE;
#define fc (gl_VertexID+WE)
#else
#define fc gl_VertexID
#endif
#ifdef EE
#ifdef FE
uniform highp int FE;
#define i8 (gl_InstanceID+FE)
#else
#define i8 (gl_InstanceID+gl_BaseInstance)
#endif
#else
#define i8 0
#endif
#endif
#define M6
#define Q3
#define E7
#define h5
#define w1(a,f0,B,L2,C6) void main(){int L2=fc;int C6=i8;
#define C8(a,f0,B,C1,j0,L2,C6) w1(a,f0,B,L2,C6)
#define g7(a,C3,D3,T3,i3,C1,j0,L2) w1(a,C3,D3,L2,C6)
#define V(a,k0)
#define Z(a)
#define q(a,k0)
#define x1(X0) gl_Position=X0;}
#define V2(H1,a) layout(location=0) out H1 Fi;void main()
#define U6(H1,a) V2(H1,a)
#define V6 gl_FrontFacing
#define K2(C) Fi=C
#define d0 gl_FragCoord.xy
#define j7
#define l3
#if defined(DE)||defined(LD)
#define Re(j8,h,C) if(!(j8)){y0(h,C);}
#define Se(j8,h,C) if(!(j8)){l1(h,C);}
#else
#define Re(j8,h,C) y0(h,C);
#define Se(j8,h,C) l1(h,C);
#endif
#ifndef G2
#define G2(a) layout(location=0) out i L1;X1(a)
#endif
#define E3 o2
#if defined(DC)&&!defined(UE)
#ifdef TE
#define N5(a) layout(input_attachment_index=0,binding=T2,set=K3) uniform mediump subpassInputMS a
#define L5(a) lb(mat4(subpassLoad(a,0),subpassLoad(a,1),subpassLoad(a,2),subpassLoad(a,3)),gl_SampleMaskIn[0])
#else
#define N5(a) layout(input_attachment_index=0,binding=T2,set=K3) uniform mediump subpassInput a
#define L5(a) subpassLoad(a)
#endif
#else
#define N5(a) p3(q3,Ah,a)
#define L5(a) texelFetch(a,ivec2(floor(d0.xy)),0)
#endif
#define B0(A,J) ((A)*(J))
precision highp float;precision highp int;
#if KC<310
e i Gi(uint u){O r1=O(u&0xffu,(u>>8)&0xffu,(u>>16)&0xffu,u>>24);return f(r1)*(1./255.);}
#define unpackUnorm4x8 Gi
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive