#define jb
#ifndef MC
#define MC __VERSION__
#endif
#define c vec2
#define M vec3
#define l4 vec3
#define e vec4
#define d mediump float
#define F mediump vec2
#define v mediump vec3
#define i mediump vec4
#define D7 mediump mat3x3
#define E7 mediump mat2x3
#define m5 mediump mat4x4
#define f0 ivec2
#define B6 ivec4
#define R0 uvec2
#define R uvec4
#define O mediump uint
#define l5 bvec2
#define V6 bvec3
#define h8 bvec4
#define X mat2
#define f
#define n1(H2) out H2
#define m7(H2) inout H2
#ifdef GL_ANGLE_base_vertex_base_instance_shader_builtin
#extension GL_ANGLE_base_vertex_base_instance_shader_builtin:require
#endif
#ifdef RE
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
#if defined(CB)&&defined(AB)&&defined(GL_ES)&&!defined(VE)
#ifdef GL_EXT_clip_cull_distance
#extension GL_EXT_clip_cull_distance:require
#elif defined(GL_ANGLE_clip_cull_distance)
#extension GL_ANGLE_clip_cull_distance:require
#endif
#endif
#if MC>=310
#define g8(g,a) layout(binding=g,std140) uniform a{
#else
#define g8(g,a) layout(std140) uniform a{
#endif
#define R9(a) }a;
#define f1(a)
#define K(g,k0,a) layout(location=g) in k0 a
#define g1
#define L(U9,A,a,k0)
#ifdef BB
#if MC>=310
#define W(g,k0,a) layout(location=g) out k0 a
#else
#define W(g,k0,a) out k0 a
#endif
#else
#if MC>=310
#define W(g,k0,a) layout(location=g) in k0 a
#else
#define W(g,k0,a) in k0 a
#endif
#endif
#define m3 flat
#define n2
#define P1
#ifdef EC
#define I0
#else
#ifdef GL_NV_shader_noperspective_interpolation
#extension GL_NV_shader_noperspective_interpolation:require
#define I0 noperspective
#else
#define I0
#endif
#endif
#ifdef BB
#define A4
#define B4
#endif
#ifdef EB
#define Y3
#define Z3
#endif
#define J5
#define K5
#ifdef EC
#define r5(h0,g,a) layout(set=h0,binding=g) uniform highp utexture2D a
#define W6(h0,g,a) layout(set=h0,binding=g) uniform highp texture2D a
#define Y2(h0,g,a) layout(set=h0,binding=g) uniform mediump texture2D a
#define S5(h0,g,a) layout(binding=g) uniform mediump texture2D a
#if defined(EB)&&defined(CB)
#endif
#elif MC>=310
#define r5(h0,g,a) layout(binding=g) uniform highp usampler2D a
#define W6(h0,g,a) layout(binding=g) uniform highp sampler2D a
#define Y2(h0,g,a) layout(binding=g) uniform mediump sampler2D a
#define S5(h0,g,a) layout(binding=g) uniform mediump sampler2D a
#else
#define r5(h0,g,a) uniform highp usampler2D a
#define W6(h0,g,a) uniform highp sampler2D a
#define Y2(h0,g,a) uniform mediump sampler2D a
#define S5(h0,g,a) uniform mediump sampler2D a
#endif
#ifdef EC
#define X6(h0,g,a) layout(set=h0,binding=g) uniform mediump sampler a;
#ifdef VF
#define f4(W5,a) layout(set=Uh,binding=W5) uniform mediump sampler a;
#define N6 f4
#define D4(a) X6(I5,Th,a)
#else
#define f4(W5,a) layout(set=c3,binding=W5) uniform mediump sampler a;
#define N6 f4
#define D4(a) X6(I5,C4,a)
#endif
#define X5(a,p,o) texture(sampler2D(a,p),o)
#define q2(a,p,o,e1) textureLod(sampler2D(a,p),o,e1)
#define Y5(a,p,o,j2) texture(sampler2D(a,p),o,j2)
#if defined(EB)&&defined(CB)&&defined(WE)
#extension GL_OES_sample_variables:require
#endif
#else
#define f4(W5,a)
#define N6(W5,a)
#define X6(h0,g,a)
#define D4(a)
#define X5(a,p,o) texture(a,o)
#define q2(a,p,o,e1) textureLod(a,o,e1)
#define Y5(a,p,o,j2) texture(a,o,j2)
#endif
#define c9(r0,p,o) X5(r0,p,o)
#define j5(r0,p,o,e1) q2(r0,p,o,e1)
#define i8(r0,p,o,j2) Y5(r0,p,o,j2)
#define M6(h0,g,a) S5(h0,g,a)
#define C7(a,p,G,Y6,W9,e1) q2(a,p,c(G,W9),e1)
#define Wi(h0,g,a) r5(h0,g,a)
#define g4
#define p1
#define q1(a,o) texelFetch(a,o,0)
#ifdef EC
#elif MC>=310
#else
#endif
#define h5
#define i5
#define n4
#define p4
#ifdef WF
#define o4(g,O1,a) Wi(c3,g,a)
#define n6(g,O1,a) W6(c3,g,a)
#define o0(a,H0) q1(a,f0((H0)&ie,(H0)>>he))
#else
#ifdef GL_ARB_shader_storage_buffer_object
#extension GL_ARB_shader_storage_buffer_object:require
#endif
#define o4(g,O1,a) layout(std430,binding=g) readonly buffer O1{R z2[];}a
#define n6(g,O1,a) layout(std430,binding=g) readonly buffer O1{e z2[];}a
#define lc(g,O1,a) layout(std430,binding=g) buffer O1{uint z2[];}a
#define o0(a,H0) a.z2[H0]
#define gf(a,H0) a.z2[H0]
#define k8(a,H0,G) atomicMax(a.z2[H0],G)
#define mc(a,H0,G) atomicAdd(a.z2[H0],G)
#define Xi(a,H0,G) atomicOr(a.z2[H0],G)
#endif
#ifdef MD
#define D1(a) void main(){f0 D=ivec2(floor(Y));int D0=int(Q9(uvec2(D),(j.V5+(Kb-1u))&~(Kb-1u)));
#define K1 }
#define r4 ,int D0
#define d2 ,D0
#ifdef XE
#define X2(g,a) layout(std430,set=O3,binding=g) buffer a##hf{uint z2[];}a
#elif defined(EC)
#define X2(g,a) layout(std430,set=O3,binding=g) coherent buffer a##hf{uint z2[];}a
#else
#define X2(g,a) layout(std430,binding=g) coherent buffer a##hf{uint z2[];}a
#endif
#define K4 X2
#define v3(h) h.z2[D0]
#define w3(h,B) h.z2[D0]=B
#define e4(h) unpackUnorm4x8(v3(h))
#define q5(h,B) w3(h,packUnorm4x8(B))
#define G5(h,G) atomicMax(h.z2[D0],G)
#define H5(h,G) atomicAdd(h.z2[D0],G)
#elif defined(GE)||defined(XF)
#ifdef GL_ARB_shader_image_load_store
#extension GL_ARB_shader_image_load_store:require
#endif
#define D1(a) void main(){f0 D=ivec2(floor(Y));
#define K1 }
#define r4 ,f0 D
#define d2 ,D
#ifdef EC
#define K4(g,a) layout(set=O3,binding=g,rgba8) uniform mediump coherent image2D a
#define X2(g,a) layout(set=O3,binding=g,r32ui) uniform highp coherent uimage2D a
#define nc(g,a) layout(set=O3,binding=g,rgb10_a2) uniform mediump coherent image2D a
#else
#define K4(g,a) layout(binding=g,rgba8) uniform mediump coherent image2D a
#define X2(g,a) layout(binding=g,r32ui) uniform highp coherent uimage2D a
#define nc(g,a) layout(binding=g,rgb10_a2) uniform mediump coherent image2D a;
#endif
#define v3(h) imageLoad(h,D).x
#define w3(h,B) imageStore(h,D,uvec4(B))
#define e4(h) imageLoad(h,D)
#define q5(h,B) imageStore(h,D,B)
#define G5(h,G) imageAtomicMax(h,D,G)
#define H5(h,G) imageAtomicAdd(h,D,G)
#else
#define D1(a) void main()
#define K1
#define r4
#define d2
#endif
#ifdef EXPORTED_PLS_IMPL_ANGLE
#extension GL_ANGLE_shader_pixel_local_storage:require
#define H1
#define y0(g,a) layout(binding=g,rgba8) uniform mediump pixelLocalANGLE a
#define w1(g,a) layout(binding=g,r32ui) uniform highp upixelLocalANGLE a
#define I1
#define L0(h) pixelLocalLoadANGLE(h)
#define m1(h) pixelLocalLoadANGLE(h).x
#define x0(h,B) pixelLocalStoreANGLE(h,B)
#define o1(h,B) pixelLocalStoreANGLE(h,uvec4(B))
#define Q2(h)
#define k2(h)
#define R2
#define S2
#endif
#ifdef YF
#ifdef Q
#extension GL_EXT_shader_pixel_local_storage2:require
#else
#extension GL_EXT_shader_pixel_local_storage:require
#endif
#define H1 __pixel_localEXT V1{
#define y0(g,a) layout(rgba8) mediump vec4 a
#define oc(g,a) layout(rgb10_a2) mediump vec4 a
#define w1(g,a) layout(r32ui) highp uint a
#define I1 };
#define L0(h) h
#define m1(h) h
#define x0(h,B) h=(B)
#define o1(h,B) h=(B)
#define Q2(h) h=h
#define k2(h) h=h
#define R2
#define S2
#ifdef Q
#define K2(a) layout(location=0,rgba8) out i S1;D1(a)
#endif
#endif
#if defined(GE)||defined(MD)
#define H1
#define I1
#define y0 K4
#define w1 X2
#define oc nc
#define L0 e4
#define x0 q5
#define m1 v3
#define o1 w3
#define Q2(h)
#define k2(h)
#if defined(GL_ARB_fragment_shader_interlock)
#extension GL_ARB_fragment_shader_interlock:require
#define R2 beginInvocationInterlockARB()
#define S2 endInvocationInterlockARB()
#elif defined(GL_INTEL_fragment_shader_ordering)
#extension GL_INTEL_fragment_shader_ordering:require
#define R2 beginFragmentShaderOrderingINTEL()
#define S2
#else
#define R2
#define S2
#endif
#endif
#ifdef ZF
#define H1
#define a5(g,a) layout(input_attachment_index=g,binding=g,set=O3) uniform mediump subpassInput l8##a
#define jf(g,a) layout(location=g) out mediump vec4 a
#define y0(g,a) a5(g,a);jf(g,a)
#define w1(g,a) layout(input_attachment_index=g,binding=g,set=O3) uniform highp usubpassInput l8##a;layout(location=g) out highp uvec4 a
#define I1
#define L0(h) subpassLoad(l8##h)
#define m1(h) subpassLoad(l8##h).x
#define x0(h,B) h=(B)
#define o1(h,B) h.x=(B)
#define Q2(h) x0(h,subpassLoad(l8##h))
#define k2(h) o1(h,subpassLoad(l8##h).x)
#define R2
#define S2
#endif
#ifdef AG
#define H1
#define y0(g,a) layout(location=g) out mediump vec4 a
#define w1(g,a) layout(location=g) out highp uvec4 a
#define I1
#define L0(h) vec4(0)
#define m1(h) 0u
#define x0(h,B) h=(B)
#define o1(h,B) h.x=(B)
#define Q2(h) h=vec4(0)
#define k2(h) h.x=0u
#define R2
#define S2
#endif
#ifndef a5
#define a5 y0
#endif
#ifdef EC
#define pc gl_VertexIndex
#ifdef HE
#define m8 gl_InstanceIndex
#else
#define m8 0
#endif
#else
#ifdef ND
uniform highp int YE;
#define pc (gl_VertexID+YE)
#else
#define pc gl_VertexID
#endif
#ifdef HE
#ifdef IE
uniform highp int IE;
#define m8 (gl_InstanceID+IE)
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
#define H4
#define l1(a,d0,A,O2,F6) void main(){int O2=pc;int F6=m8;
#define K8(a,d0,A,G1,j0,O2,F6) l1(a,d0,A,O2,F6)
#define k7(a,H3,I3,W3,o3,G1,j0,O2) l1(a,H3,I3,O2,F6)
#define V(a,k0)
#define e0(a)
#define q(a,k0)
#define B1(T0) gl_Position=T0;}
#define r2(U0,a) layout(location=0) out U0 Yi;void main()
#define Z6(U0,a) r2(U0,a)
#define a7 gl_FrontFacing
#define v2(B) Yi=B
#define Y gl_FragCoord.xy
#define n7
#define r3
#if defined(GE)||defined(MD)
#define kf(n8,h,B) if(!(n8)){x0(h,B);}
#define lf(n8,h,B) if(!(n8)){o1(h,B);}
#else
#define kf(n8,h,B) x0(h,B);
#define lf(n8,h,B) o1(h,B);
#endif
#ifndef K2
#define K2(a) layout(location=0) out i S1;D1(a)
#endif
#define J3 K1
#if defined(EC)&&!defined(XE)
#ifdef WE
#define J4(a) layout(input_attachment_index=0,binding=B2,set=O3) uniform mediump subpassInputMS a
#define I4(a) ub(mat4(subpassLoad(a,0),subpassLoad(a,1),subpassLoad(a,2),subpassLoad(a,3)),gl_SampleMaskIn[0])
#else
#define J4(a) layout(input_attachment_index=0,binding=B2,set=O3) uniform mediump subpassInput a
#define I4(a) subpassLoad(a)
#endif
#else
#define J4(a) Y2(c3,se,a)
#define I4(a) texelFetch(a,ivec2(floor(Y.xy)),0)
#endif
#define F0(E,J) ((E)*(J))
precision highp float;precision highp int;
#if MC<310
f i Zi(uint u){R y1=R(u&0xffu,(u>>8)&0xffu,(u>>16)&0xffu,u>>24);return e(y1)*(1./255.);}
#define unpackUnorm4x8 Zi
#endif
