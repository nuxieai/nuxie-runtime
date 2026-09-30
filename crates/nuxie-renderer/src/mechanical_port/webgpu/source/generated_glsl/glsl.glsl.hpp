#pragma once

#include "glsl.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char glsl[] = R"===(#define Bc
#ifndef KC
#define KC __VERSION__
#endif
#define c vec2
#define R vec3
#define R3 vec3
#define f vec4
#define d mediump float
#define E mediump vec2
#define A mediump vec3
#define i mediump vec4
#define c7 mediump mat3x3
#define d7 mediump mat2x3
#define J4 mediump mat4x4
#define Z ivec2
#define g6 ivec4
#define c1 uvec2
#define Y uvec4
#define L mediump uint
#define I4 bvec2
#define r6 bvec3
#define A7 bvec4
#define e0 mat2
#define e
#define a1(o2) out o2
#define M6(o2) inout o2
#ifdef GL_ANGLE_base_vertex_base_instance_shader_builtin
#extension GL_ANGLE_base_vertex_base_instance_shader_builtin:require
#endif
#ifdef LE
#extension GL_KHR_blend_equation_advanced:require
#endif
#ifdef YD
#extension GL_EXT_shader_framebuffer_fetch:require
#elif defined(ZD)
#extension GL_EXT_shader_pixel_local_storage:require
#elif defined(EXPORTED_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
#extension GL_ANGLE_shader_pixel_local_storage:require
#elif defined(AE)
#ifdef GL_ARB_shader_image_load_store
#extension GL_ARB_shader_image_load_store:require
#endif
#ifdef GL_OES_shader_image_atomic
#extension GL_OES_shader_image_atomic:require
#endif
#endif
#if defined(BB)&&defined(AB)&&defined(GL_ES)&&!defined(RE)
#ifdef GL_EXT_clip_cull_distance
#extension GL_EXT_clip_cull_distance:require
#elif defined(GL_ANGLE_clip_cull_distance)
#extension GL_ANGLE_clip_cull_distance:require
#endif
#endif
#if KC>=310
#define z7(g,a) layout(binding=g,std140)uniform a{
#else
#define z7(g,a) layout(std140)uniform a{
#endif
#define P8(a) }a;
#define Md(a) layout(push_constant)uniform a{
#define Nd(a0,a) a0 a;
#define Od(a) }a;
#define h1(a)
#define J(g,a0,a) layout(location=g)in a0 a
#define i1
#define K(S8,F,a,a0)
#ifdef CB
#if KC>=310
#define W(g,a0,a) layout(location=g)out a0 a
#else
#define W(g,a0,a) out a0 a
#endif
#else
#if KC>=310
#define W(g,a0,a) layout(location=g)in a0 a
#else
#define W(g,a0,a) in a0 a
#endif
#endif
#define T2 flat
#define q2
#define i2
#ifdef BC
#define I0
#else
#ifdef GL_NV_shader_noperspective_interpolation
#extension GL_NV_shader_noperspective_interpolation:require
#define I0 noperspective
#else
#define I0
#endif
#endif
#ifdef CB
#define X3
#define Y3
#endif
#ifdef EB
#define I3
#define J3
#endif
#define h5
#define i5
#ifdef BC
#define H4(V,g,a) layout(set=V,binding=g)uniform highp utexture2D a
#define k5(V,g,a) layout(set=V,binding=g)uniform highp texture2D a
#define c3(V,g,a) layout(set=V,binding=g)uniform mediump texture2D a
#define p5(V,g,a) layout(binding=g)uniform mediump texture2D a
#if defined(EB)&&defined(BB)
#endif
#elif KC>=310
#define H4(V,g,a) layout(binding=g)uniform highp usampler2D a
#define k5(V,g,a) layout(binding=g)uniform highp sampler2D a
#define c3(V,g,a) layout(binding=g)uniform mediump sampler2D a
#define p5(V,g,a) layout(binding=g)uniform mediump sampler2D a
#else
#define H4(V,g,a) uniform highp usampler2D a
#define k5(V,g,a) uniform highp sampler2D a
#define c3(V,g,a) uniform mediump sampler2D a
#define p5(V,g,a) uniform mediump sampler2D a
#endif
#ifdef BC
#define v6(V,g,a) layout(set=V,binding=g)uniform mediump sampler a;
#ifdef PF
#define f4(B7,a) layout(set=jg,binding=B7)uniform mediump sampler a;
#define a4(a) v6(g5,ig,a)
#else
#define f4(B7,a) layout(set=f3,binding=B7)uniform mediump sampler a;
#define a4(a) v6(g5,Z3,a)
#endif
#define z5(a,p,l) texture(sampler2D(a,p),l)
#define j2(a,p,l,T0) textureLod(sampler2D(a,p),l,T0)
#define A5(a,p,l,T1) texture(sampler2D(a,p),l,T1)
#if defined(EB)&&defined(BB)&&defined(SE)
#extension GL_OES_sample_variables:require
#endif
#else
#define f4(B7,a)
#define v6(V,g,a)
#define a4(a)
#define z5(a,p,l) texture(a,l)
#define j2(a,p,l,T0) textureLod(a,l,T0)
#define A5(a,p,l,T1) texture(a,l,T1)
#endif
#define j8(m0,p,l) z5(m0,p,l)
#define W6(m0,p,l,T0) j2(m0,p,l,T0)
#define C7(m0,p,l,T1) A5(m0,p,l,T1)
#define k6(V,g,a) p5(V,g,a)
#define a7(a,p,q,w6,U8,T0) j2(a,p,c(q,U8),T0)
#define jh(V,g,a) H4(V,g,a)
#define L3
#define e1
#define r1(a,l) texelFetch(a,l,0)
#ifdef BC
#elif KC>=310
#else
#endif
#define E4
#define F4
#define T3
#define U3
#ifdef QF
#define Q5(g,x1,a) H4(f3,g,a)
#define M4(g,x1,a) jh(f3,g,a)
#define R5(g,x1,a) k5(f3,g,a)
#define L0(a,C0) r1(a,Z((C0)&Rc,(C0)>>Qc))
#define T5(a,C0) r1(a,Z((C0)&Rc,(C0)>>Qc)).xy
#else
#ifdef GL_ARB_shader_storage_buffer_object
#extension GL_ARB_shader_storage_buffer_object:require
#endif
#define Q5(g,x1,a) layout(std430,binding=g)readonly buffer x1{c1 g2[];}a
#define M4(g,x1,a) layout(std430,binding=g)readonly buffer x1{Y g2[];}a
#define R5(g,x1,a) layout(std430,binding=g)readonly buffer x1{f g2[];}a
#define Va(g,x1,a) layout(std430,binding=g)buffer x1{uint g2[];}a
#define L0(a,C0) a.g2[C0]
#define T5(a,C0) a.g2[C0]
#define Sd(a,C0) a.g2[C0]
#define E7(a,C0,q) atomicMax(a.g2[C0],q)
#define Wa(a,C0,q) atomicAdd(a.g2[C0],q)
#define kh(a,C0,q) atomicOr(a.g2[C0],q)
#endif
#ifdef ID
#define O1(a) void main(){Z G=ivec2(floor(d0));int G0=int(O8(uvec2(G),(j.q6+(Ca-1u))&~(Ca-1u)));
#define d2 }
#define V3 ,int G0
#define P1 ,G0
#ifdef BE
#define H2(g,a) layout(std430,set=v3,binding=g)buffer a##Td{uint g2[];}a
#elif defined(BC)
#define H2(g,a) layout(std430,set=v3,binding=g)coherent buffer a##Td{uint g2[];}a
#else
#define H2(g,a) layout(std430,binding=g)coherent buffer a##Td{uint g2[];}a
#endif
#define Xa H2
#define X2(h) h.g2[G0]
#define Y2(h,D) h.g2[G0]=D
#define Ya(h) unpackUnorm4x8(X2(h))
#define Za(h,D) Y2(h,packUnorm4x8(D))
#define d5(h,q) atomicMax(h.g2[G0],q)
#define e5(h,q) atomicAdd(h.g2[G0],q)
#elif defined(CE)||defined(RF)
#ifdef GL_ARB_shader_image_load_store
#extension GL_ARB_shader_image_load_store:require
#endif
#define O1(a) void main(){Z G=ivec2(floor(d0));
#define d2 }
#define V3 ,Z G
#define P1 ,G
#ifdef BC
#define Xa(g,a) layout(set=v3,binding=g,rgba8)uniform mediump coherent image2D a
#define H2(g,a) layout(set=v3,binding=g,r32ui)uniform highp coherent uimage2D a
#define ab(g,a) layout(set=v3,binding=g,rgb10_a2)uniform mediump coherent image2D a
#else
#define Xa(g,a) layout(binding=g,rgba8)uniform mediump coherent image2D a
#define H2(g,a) layout(binding=g,r32ui)uniform highp coherent uimage2D a
#define ab(g,a) layout(binding=g,rgb10_a2)uniform mediump coherent image2D a;
#endif
#define X2(h) imageLoad(h,G).x
#define Y2(h,D) imageStore(h,G,uvec4(D))
#define Ya(h) imageLoad(h,G)
#define Za(h,D) imageStore(h,G,D)
#define d5(h,q) imageAtomicMax(h,G,q)
#define e5(h,q) imageAtomicAdd(h,G,q)
#else
#define O1(a) void main()
#define d2
#define V3
#define P1
#endif
#ifdef EXPORTED_PLS_IMPL_ANGLE
#extension GL_ANGLE_shader_pixel_local_storage:require
#define L1
#define z0(g,a) layout(binding=g,rgba8)uniform mediump pixelLocalANGLE a
#define k1(g,a) layout(binding=g,r32ui)uniform highp upixelLocalANGLE a
#define M1
#define K0(h) pixelLocalLoadANGLE(h)
#define Z0(h) pixelLocalLoadANGLE(h).x
#define A0(h,D) pixelLocalStoreANGLE(h,D)
#define d1(h,D) pixelLocalStoreANGLE(h,uvec4(D))
#define y2(h)
#define h2(h)
#define z2
#define A2
#endif
#ifdef SF
#ifdef O
#extension GL_EXT_shader_pixel_local_storage2:require
#else
#extension GL_EXT_shader_pixel_local_storage:require
#endif
#define L1 __pixel_localEXT U1{
#define z0(g,a) layout(rgba8)mediump vec4 a
#define bb(g,a) layout(rgb10_a2)mediump vec4 a
#define k1(g,a) layout(r32ui)highp uint a
#define M1 };
#define K0(h) h
#define Z0(h) h
#define A0(h,D) h=(D)
#define d1(h,D) h=(D)
#define y2(h) h=h
#define h2(h) h=h
#define z2
#define A2
#ifdef O
#define v2(a) layout(location=0,rgba8)out i E1;O1(a)
#endif
#endif
#if defined(CE)||defined(ID)
#define L1
#define M1
#define z0 Xa
#define k1 H2
#define bb ab
#define K0 Ya
#define A0 Za
#define Z0 X2
#define d1 Y2
#define y2(h)
#define h2(h)
#if defined(GL_ARB_fragment_shader_interlock)
#extension GL_ARB_fragment_shader_interlock:require
#define z2 beginInvocationInterlockARB()
#define A2 endInvocationInterlockARB()
#elif defined(GL_INTEL_fragment_shader_ordering)
#extension GL_INTEL_fragment_shader_ordering:require
#define z2 beginFragmentShaderOrderingINTEL()
#define A2
#else
#define z2
#define A2
#endif
#endif
#ifdef TF
#define L1
#define x4(g,a) layout(input_attachment_index=g,binding=g,set=v3)uniform mediump subpassInput F7##a
#define Ud(g,a) layout(location=g)out mediump vec4 a
#define z0(g,a) x4(g,a);Ud(g,a)
#define k1(g,a) layout(input_attachment_index=g,binding=g,set=v3)uniform highp usubpassInput F7##a;layout(location=g)out highp uvec4 a
#define M1
#define K0(h) subpassLoad(F7##h)
#define Z0(h) subpassLoad(F7##h).x
#define A0(h,D) h=(D)
#define d1(h,D) h.x=(D)
#define y2(h) A0(h,subpassLoad(F7##h))
#define h2(h) d1(h,subpassLoad(F7##h).x)
#define z2
#define A2
#endif
#ifdef UF
#define L1
#define z0(g,a) layout(location=g)out mediump vec4 a
#define k1(g,a) layout(location=g)out highp uvec4 a
#define M1
#define K0(h) vec4(0)
#define Z0(h) 0u
#define A0(h,D) h=(D)
#define d1(h,D) h.x=(D)
#define y2(h) h=vec4(0)
#define h2(h) h.x=0u
#define z2
#define A2
#endif
#ifndef x4
#define x4 z0
#endif
#ifdef BC
#define gl_VertexID gl_VertexIndex
#endif
#ifdef TE
#ifdef BC
#define V8 gl_InstanceIndex
#else
#ifdef DE
uniform highp int DE;
#define V8 (gl_InstanceID+DE)
#else
#define V8 (gl_InstanceID+gl_BaseInstance)
#endif
#endif
#else
#define V8 0
#endif
#define n6
#define A3
#define h7
#define B5
#define A1(a,g0,F,B,v) void main(){int B=gl_VertexID;int v=V8;
#define V7(a,g0,F,o1,h0,B,v) A1(a,g0,F,B,v)
#define K6(a,l3,m3,C3,D3,o1,h0,B) A1(a,l3,m3,B,v)
#define U(a,a0)
#define c0(a)
#define r(a,a0)
#define B1(Q0) gl_Position=Q0;}
#define d3(y1,a) layout(location=0)out y1 lh;void main()
#define x6(y1,a) d3(y1,a)
#define y6 gl_FrontFacing
#define L2(D) lh=D
#define d0 gl_FragCoord.xy
#define N6
#define W2
#if defined(CE)||defined(ID)
#define Vd(G7,h,D) if(!(G7)){A0(h,D);}
#define Wd(G7,h,D) if(!(G7)){d1(h,D);}
#else
#define Vd(G7,h,D) A0(h,D);
#define Wd(G7,h,D) d1(h,D);
#endif
#ifndef v2
#define v2(a) layout(location=0)out i E1;O1(a)
#endif
#define p3 d2
#if defined(BC)&&!defined(BE)
#ifdef SE
#define q5(a) layout(input_attachment_index=0,binding=G2,set=v3)uniform mediump subpassInputMS a
#define z6(a) ra(mat4(subpassLoad(a,0),subpassLoad(a,1),subpassLoad(a,2),subpassLoad(a,3)),gl_SampleMaskIn[0])
#else
#define q5(a) layout(input_attachment_index=0,binding=G2,set=v3)uniform mediump subpassInput a
#define z6(a) subpassLoad(a)
#endif
#else
#define q5(a) c3(f3,hg,a)
#define z6(a) texelFetch(a,ivec2(floor(d0.xy)),0)
#endif
#define O0(C,H) ((C)*(H))
precision highp float;precision highp int;
#if KC<310
e i mh(uint u){Y V1=Y(u&0xffu,(u>>8)&0xffu,(u>>16)&0xffu,u>>24);return f(V1)*(1./255.);}
#define unpackUnorm4x8 mh
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive