#pragma once

#include "glsl.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char glsl[] = R"===(#define rc
#ifndef LC
#define LC __VERSION__
#endif
#define d vec2
#define Q vec3
#define M3 vec3
#define g vec4
#define c mediump float
#define E mediump vec2
#define v mediump vec3
#define i mediump vec4
#define Z6 mediump mat3x3
#define a7 mediump mat2x3
#define G4 mediump mat4x4
#define Y ivec2
#define e6 ivec4
#define a1 uvec2
#define X uvec4
#define K mediump uint
#define F4 bvec2
#define q6 bvec3
#define y7 bvec4
#define f0 mat2
#define e
#define Z0(l2) out l2
#define X4(l2) inout l2
#ifdef GL_ANGLE_base_vertex_base_instance_shader_builtin
#extension GL_ANGLE_base_vertex_base_instance_shader_builtin:require
#endif
#ifdef IE
#extension GL_KHR_blend_equation_advanced:require
#endif
#ifdef VD
#extension GL_EXT_shader_framebuffer_fetch:require
#elif defined(WD)
#extension GL_EXT_shader_pixel_local_storage:require
#elif defined(EXPORTED_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
#extension GL_ANGLE_shader_pixel_local_storage:require
#elif defined(XD)
#ifdef GL_ARB_shader_image_load_store
#extension GL_ARB_shader_image_load_store:require
#endif
#ifdef GL_OES_shader_image_atomic
#extension GL_OES_shader_image_atomic:require
#endif
#endif
#if defined(CB)&&defined(BB)&&defined(GL_ES)&&!defined(NE)
#ifdef GL_EXT_clip_cull_distance
#extension GL_EXT_clip_cull_distance:require
#elif defined(GL_ANGLE_clip_cull_distance)
#extension GL_ANGLE_clip_cull_distance:require
#endif
#endif
#if LC>=310
#define x7(f,a) layout(binding=f,std140)uniform a{
#else
#define x7(f,a) layout(std140)uniform a{
#endif
#define M8(a) }a;
#define Bd(a) layout(push_constant)uniform a{
#define Cd(Z,a) Z a;
#define Dd(a) }a;
#define g1(a)
#define O(f,Z,a) layout(location=f)in Z a
#define h1
#define P(P8,F,a,Z)
#ifdef DB
#if LC>=310
#define W(f,Z,a) layout(location=f)out Z a
#else
#define W(f,Z,a) out Z a
#endif
#else
#if LC>=310
#define W(f,Z,a) layout(location=f)in Z a
#else
#define W(f,Z,a) in Z a
#endif
#endif
#define Q2 flat
#define m2
#define g2
#ifdef CC
#define H0
#else
#ifdef GL_NV_shader_noperspective_interpolation
#extension GL_NV_shader_noperspective_interpolation:require
#define H0 noperspective
#else
#define H0
#endif
#endif
#ifdef DB
#define T3
#define U3
#endif
#ifdef GB
#define D3
#define E3
#endif
#define e5
#define f5
#ifdef CC
#define E4(T,f,a) layout(set=T,binding=f)uniform highp utexture2D a
#define h5(T,f,a) layout(set=T,binding=f)uniform highp texture2D a
#define Z2(T,f,a) layout(set=T,binding=f)uniform mediump texture2D a
#define m5(T,f,a) layout(binding=f)uniform mediump texture2D a
#if defined(GB)&&defined(CB)
#endif
#elif LC>=310
#define E4(T,f,a) layout(binding=f)uniform highp usampler2D a
#define h5(T,f,a) layout(binding=f)uniform highp sampler2D a
#define Z2(T,f,a) layout(binding=f)uniform mediump sampler2D a
#define m5(T,f,a) layout(binding=f)uniform mediump sampler2D a
#else
#define E4(T,f,a) uniform highp usampler2D a
#define h5(T,f,a) uniform highp sampler2D a
#define Z2(T,f,a) uniform mediump sampler2D a
#define m5(T,f,a) uniform mediump sampler2D a
#endif
#ifdef CC
#define r6(T,f,a) layout(set=T,binding=f)uniform mediump sampler a;
#ifdef KF
#define a4(z7,a) layout(set=Qf,binding=z7)uniform mediump sampler a;
#define W3(a) r6(d5,Pf,a)
#else
#define a4(z7,a) layout(set=d3,binding=z7)uniform mediump sampler a;
#define W3(a) r6(d5,V3,a)
#endif
#define v5(a,p,l) texture(sampler2D(a,p),l)
#define o2(a,p,l,S0) textureLod(sampler2D(a,p),l,S0)
#define w5(a,p,l,R1) texture(sampler2D(a,p),l,R1)
#if defined(GB)&&defined(CB)
#extension GL_OES_sample_variables:require
#endif
#else
#define a4(z7,a)
#define r6(T,f,a)
#define W3(a)
#define v5(a,p,l) texture(a,l)
#define o2(a,p,l,S0) textureLod(a,l,S0)
#define w5(a,p,l,R1) texture(a,l,R1)
#endif
#define g8(k0,p,l) v5(k0,p,l)
#define U6(k0,p,l,S0) o2(k0,p,l,S0)
#define A7(k0,p,l,R1) w5(k0,p,l,R1)
#define i6(T,f,a) m5(T,f,a)
#define Y6(a,p,q,v6,R8,S0) o2(a,p,d(q,R8),S0)
#define Rg(T,f,a) E4(T,f,a)
#define H3
#define d1
#define q1(a,l) texelFetch(a,l,0)
#ifdef CC
#elif LC>=310
#else
#endif
#define B4
#define C4
#define O3
#define P3
#ifdef LF
#define N5(f,w1,a) E4(d3,f,a)
#define J4(f,w1,a) Rg(d3,f,a)
#define O5(f,w1,a) h5(d3,f,a)
#define J0(a,A0) q1(a,Y((A0)&Hc,(A0)>>Gc))
#define P5(a,A0) q1(a,Y((A0)&Hc,(A0)>>Gc)).xy
#else
#ifdef GL_ARB_shader_storage_buffer_object
#extension GL_ARB_shader_storage_buffer_object:require
#endif
#define N5(f,w1,a) layout(std430,binding=f)readonly buffer w1{a1 d2[];}a
#define J4(f,w1,a) layout(std430,binding=f)readonly buffer w1{X d2[];}a
#define O5(f,w1,a) layout(std430,binding=f)readonly buffer w1{g d2[];}a
#define Pa(f,w1,a) layout(std430,binding=f)buffer w1{uint d2[];}a
#define J0(a,A0) a.d2[A0]
#define P5(a,A0) a.d2[A0]
#define Hd(a,A0) a.d2[A0]
#define C7(a,A0,q) atomicMax(a.d2[A0],q)
#define Qa(a,A0,q) atomicAdd(a.d2[A0],q)
#define Sg(a,A0,q) atomicOr(a.d2[A0],q)
#endif
#ifdef FD
#define M1(a) void main(){Y G=ivec2(floor(a0));int E0=int(L8(uvec2(G),(m.p6+(wa-1u))&~(wa-1u)));
#define Z1 }
#define Q3 ,int E0
#define N1 ,E0
#ifdef YD
#define E2(f,a) layout(std430,set=G3,binding=f)buffer a##Id{uint d2[];}a
#elif defined(CC)
#define E2(f,a) layout(std430,set=G3,binding=f)coherent buffer a##Id{uint d2[];}a
#else
#define E2(f,a) layout(std430,binding=f)coherent buffer a##Id{uint d2[];}a
#endif
#define Ra E2
#define V2(h) h.d2[E0]
#define W2(h,D) h.d2[E0]=D
#define Sa(h) unpackUnorm4x8(V2(h))
#define Ta(h,D) W2(h,packUnorm4x8(D))
#define Z4(h,q) atomicMax(h.d2[E0],q)
#define a5(h,q) atomicAdd(h.d2[E0],q)
#elif defined(ZD)||defined(MF)
#ifdef GL_ARB_shader_image_load_store
#extension GL_ARB_shader_image_load_store:require
#endif
#define M1(a) void main(){Y G=ivec2(floor(a0));
#define Z1 }
#define Q3 ,Y G
#define N1 ,G
#ifdef CC
#define Ra(f,a) layout(set=G3,binding=f,rgba8)uniform mediump coherent image2D a
#define E2(f,a) layout(set=G3,binding=f,r32ui)uniform highp coherent uimage2D a
#define Ua(f,a) layout(set=G3,binding=f,rgb10_a2)uniform mediump coherent image2D a
#else
#define Ra(f,a) layout(binding=f,rgba8)uniform mediump coherent image2D a
#define E2(f,a) layout(binding=f,r32ui)uniform highp coherent uimage2D a
#define Ua(f,a) layout(binding=f,rgb10_a2)uniform mediump coherent image2D a;
#endif
#define V2(h) imageLoad(h,G).x
#define W2(h,D) imageStore(h,G,uvec4(D))
#define Sa(h) imageLoad(h,G)
#define Ta(h,D) imageStore(h,G,D)
#define Z4(h,q) imageAtomicMax(h,G,q)
#define a5(h,q) imageAtomicAdd(h,G,q)
#else
#define M1(a) void main()
#define Z1
#define Q3
#define N1
#endif
#ifdef EXPORTED_PLS_IMPL_ANGLE
#extension GL_ANGLE_shader_pixel_local_storage:require
#define J1
#define x0(f,a) layout(binding=f,rgba8)uniform mediump pixelLocalANGLE a
#define j1(f,a) layout(binding=f,r32ui)uniform highp upixelLocalANGLE a
#define K1
#define I0(h) pixelLocalLoadANGLE(h)
#define Y0(h) pixelLocalLoadANGLE(h).x
#define y0(h,D) pixelLocalStoreANGLE(h,D)
#define c1(h,D) pixelLocalStoreANGLE(h,uvec4(D))
#define w2(h)
#define e2(h)
#define x2
#define y2
#endif
#ifdef NF
#ifdef N
#extension GL_EXT_shader_pixel_local_storage2:require
#else
#extension GL_EXT_shader_pixel_local_storage:require
#endif
#define J1 __pixel_localEXT S1{
#define x0(f,a) layout(rgba8)mediump vec4 a
#define Va(f,a) layout(rgb10_a2)mediump vec4 a
#define j1(f,a) layout(r32ui)highp uint a
#define K1 };
#define I0(h) h
#define Y0(h) h
#define y0(h,D) h=(D)
#define c1(h,D) h=(D)
#define w2(h) h=h
#define e2(h) h=h
#define x2
#define y2
#ifdef N
#define p2(a) layout(location=0,rgba8)out i D1;M1(a)
#endif
#endif
#if defined(ZD)||defined(FD)
#define J1
#define K1
#define x0 Ra
#define j1 E2
#define Va Ua
#define I0 Sa
#define y0 Ta
#define Y0 V2
#define c1 W2
#define w2(h)
#define e2(h)
#if defined(GL_ARB_fragment_shader_interlock)
#extension GL_ARB_fragment_shader_interlock:require
#define x2 beginInvocationInterlockARB()
#define y2 endInvocationInterlockARB()
#elif defined(GL_INTEL_fragment_shader_ordering)
#extension GL_INTEL_fragment_shader_ordering:require
#define x2 beginFragmentShaderOrderingINTEL()
#define y2
#else
#define x2
#define y2
#endif
#endif
#ifdef OF
#define J1
#define r4(f,a) layout(input_attachment_index=f,binding=f,set=G3)uniform mediump subpassInput D7##a
#define Jd(f,a) layout(location=f)out mediump vec4 a
#define x0(f,a) r4(f,a);Jd(f,a)
#define j1(f,a) layout(input_attachment_index=f,binding=f,set=G3)uniform highp usubpassInput D7##a;layout(location=f)out highp uvec4 a
#define K1
#define I0(h) subpassLoad(D7##h)
#define Y0(h) subpassLoad(D7##h).x
#define y0(h,D) h=(D)
#define c1(h,D) h.x=(D)
#define w2(h) y0(h,subpassLoad(D7##h))
#define e2(h) c1(h,subpassLoad(D7##h).x)
#define x2
#define y2
#endif
#ifdef PF
#define J1
#define x0(f,a) layout(location=f)out mediump vec4 a
#define j1(f,a) layout(location=f)out highp uvec4 a
#define K1
#define I0(h) vec4(0)
#define Y0(h) 0u
#define y0(h,D) h=(D)
#define c1(h,D) h.x=(D)
#define w2(h) h=vec4(0)
#define e2(h) h.x=0u
#define x2
#define y2
#endif
#ifndef r4
#define r4 x0
#endif
#ifdef CC
#define gl_VertexID gl_VertexIndex
#endif
#ifdef OE
#ifdef CC
#define S8 gl_InstanceIndex
#else
#ifdef AE
uniform highp int AE;
#define S8 (gl_InstanceID+AE)
#else
#define S8 (gl_InstanceID+gl_BaseInstance)
#endif
#endif
#else
#define S8 0
#endif
#define m6
#define v3
#define f7
#define x5
#define z1(a,e0,F,B,A) void main(){int B=gl_VertexID;int A=S8;
#define S7(a,e0,F,n1,i0,B,A) z1(a,e0,F,B,A)
#define I6(a,i3,j3,x3,y3,n1,i0,B) z1(a,i3,j3,B,A)
#define U(a,Z)
#define c0(a)
#define r(a,Z)
#define A1(O0) gl_Position=O0;}
#define a3(x1,a) layout(location=0)out x1 Tg;void main()
#define w6(x1,a) a3(x1,a)
#define x6 gl_FrontFacing
#define I2(D) Tg=D
#define a0 gl_FragCoord.xy
#define K6
#define U2
#if defined(ZD)||defined(FD)
#define Kd(E7,h,D) if(!(E7)){y0(h,D);}
#define Ld(E7,h,D) if(!(E7)){c1(h,D);}
#else
#define Kd(E7,h,D) y0(h,D);
#define Ld(E7,h,D) c1(h,D);
#endif
#ifndef p2
#define p2(a) layout(location=0)out i D1;M1(a)
#endif
#define m3 Z1
#if defined(CC)&&!defined(YD)
#define j6(a) layout(input_attachment_index=0,binding=S2,set=G3)uniform mediump subpassInputMS a
#define F7(a) ka(mat4(subpassLoad(a,0),subpassLoad(a,1),subpassLoad(a,2),subpassLoad(a,3)),gl_SampleMaskIn[0])
#else
#define j6(a) Z2(d3,Of,a)
#define F7(a) texelFetch(a,ivec2(floor(a0.xy)),0)
#endif
#define R0(C,H) ((C)*(H))
precision highp float;precision highp int;
#if LC<310
e i Ug(uint u){X T1=X(u&0xffu,(u>>8)&0xffu,(u>>16)&0xffu,u>>24);return g(T1)*(1./255.);}
#define unpackUnorm4x8 Ug
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive