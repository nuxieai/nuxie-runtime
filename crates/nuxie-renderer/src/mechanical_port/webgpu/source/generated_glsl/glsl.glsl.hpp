#pragma once

#include "glsl.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char glsl[] = R"===(#define mc
#ifndef MC
#define MC __VERSION__
#endif
#define d vec2
#define c0 vec3
#define L3 vec3
#define g vec4
#define c mediump float
#define E mediump vec2
#define v mediump vec3
#define i mediump vec4
#define V6 mediump mat3x3
#define W6 mediump mat2x3
#define h5 mediump mat4x4
#define X ivec2
#define a6 ivec4
#define a1 uvec2
#define G uvec4
#define N mediump uint
#define D4 bvec2
#define n6 bvec3
#define v7 bvec4
#define g0 mat2
#define e
#define Z0(j2) out j2
#define T4(j2) inout j2
#ifdef GL_ANGLE_base_vertex_base_instance_shader_builtin
#extension GL_ANGLE_base_vertex_base_instance_shader_builtin:require
#endif
#ifdef GE
#extension GL_KHR_blend_equation_advanced:require
#endif
#ifdef TD
#extension GL_EXT_shader_framebuffer_fetch:require
#elif defined(UD)
#extension GL_EXT_shader_pixel_local_storage:require
#elif defined(EXPORTED_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
#extension GL_ANGLE_shader_pixel_local_storage:require
#elif defined(VD)
#ifdef GL_ARB_shader_image_load_store
#extension GL_ARB_shader_image_load_store:require
#endif
#ifdef GL_OES_shader_image_atomic
#extension GL_OES_shader_image_atomic:require
#endif
#endif
#if defined(CB)&&defined(BB)&&defined(GL_ES)&&!defined(LE)
#ifdef GL_EXT_clip_cull_distance
#extension GL_EXT_clip_cull_distance:require
#elif defined(GL_ANGLE_clip_cull_distance)
#extension GL_ANGLE_clip_cull_distance:require
#endif
#endif
#if MC>=310
#define r7(f,a) layout(binding=f,std140)uniform a{
#else
#define r7(f,a) layout(std140)uniform a{
#endif
#define J8(a) }a;
#define g1(a)
#define L(f,d0,a) layout(location=f)in d0 a
#define h1
#define M(M8,F,a,d0)
#ifdef DB
#if MC>=310
#define W(f,d0,a) layout(location=f)out d0 a
#else
#define W(f,d0,a) out d0 a
#endif
#else
#if MC>=310
#define W(f,d0,a) layout(location=f)in d0 a
#else
#define W(f,d0,a) in d0 a
#endif
#endif
#define O2 flat
#define k2
#define f2
#ifdef DC
#define J0
#else
#ifdef GL_NV_shader_noperspective_interpolation
#extension GL_NV_shader_noperspective_interpolation:require
#define J0 noperspective
#else
#define J0
#endif
#endif
#ifdef DB
#define S3
#define T3
#endif
#ifdef GB
#define C3
#define D3
#endif
#define a5
#define c5
#ifdef DC
#define C4(T,f,a) layout(set=T,binding=f)uniform highp utexture2D a
#define e5(T,f,a) layout(set=T,binding=f)uniform highp texture2D a
#define X2(T,f,a) layout(set=T,binding=f)uniform mediump texture2D a
#define k5(T,f,a) layout(binding=f)uniform mediump texture2D a
#if defined(GB)&&defined(CB)
#endif
#elif MC>=310
#define C4(T,f,a) layout(binding=f)uniform highp usampler2D a
#define e5(T,f,a) layout(binding=f)uniform highp sampler2D a
#define X2(T,f,a) layout(binding=f)uniform mediump sampler2D a
#define k5(T,f,a) layout(binding=f)uniform mediump sampler2D a
#else
#define C4(T,f,a) uniform highp usampler2D a
#define e5(T,f,a) uniform highp sampler2D a
#define X2(T,f,a) uniform mediump sampler2D a
#define k5(T,f,a) uniform mediump sampler2D a
#endif
#ifdef DC
#define o6(T,f,a) layout(set=T,binding=f)uniform mediump sampler a;
#ifdef IF
#define Z3(w7,a) layout(set=If,binding=w7)uniform mediump sampler a;
#define V3(a) o6(Z4,Hf,a)
#else
#define Z3(w7,a) layout(set=a3,binding=w7)uniform mediump sampler a;
#define V3(a) o6(Z4,U3,a)
#endif
#define q5(a,p,l) texture(sampler2D(a,p),l)
#define n2(a,p,l,R0) textureLod(sampler2D(a,p),l,R0)
#define r5(a,p,l,Q1) texture(sampler2D(a,p),l,Q1)
#if defined(GB)&&defined(CB)
#extension GL_OES_sample_variables:require
#endif
#else
#define Z3(w7,a)
#define o6(T,f,a)
#define V3(a)
#define q5(a,p,l) texture(a,l)
#define n2(a,p,l,R0) textureLod(a,l,R0)
#define r5(a,p,l,Q1) texture(a,l,Q1)
#endif
#define d8(k0,p,l) q5(k0,p,l)
#define Q6(k0,p,l,R0) n2(k0,p,l,R0)
#define x7(k0,p,l,Q1) r5(k0,p,l,Q1)
#define f6(T,f,a) k5(T,f,a)
#define U6(a,p,q,p6,O8,R0) n2(a,p,d(q,O8),R0)
#define Hg(T,f,a) C4(T,f,a)
#define G3
#define d1
#define q1(a,l) texelFetch(a,l,0)
#ifdef DC
#elif MC>=310
#else
#endif
#define z4
#define A4
#define O3
#define P3
#ifdef JF
#define K5(f,w1,a) C4(a3,f,a)
#define G4(f,w1,a) Hg(a3,f,a)
#define L5(f,w1,a) e5(a3,f,a)
#define N0(a,A0) q1(a,X((A0)&Dc,(A0)>>Cc))
#define M5(a,A0) q1(a,X((A0)&Dc,(A0)>>Cc)).xy
#else
#ifdef GL_ARB_shader_storage_buffer_object
#extension GL_ARB_shader_storage_buffer_object:require
#endif
#define K5(f,w1,a) layout(std430,binding=f)readonly buffer w1{a1 c2[];}a
#define G4(f,w1,a) layout(std430,binding=f)readonly buffer w1{G c2[];}a
#define L5(f,w1,a) layout(std430,binding=f)readonly buffer w1{g c2[];}a
#define Ma(f,w1,a) layout(std430,binding=f)buffer w1{uint c2[];}a
#define N0(a,A0) a.c2[A0]
#define M5(a,A0) a.c2[A0]
#define Ad(a,A0) a.c2[A0]
#define z7(a,A0,q) atomicMax(a.c2[A0],q)
#define Na(a,A0,q) atomicAdd(a.c2[A0],q)
#define Ig(a,A0,q) atomicOr(a.c2[A0],q)
#endif
#ifdef WD
#define L1(a) void main(){X J=ivec2(floor(Y));int D0=int(I8(uvec2(J),(n.m6+(ta-1u))&~(ta-1u)));
#define Y1 }
#define Q3 ,int D0
#define M1 ,D0
#ifdef ME
#define D2(f,a) layout(std430,set=F3,binding=f)buffer a##Bd{uint c2[];}a
#elif defined(DC)
#define D2(f,a) layout(std430,set=F3,binding=f)coherent buffer a##Bd{uint c2[];}a
#else
#define D2(f,a) layout(std430,binding=f)coherent buffer a##Bd{uint c2[];}a
#endif
#define Oa D2
#define T2(h) h.c2[D0]
#define U2(h,D) h.c2[D0]=D
#define Pa(h) unpackUnorm4x8(T2(h))
#define Qa(h,D) U2(h,packUnorm4x8(D))
#define W4(h,q) atomicMax(h.c2[D0],q)
#define X4(h,q) atomicAdd(h.c2[D0],q)
#elif defined(XD)||defined(KF)
#ifdef GL_ARB_shader_image_load_store
#extension GL_ARB_shader_image_load_store:require
#endif
#define L1(a) void main(){X J=ivec2(floor(Y));
#define Y1 }
#define Q3 ,X J
#define M1 ,J
#ifdef DC
#define Oa(f,a) layout(set=F3,binding=f,rgba8)uniform mediump coherent image2D a
#define D2(f,a) layout(set=F3,binding=f,r32ui)uniform highp coherent uimage2D a
#define Ra(f,a) layout(set=F3,binding=f,rgb10_a2)uniform mediump coherent image2D a
#else
#define Oa(f,a) layout(binding=f,rgba8)uniform mediump coherent image2D a
#define D2(f,a) layout(binding=f,r32ui)uniform highp coherent uimage2D a
#define Ra(f,a) layout(binding=f,rgb10_a2)uniform mediump coherent image2D a;
#endif
#define T2(h) imageLoad(h,J).x
#define U2(h,D) imageStore(h,J,uvec4(D))
#define Pa(h) imageLoad(h,J)
#define Qa(h,D) imageStore(h,J,D)
#define W4(h,q) imageAtomicMax(h,J,q)
#define X4(h,q) imageAtomicAdd(h,J,q)
#else
#define L1(a) void main()
#define Y1
#define Q3
#define M1
#endif
#ifdef EXPORTED_PLS_IMPL_ANGLE
#extension GL_ANGLE_shader_pixel_local_storage:require
#define I1
#define w0(f,a) layout(binding=f,rgba8)uniform mediump pixelLocalANGLE a
#define j1(f,a) layout(binding=f,r32ui)uniform highp upixelLocalANGLE a
#define J1
#define H0(h) pixelLocalLoadANGLE(h)
#define Y0(h) pixelLocalLoadANGLE(h).x
#define x0(h,D) pixelLocalStoreANGLE(h,D)
#define c1(h,D) pixelLocalStoreANGLE(h,uvec4(D))
#define v2(h)
#define d2(h)
#define w2
#define x2
#endif
#ifdef LF
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
#define H0(h) h
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
#if defined(XD)||defined(WD)
#define I1
#define J1
#define w0 Oa
#define j1 D2
#define Sa Ra
#define H0 Pa
#define x0 Qa
#define Y0 T2
#define c1 U2
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
#ifdef MF
#define I1
#define o4(f,a) layout(input_attachment_index=f,binding=f,set=F3)uniform mediump subpassInput A7##a
#define Cd(f,a) layout(location=f)out mediump vec4 a
#define w0(f,a) o4(f,a);Cd(f,a)
#define j1(f,a) layout(input_attachment_index=f,binding=f,set=F3)uniform highp usubpassInput A7##a;layout(location=f)out highp uvec4 a
#define J1
#define H0(h) subpassLoad(A7##h)
#define Y0(h) subpassLoad(A7##h).x
#define x0(h,D) h=(D)
#define c1(h,D) h.x=(D)
#define v2(h) x0(h,subpassLoad(A7##h))
#define d2(h) c1(h,subpassLoad(A7##h).x)
#define w2
#define x2
#endif
#ifdef NF
#define I1
#define w0(f,a) layout(location=f)out mediump vec4 a
#define j1(f,a) layout(location=f)out highp uvec4 a
#define J1
#define H0(h) vec4(0)
#define Y0(h) 0u
#define x0(h,D) h=(D)
#define c1(h,D) h.x=(D)
#define v2(h) h=vec4(0)
#define d2(h) h.x=0u
#define w2
#define x2
#endif
#ifndef o4
#define o4 w0
#endif
#ifdef DC
#define gl_VertexID gl_VertexIndex
#endif
#ifdef NE
#ifdef DC
#define P8 gl_InstanceIndex
#else
#ifdef YD
uniform highp int YD;
#define P8 (gl_InstanceID+YD)
#else
#define P8 (gl_InstanceID+gl_BaseInstance)
#endif
#endif
#else
#define P8 0
#endif
#define i6
#define v3
#define a7
#define v5
#define y1(a,e0,F,B,r) void main(){int B=gl_VertexID;int r=P8;
#define P7(a,e0,F,n1,f0,B,r) y1(a,e0,F,B,r)
#define F6(a,g3,h3,x3,y3,n1,f0,B) y1(a,g3,h3,B,r)
#define V(a,d0)
#define a0(a)
#define A(a,d0)
#define z1(O0) gl_Position=O0;}
#define Y2(S1,a) layout(location=0)out S1 Jg;void main()
#define q6 Y2
#define r6 gl_FrontFacing
#define G2(D) Jg=D
#define Y gl_FragCoord.xy
#define H6
#define S2
#if defined(XD)||defined(WD)
#define Dd(B7,h,D) if(!(B7)){x0(h,D);}
#define Ed(B7,h,D) if(!(B7)){c1(h,D);}
#else
#define Dd(B7,h,D) x0(h,D);
#define Ed(B7,h,D) c1(h,D);
#endif
#ifndef o2
#define o2(a) layout(location=0)out i C1;L1(a)
#endif
#define l3 Y1
#if defined(DC)&&!defined(ME)
#define g7(a) layout(input_attachment_index=0,binding=Q2,set=F3)uniform mediump subpassInputMS a
#define Q8(a) zc(mat4(subpassLoad(a,0),subpassLoad(a,1),subpassLoad(a,2),subpassLoad(a,3)),gl_SampleMaskIn[0])
#else
#define g7(a) X2(a3,Gf,a)
#define Q8(a) texelFetch(a,ivec2(floor(Y.xy)),0)
#endif
#define U0(C,H) ((C)*(H))
precision highp float;precision highp int;
#if MC<310
e i Kg(uint u){G T1=G(u&0xffu,(u>>8)&0xffu,(u>>16)&0xffu,u>>24);return g(T1)*(1./255.);}
#define unpackUnorm4x8 Kg
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive