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
#define k4 vec3
#define e vec4
#define d mediump float
#define E mediump vec2
#define v mediump vec3
#define i mediump vec4
#define E7 mediump mat3x3
#define F7 mediump mat2x3
#define f5 mediump mat4x4
#define g0 ivec2
#define B6 ivec4
#define R0 uvec2
#define O uvec4
#define P mediump uint
#define e5 bvec2
#define U6 bvec3
#define h8 bvec4
#define X mat2
#define f
#define l1(E2) out E2
#define n7(E2) inout E2
#ifdef GL_ANGLE_base_vertex_base_instance_shader_builtin
#extension GL_ANGLE_base_vertex_base_instance_shader_builtin:require
#endif
#ifdef SE
#extension GL_KHR_blend_equation_advanced:require
#endif
#ifdef DE
#extension GL_EXT_shader_framebuffer_fetch:require
#elif defined(EE)
#extension GL_EXT_shader_pixel_local_storage:require
#elif defined(EXPORTED_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
#extension GL_ANGLE_shader_pixel_local_storage:require
#elif defined(FE)
#ifdef GL_ARB_shader_image_load_store
#extension GL_ARB_shader_image_load_store:require
#endif
#ifdef GL_OES_shader_image_atomic
#extension GL_OES_shader_image_atomic:require
#endif
#endif
#if defined(CB)&&defined(AB)&&defined(GL_ES)&&!defined(XE)
#ifdef GL_EXT_clip_cull_distance
#extension GL_EXT_clip_cull_distance:require
#elif defined(GL_ANGLE_clip_cull_distance)
#extension GL_ANGLE_clip_cull_distance:require
#endif
#endif
#if LC>=310
#define g8(g,a) layout(binding=g,std140) uniform a{
#else
#define g8(g,a) layout(std140) uniform a{
#endif
#define O9(a) }a;
#define e1(a)
#define K(g,k0,a) layout(location=g) in k0 a
#define f1
#define L(R9,A,a,k0)
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
#define m2
#define M1
#ifdef EC
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
#define v4
#define w4
#endif
#ifdef EB
#define X3
#define Y3
#endif
#define D5
#define E5
#ifdef EC
#define j5(h0,g,a) layout(set=h0,binding=g) uniform highp utexture2D a
#define V6(h0,g,a) layout(set=h0,binding=g) uniform highp texture2D a
#define q3(h0,g,a) layout(set=h0,binding=g) uniform mediump texture2D a
#define Q5(h0,g,a) layout(binding=g) uniform mediump texture2D a
#if defined(EB)&&defined(CB)
#endif
#elif LC>=310
#define j5(h0,g,a) layout(binding=g) uniform highp usampler2D a
#define V6(h0,g,a) layout(binding=g) uniform highp sampler2D a
#define q3(h0,g,a) layout(binding=g) uniform mediump sampler2D a
#define Q5(h0,g,a) layout(binding=g) uniform mediump sampler2D a
#else
#define j5(h0,g,a) uniform highp usampler2D a
#define V6(h0,g,a) uniform highp sampler2D a
#define q3(h0,g,a) uniform mediump sampler2D a
#define Q5(h0,g,a) uniform mediump sampler2D a
#endif
#ifdef EC
#define W6(h0,g,a) layout(set=h0,binding=g) uniform mediump sampler a;
#ifdef UF
#define d4(V5,a) layout(set=Jh,binding=V5) uniform mediump sampler a;
#define N6 d4
#define y4(a) W6(C5,Ih,a)
#else
#define d4(V5,a) layout(set=v3,binding=V5) uniform mediump sampler a;
#define N6 d4
#define y4(a) W6(C5,x4,a)
#endif
#define W5(a,p,o) texture(sampler2D(a,p),o)
#define p2(a,p,o,c1) textureLod(sampler2D(a,p),o,c1)
#define X5(a,p,o,h2) texture(sampler2D(a,p),o,h2)
#if defined(EB)&&defined(CB)&&defined(YE)
#extension GL_OES_sample_variables:require
#endif
#else
#define d4(V5,a)
#define N6(V5,a)
#define W6(h0,g,a)
#define y4(a)
#define W5(a,p,o) texture(a,o)
#define p2(a,p,o,c1) textureLod(a,o,c1)
#define X5(a,p,o,h2) texture(a,o,h2)
#endif
#define X8(r0,p,o) W5(r0,p,o)
#define F5(r0,p,o,c1) p2(r0,p,o,c1)
#define i8(r0,p,o,h2) X5(r0,p,o,h2)
#define M6(h0,g,a) Q5(h0,g,a)
#define D7(a,p,F,X6,T9,c1) p2(a,p,c(F,T9),c1)
#define Li(h0,g,a) j5(h0,g,a)
#define e4
#define n1
#define v1(a,o) texelFetch(a,o,0)
#ifdef EC
#elif LC>=310
#else
#endif
#define a5
#define c5
#define m4
#define n4
#ifdef VF
#define m6(g,L1,a) j5(v3,g,a)
#define k5(g,L1,a) Li(v3,g,a)
#define n6(g,L1,a) V6(v3,g,a)
#define q0(a,E0) v1(a,g0((E0)&ae,(E0)>>Zd))
#define y5(a,E0) v1(a,g0((E0)&ae,(E0)>>Zd)).xy
#else
#ifdef GL_ARB_shader_storage_buffer_object
#extension GL_ARB_shader_storage_buffer_object:require
#endif
#define m6(g,L1,a) layout(std430,binding=g) readonly buffer L1{R0 v2[];}a
#define k5(g,L1,a) layout(std430,binding=g) readonly buffer L1{O v2[];}a
#define n6(g,L1,a) layout(std430,binding=g) readonly buffer L1{e v2[];}a
#define dc(g,L1,a) layout(std430,binding=g) buffer L1{uint v2[];}a
#define q0(a,E0) a.v2[E0]
#define y5(a,E0) a.v2[E0]
#define Te(a,E0) a.v2[E0]
#define k8(a,E0,F) atomicMax(a.v2[E0],F)
#define ec(a,E0,F) atomicAdd(a.v2[E0],F)
#define Mi(a,E0,F) atomicOr(a.v2[E0],F)
#endif
#ifdef ND
#define I1(a) void main(){g0 G=ivec2(floor(c0));int M0=int(N9(uvec2(G),(j.T6+(Fb-1u))&~(Fb-1u)));
#define O1 }
#define o4 ,int M0
#define a2 ,M0
#ifdef ZE
#define V2(g,a) layout(std430,set=K3,binding=g) buffer a##Ue{uint v2[];}a
#elif defined(EC)
#define V2(g,a) layout(std430,set=K3,binding=g) coherent buffer a##Ue{uint v2[];}a
#else
#define V2(g,a) layout(std430,binding=g) coherent buffer a##Ue{uint v2[];}a
#endif
#define S5 V2
#define n3(h) h.v2[M0]
#define o3(h,D) h.v2[M0]=D
#define N5(h) unpackUnorm4x8(n3(h))
#define P5(h,D) o3(h,packUnorm4x8(D))
#define A5(h,F) atomicMax(h.v2[M0],F)
#define B5(h,F) atomicAdd(h.v2[M0],F)
#elif defined(GE)||defined(WF)
#ifdef GL_ARB_shader_image_load_store
#extension GL_ARB_shader_image_load_store:require
#endif
#define I1(a) void main(){g0 G=ivec2(floor(c0));
#define O1 }
#define o4 ,g0 G
#define a2 ,G
#ifdef EC
#define S5(g,a) layout(set=K3,binding=g,rgba8) uniform mediump coherent image2D a
#define V2(g,a) layout(set=K3,binding=g,r32ui) uniform highp coherent uimage2D a
#define fc(g,a) layout(set=K3,binding=g,rgb10_a2) uniform mediump coherent image2D a
#else
#define S5(g,a) layout(binding=g,rgba8) uniform mediump coherent image2D a
#define V2(g,a) layout(binding=g,r32ui) uniform highp coherent uimage2D a
#define fc(g,a) layout(binding=g,rgb10_a2) uniform mediump coherent image2D a;
#endif
#define n3(h) imageLoad(h,G).x
#define o3(h,D) imageStore(h,G,uvec4(D))
#define N5(h) imageLoad(h,G)
#define P5(h,D) imageStore(h,G,D)
#define A5(h,F) imageAtomicMax(h,G,F)
#define B5(h,F) imageAtomicAdd(h,G,F)
#else
#define I1(a) void main()
#define O1
#define o4
#define a2
#endif
#ifdef EXPORTED_PLS_IMPL_ANGLE
#extension GL_ANGLE_shader_pixel_local_storage:require
#define F1
#define y0(g,a) layout(binding=g,rgba8) uniform mediump pixelLocalANGLE a
#define r1(g,a) layout(binding=g,r32ui) uniform highp upixelLocalANGLE a
#define G1
#define K0(h) pixelLocalLoadANGLE(h)
#define k1(h) pixelLocalLoadANGLE(h).x
#define x0(h,D) pixelLocalStoreANGLE(h,D)
#define m1(h,D) pixelLocalStoreANGLE(h,uvec4(D))
#define O2(h)
#define i2(h)
#define P2
#define Q2
#endif
#ifdef XF
#ifdef R
#extension GL_EXT_shader_pixel_local_storage2:require
#else
#extension GL_EXT_shader_pixel_local_storage:require
#endif
#define F1 __pixel_localEXT j2{
#define y0(g,a) layout(rgba8) mediump vec4 a
#define gc(g,a) layout(rgb10_a2) mediump vec4 a
#define r1(g,a) layout(r32ui) highp uint a
#define G1 };
#define K0(h) h
#define k1(h) h
#define x0(h,D) h=(D)
#define m1(h,D) h=(D)
#define O2(h) h=h
#define i2(h) h=h
#define P2
#define Q2
#ifdef R
#define H2(a) layout(location=0,rgba8) out i Q1;I1(a)
#endif
#endif
#if defined(GE)||defined(ND)
#define F1
#define G1
#define y0 S5
#define r1 V2
#define gc fc
#define K0 N5
#define x0 P5
#define k1 n3
#define m1 o3
#define O2(h)
#define i2(h)
#if defined(GL_ARB_fragment_shader_interlock)
#extension GL_ARB_fragment_shader_interlock:require
#define P2 beginInvocationInterlockARB()
#define Q2 endInvocationInterlockARB()
#elif defined(GL_INTEL_fragment_shader_ordering)
#extension GL_INTEL_fragment_shader_ordering:require
#define P2 beginFragmentShaderOrderingINTEL()
#define Q2
#else
#define P2
#define Q2
#endif
#endif
#ifdef YF
#define F1
#define R4(g,a) layout(input_attachment_index=g,binding=g,set=K3) uniform mediump subpassInput l8##a
#define Ve(g,a) layout(location=g) out mediump vec4 a
#define y0(g,a) R4(g,a);Ve(g,a)
#define r1(g,a) layout(input_attachment_index=g,binding=g,set=K3) uniform highp usubpassInput l8##a;layout(location=g) out highp uvec4 a
#define G1
#define K0(h) subpassLoad(l8##h)
#define k1(h) subpassLoad(l8##h).x
#define x0(h,D) h=(D)
#define m1(h,D) h.x=(D)
#define O2(h) x0(h,subpassLoad(l8##h))
#define i2(h) m1(h,subpassLoad(l8##h).x)
#define P2
#define Q2
#endif
#ifdef ZF
#define F1
#define y0(g,a) layout(location=g) out mediump vec4 a
#define r1(g,a) layout(location=g) out highp uvec4 a
#define G1
#define K0(h) vec4(0)
#define k1(h) 0u
#define x0(h,D) h=(D)
#define m1(h,D) h.x=(D)
#define O2(h) h=vec4(0)
#define i2(h) h.x=0u
#define P2
#define Q2
#endif
#ifndef R4
#define R4 y0
#endif
#ifdef EC
#define hc gl_VertexIndex
#ifdef HE
#define m8 gl_InstanceIndex
#else
#define m8 0
#endif
#else
#ifdef IE
uniform highp int AF;
#define hc (gl_VertexID+AF)
#else
#define hc gl_VertexID
#endif
#ifdef HE
#ifdef JE
uniform highp int JE;
#define m8 (gl_InstanceID+JE)
#else
#define m8 (gl_InstanceID+gl_BaseInstance)
#endif
#else
#define m8 0
#endif
#endif
#define Q6
#define S3
#define A6
#define C4
#define o1(a,e0,A,N2,G6) void main(){int N2=hc;int G6=m8;
#define G8(a,e0,A,E1,j0,N2,G6) o1(a,e0,A,N2,G6)
#define l7(a,D3,E3,V3,j3,E1,j0,N2) o1(a,D3,E3,N2,G6)
#define V(a,k0)
#define a0(a)
#define q(a,k0)
#define A1(U0) gl_Position=U0;}
#define L2(w1,a) layout(location=0) out w1 Ni;void main()
#define Y6(w1,a) L2(w1,a)
#define Z6 gl_FrontFacing
#define M2(D) Ni=D
#define c0 gl_FragCoord.xy
#define o7
#define m3
#if defined(GE)||defined(ND)
#define We(n8,h,D) if(!(n8)){x0(h,D);}
#define Xe(n8,h,D) if(!(n8)){m1(h,D);}
#else
#define We(n8,h,D) x0(h,D);
#define Xe(n8,h,D) m1(h,D);
#endif
#ifndef H2
#define H2(a) layout(location=0) out i Q1;I1(a)
#endif
#define F3 O1
#if defined(EC)&&!defined(ZE)
#ifdef YE
#define R5(a) layout(input_attachment_index=0,binding=x2,set=K3) uniform mediump subpassInputMS a
#define O5(a) pb(mat4(subpassLoad(a,0),subpassLoad(a,1),subpassLoad(a,2),subpassLoad(a,3)),gl_SampleMaskIn[0])
#else
#define R5(a) layout(input_attachment_index=0,binding=x2,set=K3) uniform mediump subpassInput a
#define O5(a) subpassLoad(a)
#endif
#else
#define R5(a) q3(v3,Hh,a)
#define O5(a) texelFetch(a,ivec2(floor(c0.xy)),0)
#endif
#define C0(B,J) ((B)*(J))
precision highp float;precision highp int;
#if LC<310
f i Oi(uint u){O x1=O(u&0xffu,(u>>8)&0xffu,(u>>16)&0xffu,u>>24);return e(x1)*(1./255.);}
#define unpackUnorm4x8 Oi
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive