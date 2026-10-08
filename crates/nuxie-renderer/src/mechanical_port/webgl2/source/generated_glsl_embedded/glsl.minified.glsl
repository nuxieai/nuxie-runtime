#define Wa
#ifndef KC
#define KC __VERSION__
#endif
#define c vec2
#define M vec3
#define h4 vec3
#define e vec4
#define d mediump float
#define D mediump vec2
#define v mediump vec3
#define i mediump vec4
#define w7 mediump mat3x3
#define x7 mediump mat2x3
#define Z4 mediump mat4x4
#define g0 ivec2
#define q6 ivec4
#define S0 uvec2
#define O uvec4
#define P mediump uint
#define Y4 bvec2
#define M6 bvec3
#define a8 bvec4
#define W mat2
#define f
#define c1(D2) out D2
#define e7(D2) inout D2
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
#define Z7(g,a) layout(binding=g,std140) uniform a{
#else
#define Z7(g,a) layout(std140) uniform a{
#endif
#define J9(a) }a;
#define f1(a)
#define K(g,k0,a) layout(location=g) in k0 a
#define g1
#define L(M9,B,a,k0)
#ifdef BB
#if KC>=310
#define X(g,k0,a) layout(location=g) out k0 a
#else
#define X(g,k0,a) out k0 a
#endif
#else
#if KC>=310
#define X(g,k0,a) layout(location=g) in k0 a
#else
#define X(g,k0,a) in k0 a
#endif
#endif
#define g3 flat
#define w2
#define l2
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
#define o4
#define p4
#endif
#ifdef EB
#define U3
#define V3
#endif
#define y5
#define z5
#ifdef DC
#define f5(e0,g,a) layout(set=e0,binding=g) uniform highp utexture2D a
#define N6(e0,g,a) layout(set=e0,binding=g) uniform highp texture2D a
#define p3(e0,g,a) layout(set=e0,binding=g) uniform mediump texture2D a
#define K5(e0,g,a) layout(binding=g) uniform mediump texture2D a
#if defined(EB)&&defined(CB)
#endif
#elif KC>=310
#define f5(e0,g,a) layout(binding=g) uniform highp usampler2D a
#define N6(e0,g,a) layout(binding=g) uniform highp sampler2D a
#define p3(e0,g,a) layout(binding=g) uniform mediump sampler2D a
#define K5(e0,g,a) layout(binding=g) uniform mediump sampler2D a
#else
#define f5(e0,g,a) uniform highp usampler2D a
#define N6(e0,g,a) uniform highp sampler2D a
#define p3(e0,g,a) uniform mediump sampler2D a
#define K5(e0,g,a) uniform mediump sampler2D a
#endif
#ifdef DC
#define O6(e0,g,a) layout(set=e0,binding=g) uniform mediump sampler a;
#ifdef RF
#define y4(c8,a) layout(set=Ah,binding=c8) uniform mediump sampler a;
#define r4(a) O6(x5,zh,a)
#else
#define y4(c8,a) layout(set=q3,binding=c8) uniform mediump sampler a;
#define r4(a) O6(x5,q4,a)
#endif
#define O5(a,p,o) texture(sampler2D(a,p),o)
#define o2(a,p,o,d1) textureLod(sampler2D(a,p),o,d1)
#define P5(a,p,o,g2) texture(sampler2D(a,p),o,g2)
#if defined(EB)&&defined(CB)&&defined(TE)
#extension GL_OES_sample_variables:require
#endif
#else
#define y4(c8,a)
#define O6(e0,g,a)
#define r4(a)
#define O5(a,p,o) texture(a,o)
#define o2(a,p,o,d1) textureLod(a,o,d1)
#define P5(a,p,o,g2) texture(a,o,g2)
#endif
#define O8(q0,p,o) O5(q0,p,o)
#define A5(q0,p,o,d1) o2(q0,p,o,d1)
#define d8(q0,p,o,g2) P5(q0,p,o,g2)
#define F6(e0,g,a) K5(e0,g,a)
#define v7(a,p,E,P6,O9,d1) o2(a,p,c(E,O9),d1)
#define Gi(e0,g,a) f5(e0,g,a)
#define a4
#define n1
#define r1(a,o) texelFetch(a,o,0)
#ifdef DC
#elif KC>=310
#else
#endif
#define W4
#define X4
#define k4
#define l4
#ifdef SF
#define g6(g,H1,a) f5(q3,g,a)
#define g5(g,H1,a) Gi(q3,g,a)
#define h6(g,H1,a) N6(q3,g,a)
#define p0(a,E0) r1(a,g0((E0)&Td,(E0)>>Sd))
#define q5(a,E0) r1(a,g0((E0)&Td,(E0)>>Sd)).xy
#else
#ifdef GL_ARB_shader_storage_buffer_object
#extension GL_ARB_shader_storage_buffer_object:require
#endif
#define g6(g,H1,a) layout(std430,binding=g) readonly buffer H1{S0 v2[];}a
#define g5(g,H1,a) layout(std430,binding=g) readonly buffer H1{O v2[];}a
#define h6(g,H1,a) layout(std430,binding=g) readonly buffer H1{e v2[];}a
#define Wb(g,H1,a) layout(std430,binding=g) buffer H1{uint v2[];}a
#define p0(a,E0) a.v2[E0]
#define q5(a,E0) a.v2[E0]
#define Oe(a,E0) a.v2[E0]
#define f8(a,E0,E) atomicMax(a.v2[E0],E)
#define Xb(a,E0,E) atomicAdd(a.v2[E0],E)
#define Hi(a,E0,E) atomicOr(a.v2[E0],E)
#endif
#ifdef LD
#define Y1(a) void main(){g0 G=ivec2(floor(d0));int M0=int(I9(uvec2(G),(j.L6+(vb-1u))&~(vb-1u)));
#define p2 }
#define m4 ,int M0
#define Z1 ,M0
#ifdef UE
#define V2(g,a) layout(std430,set=J3,binding=g) buffer a##Pe{uint v2[];}a
#elif defined(DC)
#define V2(g,a) layout(std430,set=J3,binding=g) coherent buffer a##Pe{uint v2[];}a
#else
#define V2(g,a) layout(std430,binding=g) coherent buffer a##Pe{uint v2[];}a
#endif
#define Yb V2
#define m3(h) h.v2[M0]
#define n3(h,C) h.v2[M0]=C
#define Zb(h) unpackUnorm4x8(m3(h))
#define ac(h,C) n3(h,packUnorm4x8(C))
#define v5(h,E) atomicMax(h.v2[M0],E)
#define w5(h,E) atomicAdd(h.v2[M0],E)
#elif defined(DE)||defined(TF)
#ifdef GL_ARB_shader_image_load_store
#extension GL_ARB_shader_image_load_store:require
#endif
#define Y1(a) void main(){g0 G=ivec2(floor(d0));
#define p2 }
#define m4 ,g0 G
#define Z1 ,G
#ifdef DC
#define Yb(g,a) layout(set=J3,binding=g,rgba8) uniform mediump coherent image2D a
#define V2(g,a) layout(set=J3,binding=g,r32ui) uniform highp coherent uimage2D a
#define bc(g,a) layout(set=J3,binding=g,rgb10_a2) uniform mediump coherent image2D a
#else
#define Yb(g,a) layout(binding=g,rgba8) uniform mediump coherent image2D a
#define V2(g,a) layout(binding=g,r32ui) uniform highp coherent uimage2D a
#define bc(g,a) layout(binding=g,rgb10_a2) uniform mediump coherent image2D a;
#endif
#define m3(h) imageLoad(h,G).x
#define n3(h,C) imageStore(h,G,uvec4(C))
#define Zb(h) imageLoad(h,G)
#define ac(h,C) imageStore(h,G,C)
#define v5(h,E) imageAtomicMax(h,G,E)
#define w5(h,E) imageAtomicAdd(h,G,E)
#else
#define Y1(a) void main()
#define p2
#define m4
#define Z1
#endif
#ifdef EXPORTED_PLS_IMPL_ANGLE
#extension GL_ANGLE_shader_pixel_local_storage:require
#define V1
#define C0(g,a) layout(binding=g,rgba8) uniform mediump pixelLocalANGLE a
#define q1(g,a) layout(binding=g,r32ui) uniform highp upixelLocalANGLE a
#define W1
#define R0(h) pixelLocalLoadANGLE(h)
#define l1(h) pixelLocalLoadANGLE(h).x
#define z0(h,C) pixelLocalStoreANGLE(h,C)
#define m1(h,C) pixelLocalStoreANGLE(h,uvec4(C))
#define N2(h)
#define h2(h)
#define O2
#define P2
#endif
#ifdef UF
#ifdef U
#extension GL_EXT_shader_pixel_local_storage2:require
#else
#extension GL_EXT_shader_pixel_local_storage:require
#endif
#define V1 __pixel_localEXT i2{
#define C0(g,a) layout(rgba8) mediump vec4 a
#define cc(g,a) layout(rgb10_a2) mediump vec4 a
#define q1(g,a) layout(r32ui) highp uint a
#define W1 };
#define R0(h) h
#define l1(h) h
#define z0(h,C) h=(C)
#define m1(h,C) h=(C)
#define N2(h) h=h
#define h2(h) h=h
#define O2
#define P2
#ifdef U
#define G2(a) layout(location=0,rgba8) out i N1;Y1(a)
#endif
#endif
#if defined(DE)||defined(LD)
#define V1
#define W1
#define C0 Yb
#define q1 V2
#define cc bc
#define R0 Zb
#define z0 ac
#define l1 m3
#define m1 n3
#define N2(h)
#define h2(h)
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
#ifdef VF
#define V1
#define N4(g,a) layout(input_attachment_index=g,binding=g,set=J3) uniform mediump subpassInput g8##a
#define Qe(g,a) layout(location=g) out mediump vec4 a
#define C0(g,a) N4(g,a);Qe(g,a)
#define q1(g,a) layout(input_attachment_index=g,binding=g,set=J3) uniform highp usubpassInput g8##a;layout(location=g) out highp uvec4 a
#define W1
#define R0(h) subpassLoad(g8##h)
#define l1(h) subpassLoad(g8##h).x
#define z0(h,C) h=(C)
#define m1(h,C) h.x=(C)
#define N2(h) z0(h,subpassLoad(g8##h))
#define h2(h) m1(h,subpassLoad(g8##h).x)
#define O2
#define P2
#endif
#ifdef WF
#define V1
#define C0(g,a) layout(location=g) out mediump vec4 a
#define q1(g,a) layout(location=g) out highp uvec4 a
#define W1
#define R0(h) vec4(0)
#define l1(h) 0u
#define z0(h,C) h=(C)
#define m1(h,C) h.x=(C)
#define N2(h) h=vec4(0)
#define h2(h) h.x=0u
#define O2
#define P2
#endif
#ifndef N4
#define N4 C0
#endif
#ifdef DC
#define dc gl_VertexIndex
#ifdef EE
#define h8 gl_InstanceIndex
#else
#define h8 0
#endif
#else
#ifdef VE
uniform highp int WE;
#define dc (gl_VertexID+WE)
#else
#define dc gl_VertexID
#endif
#ifdef EE
#ifdef FE
uniform highp int FE;
#define h8 (gl_InstanceID+FE)
#else
#define h8 (gl_InstanceID+gl_BaseInstance)
#endif
#else
#define h8 0
#endif
#endif
#define I6
#define P3
#define B7
#define e5
#define x1(a,f0,B,L2,z6) void main(){int L2=dc;int z6=h8;
#define A8(a,f0,B,D1,j0,L2,z6) x1(a,f0,B,L2,z6)
#define c7(a,B3,C3,S3,x2,D1,j0,L2) x1(a,B3,C3,L2,z6)
#define V(a,k0)
#define Z(a)
#define q(a,k0)
#define y1(Y0) gl_Position=Y0;}
#define W2(I1,a) layout(location=0) out I1 Ii;void main()
#define Q6(I1,a) W2(I1,a)
#define R6 gl_FrontFacing
#define K2(C) Ii=C
#define d0 gl_FragCoord.xy
#define f7
#define l3
#if defined(DE)||defined(LD)
#define Re(i8,h,C) if(!(i8)){z0(h,C);}
#define Se(i8,h,C) if(!(i8)){m1(h,C);}
#else
#define Re(i8,h,C) z0(h,C);
#define Se(i8,h,C) m1(h,C);
#endif
#ifndef G2
#define G2(a) layout(location=0) out i N1;Y1(a)
#endif
#define D3 p2
#if defined(DC)&&!defined(UE)
#ifdef TE
#define L5(a) layout(input_attachment_index=0,binding=U2,set=J3) uniform mediump subpassInputMS a
#define J5(a) gb(mat4(subpassLoad(a,0),subpassLoad(a,1),subpassLoad(a,2),subpassLoad(a,3)),gl_SampleMaskIn[0])
#else
#define L5(a) layout(input_attachment_index=0,binding=U2,set=J3) uniform mediump subpassInput a
#define J5(a) subpassLoad(a)
#endif
#else
#define L5(a) p3(q3,yh,a)
#define J5(a) texelFetch(a,ivec2(floor(d0.xy)),0)
#endif
#define y0(A,J) ((A)*(J))
precision highp float;precision highp int;
#if KC<310
f i Ji(uint u){O v1=O(u&0xffu,(u>>8)&0xffu,(u>>16)&0xffu,u>>24);return e(v1)*(1./255.);}
#define unpackUnorm4x8 Ji
#endif
