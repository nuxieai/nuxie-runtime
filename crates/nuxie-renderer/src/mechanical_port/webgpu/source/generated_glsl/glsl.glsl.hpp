#pragma once

#include "glsl.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char glsl[] = R"===(#define eb
#ifndef LC
#define LC __VERSION__
#endif
#define c vec2
#define M vec3
#define i4 vec3
#define f vec4
#define d mediump float
#define D mediump vec2
#define v mediump vec3
#define i mediump vec4
#define C7 mediump mat3x3
#define D7 mediump mat2x3
#define d5 mediump mat4x4
#define g0 ivec2
#define x6 ivec4
#define R0 uvec2
#define O uvec4
#define P mediump uint
#define c5 bvec2
#define T6 bvec3
#define g8 bvec4
#define X mat2
#define e
#define l1(E2) out E2
#define m7(E2) inout E2
#ifdef GL_ANGLE_base_vertex_base_instance_shader_builtin
#extension GL_ANGLE_base_vertex_base_instance_shader_builtin:require
#endif
#ifdef PE
#extension GL_KHR_blend_equation_advanced:require
#endif
#ifdef CE
#extension GL_EXT_shader_framebuffer_fetch:require
#elif defined(DE)
#extension GL_EXT_shader_pixel_local_storage:require
#elif defined(EXPORTED_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
#extension GL_ANGLE_shader_pixel_local_storage:require
#elif defined(EE)
#ifdef GL_ARB_shader_image_load_store
#extension GL_ARB_shader_image_load_store:require
#endif
#ifdef GL_OES_shader_image_atomic
#extension GL_OES_shader_image_atomic:require
#endif
#endif
#if defined(CB)&&defined(AB)&&defined(GL_ES)&&!defined(UE)
#ifdef GL_EXT_clip_cull_distance
#extension GL_EXT_clip_cull_distance:require
#elif defined(GL_ANGLE_clip_cull_distance)
#extension GL_ANGLE_clip_cull_distance:require
#endif
#endif
#if LC>=310
#define f8(g,a) layout(binding=g,std140) uniform a{
#else
#define f8(g,a) layout(std140) uniform a{
#endif
#define O9(a) }a;
#define e1(a)
#define K(g,k0,a) layout(location=g) in k0 a
#define f1
#define L(R9,B,a,k0)
#ifdef BB
#if LC>=310
#define W(g,k0,a) layout(location=g) out k0 a
#else
#define W(g,k0,a) out k0 a
#endif
#else
#if LC>=310
#define W(g,k0,a) layout(location=g) in k0 a
#else
#define W(g,k0,a) in k0 a
#endif
#endif
#define h3 flat
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
#define C5
#define D5
#ifdef DC
#define i5(e0,g,a) layout(set=e0,binding=g) uniform highp utexture2D a
#define U6(e0,g,a) layout(set=e0,binding=g) uniform highp texture2D a
#define q3(e0,g,a) layout(set=e0,binding=g) uniform mediump texture2D a
#define N5(e0,g,a) layout(binding=g) uniform mediump texture2D a
#if defined(EB)&&defined(CB)
#endif
#elif LC>=310
#define i5(e0,g,a) layout(binding=g) uniform highp usampler2D a
#define U6(e0,g,a) layout(binding=g) uniform highp sampler2D a
#define q3(e0,g,a) layout(binding=g) uniform mediump sampler2D a
#define N5(e0,g,a) layout(binding=g) uniform mediump sampler2D a
#else
#define i5(e0,g,a) uniform highp usampler2D a
#define U6(e0,g,a) uniform highp sampler2D a
#define q3(e0,g,a) uniform mediump sampler2D a
#define N5(e0,g,a) uniform mediump sampler2D a
#endif
#ifdef DC
#define V6(e0,g,a) layout(set=e0,binding=g) uniform mediump sampler a;
#ifdef TF
#define a4(R5,a) layout(set=Gh,binding=R5) uniform mediump sampler a;
#define M6 a4
#define w4(a) V6(B5,Fh,a)
#else
#define a4(R5,a) layout(set=v3,binding=R5) uniform mediump sampler a;
#define M6 a4
#define w4(a) V6(B5,v4,a)
#endif
#define S5(a,p,o) texture(sampler2D(a,p),o)
#define n2(a,p,o,c1) textureLod(sampler2D(a,p),o,c1)
#define T5(a,p,o,f2) texture(sampler2D(a,p),o,f2)
#if defined(EB)&&defined(CB)&&defined(VE)
#extension GL_OES_sample_variables:require
#endif
#else
#define a4(R5,a)
#define M6(R5,a)
#define V6(e0,g,a)
#define w4(a)
#define S5(a,p,o) texture(a,o)
#define n2(a,p,o,c1) textureLod(a,o,c1)
#define T5(a,p,o,f2) texture(a,o,f2)
#endif
#define W8(q0,p,o) S5(q0,p,o)
#define E5(q0,p,o,c1) n2(q0,p,o,c1)
#define h8(q0,p,o,f2) T5(q0,p,o,f2)
#define K6(e0,g,a) N5(e0,g,a)
#define B7(a,p,E,W6,T9,c1) n2(a,p,c(E,T9),c1)
#define Hi(e0,g,a) i5(e0,g,a)
#define c4
#define n1
#define r1(a,o) texelFetch(a,o,0)
#ifdef DC
#elif LC>=310
#else
#endif
#define Y4
#define Z4
#define k4
#define l4
#ifdef UF
#define j6(g,J1,a) i5(v3,g,a)
#define j5(g,J1,a) Hi(v3,g,a)
#define k6(g,J1,a) U6(v3,g,a)
#define p0(a,E0) r1(a,g0((E0)&Wd,(E0)>>Vd))
#define x5(a,E0) r1(a,g0((E0)&Wd,(E0)>>Vd)).xy
#else
#ifdef GL_ARB_shader_storage_buffer_object
#extension GL_ARB_shader_storage_buffer_object:require
#endif
#define j6(g,J1,a) layout(std430,binding=g) readonly buffer J1{R0 r2[];}a
#define j5(g,J1,a) layout(std430,binding=g) readonly buffer J1{O r2[];}a
#define k6(g,J1,a) layout(std430,binding=g) readonly buffer J1{f r2[];}a
#define cc(g,J1,a) layout(std430,binding=g) buffer J1{uint r2[];}a
#define p0(a,E0) a.r2[E0]
#define x5(a,E0) a.r2[E0]
#define Qe(a,E0) a.r2[E0]
#define j8(a,E0,E) atomicMax(a.r2[E0],E)
#define dc(a,E0,E) atomicAdd(a.r2[E0],E)
#define Ii(a,E0,E) atomicOr(a.r2[E0],E)
#endif
#ifdef ND
#define X1(a) void main(){g0 G=ivec2(floor(d0));int L0=int(N9(uvec2(G),(j.S6+(Eb-1u))&~(Eb-1u)));
#define o2 }
#define m4 ,int L0
#define Y1 ,L0
#ifdef WE
#define U2(g,a) layout(std430,set=K3,binding=g) buffer a##Re{uint r2[];}a
#elif defined(DC)
#define U2(g,a) layout(std430,set=K3,binding=g) coherent buffer a##Re{uint r2[];}a
#else
#define U2(g,a) layout(std430,binding=g) coherent buffer a##Re{uint r2[];}a
#endif
#define L6 U2
#define n3(h) h.r2[L0]
#define o3(h,C) h.r2[L0]=C
#define I6(h) unpackUnorm4x8(n3(h))
#define J6(h,C) o3(h,packUnorm4x8(C))
#define z5(h,E) atomicMax(h.r2[L0],E)
#define A5(h,E) atomicAdd(h.r2[L0],E)
#elif defined(FE)||defined(VF)
#ifdef GL_ARB_shader_image_load_store
#extension GL_ARB_shader_image_load_store:require
#endif
#define X1(a) void main(){g0 G=ivec2(floor(d0));
#define o2 }
#define m4 ,g0 G
#define Y1 ,G
#ifdef DC
#define L6(g,a) layout(set=K3,binding=g,rgba8) uniform mediump coherent image2D a
#define U2(g,a) layout(set=K3,binding=g,r32ui) uniform highp coherent uimage2D a
#define ec(g,a) layout(set=K3,binding=g,rgb10_a2) uniform mediump coherent image2D a
#else
#define L6(g,a) layout(binding=g,rgba8) uniform mediump coherent image2D a
#define U2(g,a) layout(binding=g,r32ui) uniform highp coherent uimage2D a
#define ec(g,a) layout(binding=g,rgb10_a2) uniform mediump coherent image2D a;
#endif
#define n3(h) imageLoad(h,G).x
#define o3(h,C) imageStore(h,G,uvec4(C))
#define I6(h) imageLoad(h,G)
#define J6(h,C) imageStore(h,G,C)
#define z5(h,E) imageAtomicMax(h,G,E)
#define A5(h,E) imageAtomicAdd(h,G,E)
#else
#define X1(a) void main()
#define o2
#define m4
#define Y1
#endif
#ifdef EXPORTED_PLS_IMPL_ANGLE
#extension GL_ANGLE_shader_pixel_local_storage:require
#define E1
#define z0(g,a) layout(binding=g,rgba8) uniform mediump pixelLocalANGLE a
#define q1(g,a) layout(binding=g,r32ui) uniform highp upixelLocalANGLE a
#define F1
#define O0(h) pixelLocalLoadANGLE(h)
#define k1(h) pixelLocalLoadANGLE(h).x
#define x0(h,C) pixelLocalStoreANGLE(h,C)
#define m1(h,C) pixelLocalStoreANGLE(h,uvec4(C))
#define N2(h)
#define g2(h)
#define O2
#define P2
#endif
#ifdef WF
#ifdef U
#extension GL_EXT_shader_pixel_local_storage2:require
#else
#extension GL_EXT_shader_pixel_local_storage:require
#endif
#define E1 __pixel_localEXT h2{
#define z0(g,a) layout(rgba8) mediump vec4 a
#define fc(g,a) layout(rgb10_a2) mediump vec4 a
#define q1(g,a) layout(r32ui) highp uint a
#define F1 };
#define O0(h) h
#define k1(h) h
#define x0(h,C) h=(C)
#define m1(h,C) h=(C)
#define N2(h) h=h
#define g2(h) h=h
#define O2
#define P2
#ifdef U
#define H2(a) layout(location=0,rgba8) out i N1;X1(a)
#endif
#endif
#if defined(FE)||defined(ND)
#define E1
#define F1
#define z0 L6
#define q1 U2
#define fc ec
#define O0 I6
#define x0 J6
#define k1 n3
#define m1 o3
#define N2(h)
#define g2(h)
#if defined(GL_ARB_fragment_shader_interlock)
#extension GL_ARB_fragment_shader_interlock:require
#define O2 beginInvocationInterlockARB()
#define P2 endInvocationInterlockARB()
#elif defined(GL_INTEL_fragment_shader_ordering)
#extension GL_INTEL_fragment_shader_ordering:require
#define O2 beginFragmentShaderOrderingINTEL()
#define P2
#else
#define O2
#define P2
#endif
#endif
#ifdef XF
#define E1
#define P4(g,a) layout(input_attachment_index=g,binding=g,set=K3) uniform mediump subpassInput k8##a
#define Se(g,a) layout(location=g) out mediump vec4 a
#define z0(g,a) P4(g,a);Se(g,a)
#define q1(g,a) layout(input_attachment_index=g,binding=g,set=K3) uniform highp usubpassInput k8##a;layout(location=g) out highp uvec4 a
#define F1
#define O0(h) subpassLoad(k8##h)
#define k1(h) subpassLoad(k8##h).x
#define x0(h,C) h=(C)
#define m1(h,C) h.x=(C)
#define N2(h) x0(h,subpassLoad(k8##h))
#define g2(h) m1(h,subpassLoad(k8##h).x)
#define O2
#define P2
#endif
#ifdef YF
#define E1
#define z0(g,a) layout(location=g) out mediump vec4 a
#define q1(g,a) layout(location=g) out highp uvec4 a
#define F1
#define O0(h) vec4(0)
#define k1(h) 0u
#define x0(h,C) h=(C)
#define m1(h,C) h.x=(C)
#define N2(h) h=vec4(0)
#define g2(h) h.x=0u
#define O2
#define P2
#endif
#ifndef P4
#define P4 z0
#endif
#ifdef DC
#define gc gl_VertexIndex
#ifdef GE
#define l8 gl_InstanceIndex
#else
#define l8 0
#endif
#else
#ifdef XE
uniform highp int YE;
#define gc (gl_VertexID+YE)
#else
#define gc gl_VertexID
#endif
#ifdef GE
#ifdef HE
uniform highp int HE;
#define l8 (gl_InstanceID+HE)
#else
#define l8 (gl_InstanceID+gl_BaseInstance)
#endif
#else
#define l8 0
#endif
#endif
#define P6
#define Q3
#define H7
#define h5
#define y1(a,f0,B,M2,C6) void main(){int M2=gc;int C6=l8;
#define F8(a,f0,B,D1,j0,M2,C6) y1(a,f0,B,M2,C6)
#define k7(a,D3,E3,T3,j3,D1,j0,M2) y1(a,D3,E3,M2,C6)
#define V(a,k0)
#define a0(a)
#define q(a,k0)
#define z1(X0) gl_Position=X0;}
#define V2(K1,a) layout(location=0) out K1 Ji;void main()
#define X6(K1,a) V2(K1,a)
#define Y6 gl_FrontFacing
#define L2(C) Ji=C
#define d0 gl_FragCoord.xy
#define n7
#define m3
#if defined(FE)||defined(ND)
#define Te(m8,h,C) if(!(m8)){x0(h,C);}
#define Ue(m8,h,C) if(!(m8)){m1(h,C);}
#else
#define Te(m8,h,C) x0(h,C);
#define Ue(m8,h,C) m1(h,C);
#endif
#ifndef H2
#define H2(a) layout(location=0) out i N1;X1(a)
#endif
#define F3 o2
#if defined(DC)&&!defined(WE)
#ifdef VE
#define O5(a) layout(input_attachment_index=0,binding=x2,set=K3) uniform mediump subpassInputMS a
#define M5(a) pb(mat4(subpassLoad(a,0),subpassLoad(a,1),subpassLoad(a,2),subpassLoad(a,3)),gl_SampleMaskIn[0])
#else
#define O5(a) layout(input_attachment_index=0,binding=x2,set=K3) uniform mediump subpassInput a
#define M5(a) subpassLoad(a)
#endif
#else
#define O5(a) q3(v3,Eh,a)
#define M5(a) texelFetch(a,ivec2(floor(d0.xy)),0)
#endif
#define C0(A,J) ((A)*(J))
precision highp float;precision highp int;
#if LC<310
e i Ki(uint u){O v1=O(u&0xffu,(u>>8)&0xffu,(u>>16)&0xffu,u>>24);return f(v1)*(1./255.);}
#define unpackUnorm4x8 Ki
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive