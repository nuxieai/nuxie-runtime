#pragma once

#include "glsl.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char glsl[] = R"===(#define oc
#ifndef NC
#define NC __VERSION__
#endif
#define d vec2
#define R vec3
#define N3 vec3
#define g vec4
#define c mediump float
#define E mediump vec2
#define A mediump vec3
#define i mediump vec4
#define X6 mediump mat3x3
#define Y6 mediump mat2x3
#define i5 mediump mat4x4
#define Y ivec2
#define d6 ivec4
#define a1 uvec2
#define G uvec4
#define N mediump uint
#define F4 bvec2
#define p6 bvec3
#define x7 bvec4
#define f0 mat2
#define e
#define Z0(k2) out k2
#define V4(k2) inout k2
#ifdef GL_ANGLE_base_vertex_base_instance_shader_builtin
#extension GL_ANGLE_base_vertex_base_instance_shader_builtin:require
#endif
#ifdef JE
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
#if defined(CB)&&defined(BB)&&defined(GL_ES)&&!defined(OE)
#ifdef GL_EXT_clip_cull_distance
#extension GL_EXT_clip_cull_distance:require
#elif defined(GL_ANGLE_clip_cull_distance)
#extension GL_ANGLE_clip_cull_distance:require
#endif
#endif
#if NC>=310
#define w7(f,a) layout(binding=f,std140)uniform a{
#else
#define w7(f,a) layout(std140)uniform a{
#endif
#define L8(a) }a;
#define g1(a)
#define L(f,d0,a) layout(location=f)in d0 a
#define h1
#define M(O8,F,a,d0)
#ifdef DB
#if NC>=310
#define X(f,d0,a) layout(location=f)out d0 a
#else
#define X(f,d0,a) out d0 a
#endif
#else
#if NC>=310
#define X(f,d0,a) layout(location=f)in d0 a
#else
#define X(f,d0,a) in d0 a
#endif
#endif
#define Q2 flat
#define l2
#define f2
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
#ifdef DB
#define U3
#define V3
#endif
#ifdef GB
#define E3
#define F3
#endif
#define c5
#define d5
#ifdef EC
#define E4(U,f,a) layout(set=U,binding=f)uniform highp utexture2D a
#define f5(U,f,a) layout(set=U,binding=f)uniform highp texture2D a
#define Z2(U,f,a) layout(set=U,binding=f)uniform mediump texture2D a
#define l5(U,f,a) layout(binding=f)uniform mediump texture2D a
#if defined(GB)&&defined(CB)
#endif
#elif NC>=310
#define E4(U,f,a) layout(binding=f)uniform highp usampler2D a
#define f5(U,f,a) layout(binding=f)uniform highp sampler2D a
#define Z2(U,f,a) layout(binding=f)uniform mediump sampler2D a
#define l5(U,f,a) layout(binding=f)uniform mediump sampler2D a
#else
#define E4(U,f,a) uniform highp usampler2D a
#define f5(U,f,a) uniform highp sampler2D a
#define Z2(U,f,a) uniform mediump sampler2D a
#define l5(U,f,a) uniform mediump sampler2D a
#endif
#ifdef EC
#define q6(U,f,a) layout(set=U,binding=f)uniform mediump sampler a;
#ifdef KF
#define c4(y7,a) layout(set=Mf,binding=y7)uniform mediump sampler a;
#define X3(a) q6(a5,Lf,a)
#else
#define c4(y7,a) layout(set=d3,binding=y7)uniform mediump sampler a;
#define X3(a) q6(a5,W3,a)
#endif
#define r5(a,p,l) texture(sampler2D(a,p),l)
#define n2(a,p,l,S0) textureLod(sampler2D(a,p),l,S0)
#define v5(a,p,l,Q1) texture(sampler2D(a,p),l,Q1)
#if defined(GB)&&defined(CB)
#extension GL_OES_sample_variables:require
#endif
#else
#define c4(y7,a)
#define q6(U,f,a)
#define X3(a)
#define r5(a,p,l) texture(a,l)
#define n2(a,p,l,S0) textureLod(a,l,S0)
#define v5(a,p,l,Q1) texture(a,l,Q1)
#endif
#define f8(k0,p,l) r5(k0,p,l)
#define S6(k0,p,l,S0) n2(k0,p,l,S0)
#define z7(k0,p,l,Q1) v5(k0,p,l,Q1)
#define h6(U,f,a) l5(U,f,a)
#define W6(a,p,q,r6,Q8,S0) n2(a,p,d(q,Q8),S0)
#define Ng(U,f,a) E4(U,f,a)
#define I3
#define d1
#define q1(a,l) texelFetch(a,l,0)
#ifdef EC
#elif NC>=310
#else
#endif
#define B4
#define C4
#define Q3
#define R3
#ifdef LF
#define M5(f,w1,a) E4(d3,f,a)
#define I4(f,w1,a) Ng(d3,f,a)
#define N5(f,w1,a) f5(d3,f,a)
#define J0(a,A0) q1(a,Y((A0)&Fc,(A0)>>Ec))
#define O5(a,A0) q1(a,Y((A0)&Fc,(A0)>>Ec)).xy
#else
#ifdef GL_ARB_shader_storage_buffer_object
#extension GL_ARB_shader_storage_buffer_object:require
#endif
#define M5(f,w1,a) layout(std430,binding=f)readonly buffer w1{a1 c2[];}a
#define I4(f,w1,a) layout(std430,binding=f)readonly buffer w1{G c2[];}a
#define N5(f,w1,a) layout(std430,binding=f)readonly buffer w1{g c2[];}a
#define Ma(f,w1,a) layout(std430,binding=f)buffer w1{uint c2[];}a
#define J0(a,A0) a.c2[A0]
#define O5(a,A0) a.c2[A0]
#define Cd(a,A0) a.c2[A0]
#define B7(a,A0,q) atomicMax(a.c2[A0],q)
#define Na(a,A0,q) atomicAdd(a.c2[A0],q)
#define Og(a,A0,q) atomicOr(a.c2[A0],q)
#endif
#ifdef YD
#define L1(a) void main(){Y J=ivec2(floor(Z));int D0=int(K8(uvec2(J),(m.o6+(ta-1u))&~(ta-1u)));
#define Y1 }
#define S3 ,int D0
#define M1 ,D0
#ifdef ZD
#define E2(f,a) layout(std430,set=H3,binding=f)buffer a##Dd{uint c2[];}a
#elif defined(EC)
#define E2(f,a) layout(std430,set=H3,binding=f)coherent buffer a##Dd{uint c2[];}a
#else
#define E2(f,a) layout(std430,binding=f)coherent buffer a##Dd{uint c2[];}a
#endif
#define Oa E2
#define V2(h) h.c2[D0]
#define W2(h,D) h.c2[D0]=D
#define Pa(h) unpackUnorm4x8(V2(h))
#define Qa(h,D) W2(h,packUnorm4x8(D))
#define X4(h,q) atomicMax(h.c2[D0],q)
#define Y4(h,q) atomicAdd(h.c2[D0],q)
#elif defined(AE)||defined(MF)
#ifdef GL_ARB_shader_image_load_store
#extension GL_ARB_shader_image_load_store:require
#endif
#define L1(a) void main(){Y J=ivec2(floor(Z));
#define Y1 }
#define S3 ,Y J
#define M1 ,J
#ifdef EC
#define Oa(f,a) layout(set=H3,binding=f,rgba8)uniform mediump coherent image2D a
#define E2(f,a) layout(set=H3,binding=f,r32ui)uniform highp coherent uimage2D a
#define Ra(f,a) layout(set=H3,binding=f,rgb10_a2)uniform mediump coherent image2D a
#else
#define Oa(f,a) layout(binding=f,rgba8)uniform mediump coherent image2D a
#define E2(f,a) layout(binding=f,r32ui)uniform highp coherent uimage2D a
#define Ra(f,a) layout(binding=f,rgb10_a2)uniform mediump coherent image2D a;
#endif
#define V2(h) imageLoad(h,J).x
#define W2(h,D) imageStore(h,J,uvec4(D))
#define Pa(h) imageLoad(h,J)
#define Qa(h,D) imageStore(h,J,D)
#define X4(h,q) imageAtomicMax(h,J,q)
#define Y4(h,q) imageAtomicAdd(h,J,q)
#else
#define L1(a) void main()
#define Y1
#define S3
#define M1
#endif
#ifdef EXPORTED_PLS_IMPL_ANGLE
#extension GL_ANGLE_shader_pixel_local_storage:require
#define I1
#define w0(f,a) layout(binding=f,rgba8)uniform mediump pixelLocalANGLE a
#define j1(f,a) layout(binding=f,r32ui)uniform highp upixelLocalANGLE a
#define J1
#define I0(h) pixelLocalLoadANGLE(h)
#define Y0(h) pixelLocalLoadANGLE(h).x
#define x0(h,D) pixelLocalStoreANGLE(h,D)
#define c1(h,D) pixelLocalStoreANGLE(h,uvec4(D))
#define v2(h)
#define d2(h)
#define w2
#define x2
#endif
#ifdef NF
#ifdef Q
#extension GL_EXT_shader_pixel_local_storage2:require
#else
#extension GL_EXT_shader_pixel_local_storage:require
#endif
#define I1 __pixel_localEXT R1{
#define w0(f,a) layout(rgba8)mediump vec4 a
#define Sa(f,a) layout(rgb10_a2)mediump vec4 a
#define j1(f,a) layout(r32ui)highp uint a
#define J1 };
#define I0(h) h
#define Y0(h) h
#define x0(h,D) h=(D)
#define c1(h,D) h=(D)
#define v2(h) h=h
#define d2(h) h=h
#define w2
#define x2
#ifdef Q
#define o2(a) layout(location=0,rgba8)out i C1;L1(a)
#endif
#endif
#if defined(AE)||defined(YD)
#define I1
#define J1
#define w0 Oa
#define j1 E2
#define Sa Ra
#define I0 Pa
#define x0 Qa
#define Y0 V2
#define c1 W2
#define v2(h)
#define d2(h)
#if defined(GL_ARB_fragment_shader_interlock)
#extension GL_ARB_fragment_shader_interlock:require
#define w2 beginInvocationInterlockARB()
#define x2 endInvocationInterlockARB()
#elif defined(GL_INTEL_fragment_shader_ordering)
#extension GL_INTEL_fragment_shader_ordering:require
#define w2 beginFragmentShaderOrderingINTEL()
#define x2
#else
#define w2
#define x2
#endif
#endif
#ifdef OF
#define I1
#define q4(f,a) layout(input_attachment_index=f,binding=f,set=H3)uniform mediump subpassInput C7##a
#define Ed(f,a) layout(location=f)out mediump vec4 a
#define w0(f,a) q4(f,a);Ed(f,a)
#define j1(f,a) layout(input_attachment_index=f,binding=f,set=H3)uniform highp usubpassInput C7##a;layout(location=f)out highp uvec4 a
#define J1
#define I0(h) subpassLoad(C7##h)
#define Y0(h) subpassLoad(C7##h).x
#define x0(h,D) h=(D)
#define c1(h,D) h.x=(D)
#define v2(h) x0(h,subpassLoad(C7##h))
#define d2(h) c1(h,subpassLoad(C7##h).x)
#define w2
#define x2
#endif
#ifdef PF
#define I1
#define w0(f,a) layout(location=f)out mediump vec4 a
#define j1(f,a) layout(location=f)out highp uvec4 a
#define J1
#define I0(h) vec4(0)
#define Y0(h) 0u
#define x0(h,D) h=(D)
#define c1(h,D) h.x=(D)
#define v2(h) h=vec4(0)
#define d2(h) h.x=0u
#define w2
#define x2
#endif
#ifndef q4
#define q4 w0
#endif
#ifdef EC
#define gl_VertexID gl_VertexIndex
#endif
#ifdef PE
#ifdef EC
#define R8 gl_InstanceIndex
#else
#ifdef BE
uniform highp int BE;
#define R8 (gl_InstanceID+BE)
#else
#define R8 (gl_InstanceID+gl_BaseInstance)
#endif
#endif
#else
#define R8 0
#endif
#define k6
#define w3
#define d7
#define w5
#define y1(a,e0,F,B,v) void main(){int B=gl_VertexID;int v=R8;
#define R7(a,e0,F,n1,g0,B,v) y1(a,e0,F,B,v)
#define H6(a,i3,j3,y3,z3,n1,g0,B) y1(a,i3,j3,B,v)
#define V(a,d0)
#define a0(a)
#define r(a,d0)
#define z1(O0) gl_Position=O0;}
#define a3(S1,a) layout(location=0)out S1 Pg;void main()
#define v6 a3
#define w6 gl_FrontFacing
#define I2(D) Pg=D
#define Z gl_FragCoord.xy
#define J6
#define U2
#if defined(AE)||defined(YD)
#define Fd(D7,h,D) if(!(D7)){x0(h,D);}
#define Gd(D7,h,D) if(!(D7)){c1(h,D);}
#else
#define Fd(D7,h,D) x0(h,D);
#define Gd(D7,h,D) c1(h,D);
#endif
#ifndef o2
#define o2(a) layout(location=0)out i C1;L1(a)
#endif
#define n3 Y1
#if defined(EC)&&!defined(ZD)
#define i7(a) layout(input_attachment_index=0,binding=S2,set=H3)uniform mediump subpassInputMS a
#define S8(a) Bc(mat4(subpassLoad(a,0),subpassLoad(a,1),subpassLoad(a,2),subpassLoad(a,3)),gl_SampleMaskIn[0])
#else
#define i7(a) Z2(d3,Kf,a)
#define S8(a) texelFetch(a,ivec2(floor(Z.xy)),0)
#endif
#define R0(C,H) ((C)*(H))
precision highp float;precision highp int;
#if NC<310
e i Qg(uint u){G T1=G(u&0xffu,(u>>8)&0xffu,(u>>16)&0xffu,u>>24);return g(T1)*(1./255.);}
#define unpackUnorm4x8 Qg
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive