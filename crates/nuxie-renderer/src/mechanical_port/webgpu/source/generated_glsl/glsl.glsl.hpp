#pragma once

#include "glsl.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char glsl[] = R"===(#define fb
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
#define D7 mediump mat3x3
#define E7 mediump mat2x3
#define h5 mediump mat4x4
#define g0 ivec2
#define B6 ivec4
#define T0 uvec2
#define O uvec4
#define P mediump uint
#define g5 bvec2
#define U6 bvec3
#define h8 bvec4
#define X mat2
#define f
#define l1(E2) out E2
#define m7(E2) inout E2
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
#define P9(a) }a;
#define e1(a)
#define K(g,k0,a) layout(location=g) in k0 a
#define f1
#define L(S9,A,a,k0)
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
#define H0
#else
#ifdef GL_NV_shader_noperspective_interpolation
#extension GL_NV_shader_noperspective_interpolation:require
#define H0 noperspective
#else
#define H0
#endif
#endif
#ifdef BB
#define x4
#define y4
#endif
#ifdef DB
#define Y3
#define Z3
#endif
#define E5
#define F5
#ifdef EC
#define l5(h0,g,a) layout(set=h0,binding=g) uniform highp utexture2D a
#define V6(h0,g,a) layout(set=h0,binding=g) uniform highp texture2D a
#define q3(h0,g,a) layout(set=h0,binding=g) uniform mediump texture2D a
#define Q5(h0,g,a) layout(binding=g) uniform mediump texture2D a
#if defined(DB)&&defined(CB)
#endif
#elif LC>=310
#define l5(h0,g,a) layout(binding=g) uniform highp usampler2D a
#define V6(h0,g,a) layout(binding=g) uniform highp sampler2D a
#define q3(h0,g,a) layout(binding=g) uniform mediump sampler2D a
#define Q5(h0,g,a) layout(binding=g) uniform mediump sampler2D a
#else
#define l5(h0,g,a) uniform highp usampler2D a
#define V6(h0,g,a) uniform highp sampler2D a
#define q3(h0,g,a) uniform mediump sampler2D a
#define Q5(h0,g,a) uniform mediump sampler2D a
#endif
#ifdef EC
#define W6(h0,g,a) layout(set=h0,binding=g) uniform mediump sampler a;
#ifdef UF
#define e4(V5,a) layout(set=Mh,binding=V5) uniform mediump sampler a;
#define N6 e4
#define A4(a) W6(D5,Lh,a)
#else
#define e4(V5,a) layout(set=v3,binding=V5) uniform mediump sampler a;
#define N6 e4
#define A4(a) W6(D5,z4,a)
#endif
#define W5(a,p,o) texture(sampler2D(a,p),o)
#define p2(a,p,o,c1) textureLod(sampler2D(a,p),o,c1)
#define X5(a,p,o,h2) texture(sampler2D(a,p),o,h2)
#if defined(DB)&&defined(CB)&&defined(YE)
#extension GL_OES_sample_variables:require
#endif
#else
#define e4(V5,a)
#define N6(V5,a)
#define W6(h0,g,a)
#define A4(a)
#define W5(a,p,o) texture(a,o)
#define p2(a,p,o,c1) textureLod(a,o,c1)
#define X5(a,p,o,h2) texture(a,o,h2)
#endif
#define Y8(r0,p,o) W5(r0,p,o)
#define e5(r0,p,o,c1) p2(r0,p,o,c1)
#define i8(r0,p,o,h2) X5(r0,p,o,h2)
#define M6(h0,g,a) Q5(h0,g,a)
#define C7(a,p,F,X6,U9,c1) p2(a,p,c(F,U9),c1)
#define Pi(h0,g,a) l5(h0,g,a)
#define f4
#define n1
#define v1(a,o) texelFetch(a,o,0)
#ifdef EC
#elif LC>=310
#else
#endif
#define c5
#define d5
#define m4
#define n4
#ifdef VF
#define m6(g,L1,a) l5(v3,g,a)
#define m5(g,L1,a) Pi(v3,g,a)
#define n6(g,L1,a) V6(v3,g,a)
#define q0(a,G0) v1(a,g0((G0)&de,(G0)>>ce))
#define z5(a,G0) v1(a,g0((G0)&de,(G0)>>ce)).xy
#else
#ifdef GL_ARB_shader_storage_buffer_object
#extension GL_ARB_shader_storage_buffer_object:require
#endif
#define m6(g,L1,a) layout(std430,binding=g) readonly buffer L1{T0 v2[];}a
#define m5(g,L1,a) layout(std430,binding=g) readonly buffer L1{O v2[];}a
#define n6(g,L1,a) layout(std430,binding=g) readonly buffer L1{e v2[];}a
#define hc(g,L1,a) layout(std430,binding=g) buffer L1{uint v2[];}a
#define q0(a,G0) a.v2[G0]
#define z5(a,G0) a.v2[G0]
#define bf(a,G0) a.v2[G0]
#define k8(a,G0,F) atomicMax(a.v2[G0],F)
#define ic(a,G0,F) atomicAdd(a.v2[G0],F)
#define Qi(a,G0,F) atomicOr(a.v2[G0],F)
#endif
#ifdef ND
#define I1(a) void main(){g0 G=ivec2(floor(c0));int P0=int(O9(uvec2(G),(j.T6+(Gb-1u))&~(Gb-1u)));
#define O1 }
#define p4 ,int P0
#define a2 ,P0
#ifdef ZE
#define V2(g,a) layout(std430,set=L3,binding=g) buffer a##cf{uint v2[];}a
#elif defined(EC)
#define V2(g,a) layout(std430,set=L3,binding=g) coherent buffer a##cf{uint v2[];}a
#else
#define V2(g,a) layout(std430,binding=g) coherent buffer a##cf{uint v2[];}a
#endif
#define S5 V2
#define n3(h) h.v2[P0]
#define o3(h,D) h.v2[P0]=D
#define N5(h) unpackUnorm4x8(n3(h))
#define P5(h,D) o3(h,packUnorm4x8(D))
#define B5(h,F) atomicMax(h.v2[P0],F)
#define C5(h,F) atomicAdd(h.v2[P0],F)
#elif defined(GE)||defined(WF)
#ifdef GL_ARB_shader_image_load_store
#extension GL_ARB_shader_image_load_store:require
#endif
#define I1(a) void main(){g0 G=ivec2(floor(c0));
#define O1 }
#define p4 ,g0 G
#define a2 ,G
#ifdef EC
#define S5(g,a) layout(set=L3,binding=g,rgba8) uniform mediump coherent image2D a
#define V2(g,a) layout(set=L3,binding=g,r32ui) uniform highp coherent uimage2D a
#define jc(g,a) layout(set=L3,binding=g,rgb10_a2) uniform mediump coherent image2D a
#else
#define S5(g,a) layout(binding=g,rgba8) uniform mediump coherent image2D a
#define V2(g,a) layout(binding=g,r32ui) uniform highp coherent uimage2D a
#define jc(g,a) layout(binding=g,rgb10_a2) uniform mediump coherent image2D a;
#endif
#define n3(h) imageLoad(h,G).x
#define o3(h,D) imageStore(h,G,uvec4(D))
#define N5(h) imageLoad(h,G)
#define P5(h,D) imageStore(h,G,D)
#define B5(h,F) imageAtomicMax(h,G,F)
#define C5(h,F) imageAtomicAdd(h,G,F)
#else
#define I1(a) void main()
#define O1
#define p4
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
#define kc(g,a) layout(rgb10_a2) mediump vec4 a
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
#define kc jc
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
#define U4(g,a) layout(input_attachment_index=g,binding=g,set=L3) uniform mediump subpassInput l8##a
#define df(g,a) layout(location=g) out mediump vec4 a
#define y0(g,a) U4(g,a);df(g,a)
#define r1(g,a) layout(input_attachment_index=g,binding=g,set=L3) uniform highp usubpassInput l8##a;layout(location=g) out highp uvec4 a
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
#ifndef U4
#define U4 y0
#endif
#ifdef EC
#define lc gl_VertexIndex
#ifdef HE
#define m8 gl_InstanceIndex
#else
#define m8 0
#endif
#else
#ifdef IE
uniform highp int AF;
#define lc (gl_VertexID+AF)
#else
#define lc gl_VertexID
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
#define T3
#define A6
#define E4
#define o1(a,e0,A,N2,G6) void main(){int N2=lc;int G6=m8;
#define H8(a,e0,A,E1,j0,N2,G6) o1(a,e0,A,N2,G6)
#define k7(a,E3,F3,W3,j3,E1,j0,N2) o1(a,E3,F3,N2,G6)
#define V(a,k0)
#define a0(a)
#define q(a,k0)
#define A1(V0) gl_Position=V0;}
#define L2(w1,a) layout(location=0) out w1 Ri;void main()
#define Y6(w1,a) L2(w1,a)
#define Z6 gl_FrontFacing
#define M2(D) Ri=D
#define c0 gl_FragCoord.xy
#define n7
#define m3
#if defined(GE)||defined(ND)
#define ef(n8,h,D) if(!(n8)){x0(h,D);}
#define ff(n8,h,D) if(!(n8)){m1(h,D);}
#else
#define ef(n8,h,D) x0(h,D);
#define ff(n8,h,D) m1(h,D);
#endif
#ifndef H2
#define H2(a) layout(location=0) out i Q1;I1(a)
#endif
#define G3 O1
#if defined(EC)&&!defined(ZE)
#ifdef YE
#define R5(a) layout(input_attachment_index=0,binding=x2,set=L3) uniform mediump subpassInputMS a
#define O5(a) qb(mat4(subpassLoad(a,0),subpassLoad(a,1),subpassLoad(a,2),subpassLoad(a,3)),gl_SampleMaskIn[0])
#else
#define R5(a) layout(input_attachment_index=0,binding=x2,set=L3) uniform mediump subpassInput a
#define O5(a) subpassLoad(a)
#endif
#else
#define R5(a) q3(v3,Kh,a)
#define O5(a) texelFetch(a,ivec2(floor(c0.xy)),0)
#endif
#define D0(B,J) ((B)*(J))
precision highp float;precision highp int;
#if LC<310
f i Si(uint u){O x1=O(u&0xffu,(u>>8)&0xffu,(u>>16)&0xffu,u>>24);return e(x1)*(1./255.);}
#define unpackUnorm4x8 Si
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive