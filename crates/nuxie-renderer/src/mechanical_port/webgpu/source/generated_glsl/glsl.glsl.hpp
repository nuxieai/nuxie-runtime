#pragma once

#include "glsl.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char glsl[] = R"===(#define ya
#ifndef KC
#define KC __VERSION__
#endif
#define c vec2
#define O vec3
#define d4 vec3
#define e vec4
#define d mediump float
#define C mediump vec2
#define v mediump vec3
#define i mediump vec4
#define k7 mediump mat3x3
#define l7 mediump mat2x3
#define T4 mediump mat4x4
#define e0 ivec2
#define m6 ivec4
#define O0 uvec2
#define M uvec4
#define Q mediump uint
#define S4 bvec2
#define B6 bvec3
#define H7 bvec4
#define Y mat2
#define f
#define i1(w2) out w2
#define V6(w2) inout w2
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
#if defined(CB)&&defined(AB)&&defined(GL_ES)&&!defined(QE)
#ifdef GL_EXT_clip_cull_distance
#extension GL_EXT_clip_cull_distance:require
#elif defined(GL_ANGLE_clip_cull_distance)
#extension GL_ANGLE_clip_cull_distance:require
#endif
#endif
#if KC>=310
#define G7(g,a) layout(binding=g,std140) uniform a{
#else
#define G7(g,a) layout(std140) uniform a{
#endif
#define e9(a) }a;
#define c1(a)
#define K(g,j0,a) layout(location=g) in j0 a
#define d1
#define L(h9,D,a,j0)
#ifdef BB
#if KC>=310
#define V(g,j0,a) layout(location=g) out j0 a
#else
#define V(g,j0,a) out j0 a
#endif
#else
#if KC>=310
#define V(g,j0,a) layout(location=g) in j0 a
#else
#define V(g,j0,a) in j0 a
#endif
#endif
#define Z2 flat
#define l2
#define e2
#ifdef DC
#define E0
#else
#ifdef GL_NV_shader_noperspective_interpolation
#extension GL_NV_shader_noperspective_interpolation:require
#define E0 noperspective
#else
#define E0
#endif
#endif
#ifdef BB
#define k4
#define l4
#endif
#ifdef FB
#define O3
#define P3
#endif
#define x5
#define y5
#ifdef DC
#define W4(c0,g,a) layout(set=c0,binding=g) uniform highp utexture2D a
#define C6(c0,g,a) layout(set=c0,binding=g) uniform highp texture2D a
#define i3(c0,g,a) layout(set=c0,binding=g) uniform mediump texture2D a
#define D5(c0,g,a) layout(binding=g) uniform mediump texture2D a
#if defined(FB)&&defined(CB)
#endif
#elif KC>=310
#define W4(c0,g,a) layout(binding=g) uniform highp usampler2D a
#define C6(c0,g,a) layout(binding=g) uniform highp sampler2D a
#define i3(c0,g,a) layout(binding=g) uniform mediump sampler2D a
#define D5(c0,g,a) layout(binding=g) uniform mediump sampler2D a
#else
#define W4(c0,g,a) uniform highp usampler2D a
#define C6(c0,g,a) uniform highp sampler2D a
#define i3(c0,g,a) uniform mediump sampler2D a
#define D5(c0,g,a) uniform mediump sampler2D a
#endif
#ifdef DC
#define D6(c0,g,a) layout(set=c0,binding=g) uniform mediump sampler a;
#ifdef PF
#define p4(I7,a) layout(set=Tg,binding=I7) uniform mediump sampler a;
#define n4(a) D6(w5,Sg,a)
#else
#define p4(I7,a) layout(set=l3,binding=I7) uniform mediump sampler a;
#define n4(a) D6(w5,m4,a)
#endif
#define K5(a,o,l) texture(sampler2D(a,o),l)
#define o2(a,o,l,Y0) textureLod(sampler2D(a,o),l,Y0)
#define L5(a,o,l,Z1) texture(sampler2D(a,o),l,Z1)
#if defined(FB)&&defined(CB)&&defined(RE)
#extension GL_OES_sample_variables:require
#endif
#else
#define p4(I7,a)
#define D6(c0,g,a)
#define n4(a)
#define K5(a,o,l) texture(a,l)
#define o2(a,o,l,Y0) textureLod(a,l,Y0)
#define L5(a,o,l,Z1) texture(a,l,Z1)
#endif
#define v8(q0,o,l) K5(q0,o,l)
#define i6(q0,o,l,Y0) o2(q0,o,l,Y0)
#define J7(q0,o,l,Z1) L5(q0,o,l,Z1)
#define q6(c0,g,a) D5(c0,g,a)
#define j7(a,o,F,E6,j9,Y0) o2(a,o,c(F,j9),Y0)
#define Sh(c0,g,a) W4(c0,g,a)
#define S3
#define k1
#define p1(a,l) texelFetch(a,l,0)
#ifdef DC
#elif KC>=310
#else
#endif
#define Q4
#define R4
#define g4
#define h4
#ifdef QF
#define Z5(g,F1,a) W4(l3,g,a)
#define X4(g,F1,a) Sh(l3,g,a)
#define a6(g,F1,a) C6(l3,g,a)
#define p0(a,D0) p1(a,e0((D0)&pd,(D0)>>od))
#define l5(a,D0) p1(a,e0((D0)&pd,(D0)>>od)).xy
#else
#ifdef GL_ARB_shader_storage_buffer_object
#extension GL_ARB_shader_storage_buffer_object:require
#endif
#define Z5(g,F1,a) layout(std430,binding=g) readonly buffer F1{O0 k2[];}a
#define X4(g,F1,a) layout(std430,binding=g) readonly buffer F1{M k2[];}a
#define a6(g,F1,a) layout(std430,binding=g) readonly buffer F1{e k2[];}a
#define nb(g,F1,a) layout(std430,binding=g) buffer F1{uint k2[];}a
#define p0(a,D0) a.k2[D0]
#define l5(a,D0) a.k2[D0]
#define ge(a,D0) a.k2[D0]
#define L7(a,D0,F) atomicMax(a.k2[D0],F)
#define ob(a,D0,F) atomicAdd(a.k2[D0],F)
#define Th(a,D0,F) atomicOr(a.k2[D0],F)
#endif
#ifdef ID
#define U1(a) void main(){e0 H=ivec2(floor(f0));int K0=int(d9(uvec2(H),(j.A6+(Ua-1u))&~(Ua-1u)));
#define h2 }
#define i4 ,int K0
#define V1 ,K0
#ifdef SE
#define L2(g,a) layout(std430,set=D3,binding=g) buffer a##he{uint k2[];}a
#elif defined(DC)
#define L2(g,a) layout(std430,set=D3,binding=g) coherent buffer a##he{uint k2[];}a
#else
#define L2(g,a) layout(std430,binding=g) coherent buffer a##he{uint k2[];}a
#endif
#define pb L2
#define f3(h) h.k2[K0]
#define g3(h,E) h.k2[K0]=E
#define qb(h) unpackUnorm4x8(f3(h))
#define rb(h,E) g3(h,packUnorm4x8(E))
#define p5(h,F) atomicMax(h.k2[K0],F)
#define q5(h,F) atomicAdd(h.k2[K0],F)
#elif defined(BE)||defined(RF)
#ifdef GL_ARB_shader_image_load_store
#extension GL_ARB_shader_image_load_store:require
#endif
#define U1(a) void main(){e0 H=ivec2(floor(f0));
#define h2 }
#define i4 ,e0 H
#define V1 ,H
#ifdef DC
#define pb(g,a) layout(set=D3,binding=g,rgba8) uniform mediump coherent image2D a
#define L2(g,a) layout(set=D3,binding=g,r32ui) uniform highp coherent uimage2D a
#define sb(g,a) layout(set=D3,binding=g,rgb10_a2) uniform mediump coherent image2D a
#else
#define pb(g,a) layout(binding=g,rgba8) uniform mediump coherent image2D a
#define L2(g,a) layout(binding=g,r32ui) uniform highp coherent uimage2D a
#define sb(g,a) layout(binding=g,rgb10_a2) uniform mediump coherent image2D a;
#endif
#define f3(h) imageLoad(h,H).x
#define g3(h,E) imageStore(h,H,uvec4(E))
#define qb(h) imageLoad(h,H)
#define rb(h,E) imageStore(h,H,E)
#define p5(h,F) imageAtomicMax(h,H,F)
#define q5(h,F) imageAtomicAdd(h,H,F)
#else
#define U1(a) void main()
#define h2
#define i4
#define V1
#endif
#ifdef EXPORTED_PLS_IMPL_ANGLE
#extension GL_ANGLE_shader_pixel_local_storage:require
#define S1
#define B0(g,a) layout(binding=g,rgba8) uniform mediump pixelLocalANGLE a
#define o1(g,a) layout(binding=g,r32ui) uniform highp upixelLocalANGLE a
#define T1
#define N0(h) pixelLocalLoadANGLE(h)
#define h1(h) pixelLocalLoadANGLE(h).x
#define y0(h,E) pixelLocalStoreANGLE(h,E)
#define j1(h,E) pixelLocalStoreANGLE(h,uvec4(E))
#define D2(h)
#define a2(h)
#define E2
#define F2
#endif
#ifdef SF
#ifdef W
#extension GL_EXT_shader_pixel_local_storage2:require
#else
#extension GL_EXT_shader_pixel_local_storage:require
#endif
#define S1 __pixel_localEXT c2{
#define B0(g,a) layout(rgba8) mediump vec4 a
#define tb(g,a) layout(rgb10_a2) mediump vec4 a
#define o1(g,a) layout(r32ui) highp uint a
#define T1 };
#define N0(h) h
#define h1(h) h
#define y0(h,E) h=(E)
#define j1(h,E) h=(E)
#define D2(h) h=h
#define a2(h) h=h
#define E2
#define F2
#ifdef W
#define z2(a) layout(location=0,rgba8) out i L1;U1(a)
#endif
#endif
#if defined(BE)||defined(ID)
#define S1
#define T1
#define B0 pb
#define o1 L2
#define tb sb
#define N0 qb
#define y0 rb
#define h1 f3
#define j1 g3
#define D2(h)
#define a2(h)
#if defined(GL_ARB_fragment_shader_interlock)
#extension GL_ARB_fragment_shader_interlock:require
#define E2 beginInvocationInterlockARB()
#define F2 endInvocationInterlockARB()
#elif defined(GL_INTEL_fragment_shader_ordering)
#extension GL_INTEL_fragment_shader_ordering:require
#define E2 beginFragmentShaderOrderingINTEL()
#define F2
#else
#define E2
#define F2
#endif
#endif
#ifdef TF
#define S1
#define J4(g,a) layout(input_attachment_index=g,binding=g,set=D3) uniform mediump subpassInput M7##a
#define ie(g,a) layout(location=g) out mediump vec4 a
#define B0(g,a) J4(g,a);ie(g,a)
#define o1(g,a) layout(input_attachment_index=g,binding=g,set=D3) uniform highp usubpassInput M7##a;layout(location=g) out highp uvec4 a
#define T1
#define N0(h) subpassLoad(M7##h)
#define h1(h) subpassLoad(M7##h).x
#define y0(h,E) h=(E)
#define j1(h,E) h.x=(E)
#define D2(h) y0(h,subpassLoad(M7##h))
#define a2(h) j1(h,subpassLoad(M7##h).x)
#define E2
#define F2
#endif
#ifdef UF
#define S1
#define B0(g,a) layout(location=g) out mediump vec4 a
#define o1(g,a) layout(location=g) out highp uvec4 a
#define T1
#define N0(h) vec4(0)
#define h1(h) 0u
#define y0(h,E) h=(E)
#define j1(h,E) h.x=(E)
#define D2(h) h=vec4(0)
#define a2(h) h.x=0u
#define E2
#define F2
#endif
#ifndef J4
#define J4 B0
#endif
#ifdef DC
#define ub gl_VertexIndex
#ifdef CE
#define N7 gl_InstanceIndex
#else
#define N7 0
#endif
#else
#ifdef TE
uniform highp int UE;
#define ub (gl_VertexID+UE)
#else
#define ub gl_VertexID
#endif
#ifdef CE
#ifdef DE
uniform highp int DE;
#define N7 (gl_InstanceID+DE)
#else
#define N7 (gl_InstanceID+gl_BaseInstance)
#endif
#else
#define N7 0
#endif
#endif
#define w6
#define H3
#define p7
#define Z4
#define w1(a,d0,D,p3,F6) void main(){int p3=ub;int F6=N7;
#define g8(a,d0,D,B1,h0,p3,F6) w1(a,d0,D,p3,F6)
#define T6(a,x3,y3,K3,L3,B1,h0,p3) w1(a,x3,y3,p3,F6)
#define T(a,j0)
#define Z(a)
#define q(a,j0)
#define x1(U0) gl_Position=U0;}
#define j3(G1,a) layout(location=0) out G1 Uh;void main()
#define G6(G1,a) j3(G1,a)
#define H6 gl_FrontFacing
#define P2(E) Uh=E
#define f0 gl_FragCoord.xy
#define W6
#define e3
#if defined(BE)||defined(ID)
#define je(O7,h,E) if(!(O7)){y0(h,E);}
#define ke(O7,h,E) if(!(O7)){j1(h,E);}
#else
#define je(O7,h,E) y0(h,E);
#define ke(O7,h,E) j1(h,E);
#endif
#ifndef z2
#define z2(a) layout(location=0) out i L1;U1(a)
#endif
#define A3 h2
#if defined(DC)&&!defined(SE)
#ifdef RE
#define E5(a) layout(input_attachment_index=0,binding=K2,set=D3) uniform mediump subpassInputMS a
#define I6(a) Ia(mat4(subpassLoad(a,0),subpassLoad(a,1),subpassLoad(a,2),subpassLoad(a,3)),gl_SampleMaskIn[0])
#else
#define E5(a) layout(input_attachment_index=0,binding=K2,set=D3) uniform mediump subpassInput a
#define I6(a) subpassLoad(a)
#endif
#else
#define E5(a) i3(l3,Rg,a)
#define I6(a) texelFetch(a,ivec2(floor(f0.xy)),0)
#endif
#define M0(B,J) ((B)*(J))
precision highp float;precision highp int;
#if KC<310
f i Vh(uint u){M q1=M(u&0xffu,(u>>8)&0xffu,(u>>16)&0xffu,u>>24);return e(q1)*(1./255.);}
#define unpackUnorm4x8 Vh
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive