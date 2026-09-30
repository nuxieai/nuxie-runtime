#define yc
#ifndef LC
#define LC __VERSION__
#endif
#define c vec2
#define Q vec3
#define N3 vec3
#define f vec4
#define d mediump float
#define E mediump vec2
#define A mediump vec3
#define i mediump vec4
#define a7 mediump mat3x3
#define c7 mediump mat2x3
#define G4 mediump mat4x4
#define Y ivec2
#define g6 ivec4
#define a1 uvec2
#define X uvec4
#define L mediump uint
#define F4 bvec2
#define r6 bvec3
#define z7 bvec4
#define d0 mat2
#define e
#define Z0(n2) out n2
#define X4(n2) inout n2
#ifdef GL_ANGLE_base_vertex_base_instance_shader_builtin
#extension GL_ANGLE_base_vertex_base_instance_shader_builtin:require
#endif
#ifdef ME
#extension GL_KHR_blend_equation_advanced:require
#endif
#ifdef ZD
#extension GL_EXT_shader_framebuffer_fetch:require
#elif defined(AE)
#extension GL_EXT_shader_pixel_local_storage:require
#elif defined(EXPORTED_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
#extension GL_ANGLE_shader_pixel_local_storage:require
#elif defined(BE)
#ifdef GL_ARB_shader_image_load_store
#extension GL_ARB_shader_image_load_store:require
#endif
#ifdef GL_OES_shader_image_atomic
#extension GL_OES_shader_image_atomic:require
#endif
#endif
#if defined(CB)&&defined(BB)&&defined(GL_ES)&&!defined(SE)
#ifdef GL_EXT_clip_cull_distance
#extension GL_EXT_clip_cull_distance:require
#elif defined(GL_ANGLE_clip_cull_distance)
#extension GL_ANGLE_clip_cull_distance:require
#endif
#endif
#if LC>=310
#define y7(g,a) layout(binding=g,std140)uniform a{
#else
#define y7(g,a) layout(std140)uniform a{
#endif
#define N8(a) }a;
#define Jd(a) layout(push_constant)uniform a{
#define Kd(Z,a) Z a;
#define Ld(a) }a;
#define f1(a)
#define J(g,Z,a) layout(location=g)in Z a
#define g1
#define K(Q8,F,a,Z)
#ifdef DB
#if LC>=310
#define V(g,Z,a) layout(location=g)out Z a
#else
#define V(g,Z,a) out Z a
#endif
#else
#if LC>=310
#define V(g,Z,a) layout(location=g)in Z a
#else
#define V(g,Z,a) in Z a
#endif
#endif
#define T2 flat
#define p2
#define h2
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
#define U3
#define V3
#endif
#ifdef FB
#define F3
#define G3
#endif
#define e5
#define f5
#ifdef CC
#define E4(U,g,a) layout(set=U,binding=g)uniform highp utexture2D a
#define h5(U,g,a) layout(set=U,binding=g)uniform highp texture2D a
#define c3(U,g,a) layout(set=U,binding=g)uniform mediump texture2D a
#define m5(U,g,a) layout(binding=g)uniform mediump texture2D a
#if defined(FB)&&defined(CB)
#endif
#elif LC>=310
#define E4(U,g,a) layout(binding=g)uniform highp usampler2D a
#define h5(U,g,a) layout(binding=g)uniform highp sampler2D a
#define c3(U,g,a) layout(binding=g)uniform mediump sampler2D a
#define m5(U,g,a) layout(binding=g)uniform mediump sampler2D a
#else
#define E4(U,g,a) uniform highp usampler2D a
#define h5(U,g,a) uniform highp sampler2D a
#define c3(U,g,a) uniform mediump sampler2D a
#define m5(U,g,a) uniform mediump sampler2D a
#endif
#ifdef CC
#define v6(U,g,a) layout(set=U,binding=g)uniform mediump sampler a;
#ifdef QF
#define c4(A7,a) layout(set=gg,binding=A7)uniform mediump sampler a;
#define X3(a) v6(d5,fg,a)
#else
#define c4(A7,a) layout(set=e3,binding=A7)uniform mediump sampler a;
#define X3(a) v6(d5,W3,a)
#endif
#define w5(a,p,l) texture(sampler2D(a,p),l)
#define i2(a,p,l,S0) textureLod(sampler2D(a,p),l,S0)
#define x5(a,p,l,R1) texture(sampler2D(a,p),l,R1)
#if defined(FB)&&defined(CB)&&defined(TE)
#extension GL_OES_sample_variables:require
#endif
#else
#define c4(A7,a)
#define v6(U,g,a)
#define X3(a)
#define w5(a,p,l) texture(a,l)
#define i2(a,p,l,S0) textureLod(a,l,S0)
#define x5(a,p,l,R1) texture(a,l,R1)
#endif
#define h8(l0,p,l) w5(l0,p,l)
#define V6(l0,p,l,S0) i2(l0,p,l,S0)
#define B7(l0,p,l,R1) x5(l0,p,l,R1)
#define k6(U,g,a) m5(U,g,a)
#define Z6(a,p,q,w6,S8,S0) i2(a,p,c(q,S8),S0)
#define gh(U,g,a) E4(U,g,a)
#define I3
#define d1
#define p1(a,l) texelFetch(a,l,0)
#ifdef CC
#elif LC>=310
#else
#endif
#define B4
#define C4
#define P3
#define Q3
#ifdef RF
#define O5(g,v1,a) E4(e3,g,a)
#define J4(g,v1,a) gh(e3,g,a)
#define P5(g,v1,a) h5(e3,g,a)
#define K0(a,B0) p1(a,Y((B0)&Oc,(B0)>>Nc))
#define R5(a,B0) p1(a,Y((B0)&Oc,(B0)>>Nc)).xy
#else
#ifdef GL_ARB_shader_storage_buffer_object
#extension GL_ARB_shader_storage_buffer_object:require
#endif
#define O5(g,v1,a) layout(std430,binding=g)readonly buffer v1{a1 e2[];}a
#define J4(g,v1,a) layout(std430,binding=g)readonly buffer v1{X e2[];}a
#define P5(g,v1,a) layout(std430,binding=g)readonly buffer v1{f e2[];}a
#define Ta(g,v1,a) layout(std430,binding=g)buffer v1{uint e2[];}a
#define K0(a,B0) a.e2[B0]
#define R5(a,B0) a.e2[B0]
#define Pd(a,B0) a.e2[B0]
#define D7(a,B0,q) atomicMax(a.e2[B0],q)
#define Ua(a,B0,q) atomicAdd(a.e2[B0],q)
#define hh(a,B0,q) atomicOr(a.e2[B0],q)
#endif
#ifdef JD
#define M1(a) void main(){Y G=ivec2(floor(c0));int F0=int(M8(uvec2(G),(n.q6+(Aa-1u))&~(Aa-1u)));
#define a2 }
#define R3 ,int F0
#define N1 ,F0
#ifdef CE
#define H2(g,a) layout(std430,set=p3,binding=g)buffer a##Qd{uint e2[];}a
#elif defined(CC)
#define H2(g,a) layout(std430,set=p3,binding=g)coherent buffer a##Qd{uint e2[];}a
#else
#define H2(g,a) layout(std430,binding=g)coherent buffer a##Qd{uint e2[];}a
#endif
#define Va H2
#define X2(h) h.e2[F0]
#define Y2(h,D) h.e2[F0]=D
#define Wa(h) unpackUnorm4x8(X2(h))
#define Xa(h,D) Y2(h,packUnorm4x8(D))
#define Z4(h,q) atomicMax(h.e2[F0],q)
#define a5(h,q) atomicAdd(h.e2[F0],q)
#elif defined(DE)||defined(SF)
#ifdef GL_ARB_shader_image_load_store
#extension GL_ARB_shader_image_load_store:require
#endif
#define M1(a) void main(){Y G=ivec2(floor(c0));
#define a2 }
#define R3 ,Y G
#define N1 ,G
#ifdef CC
#define Va(g,a) layout(set=p3,binding=g,rgba8)uniform mediump coherent image2D a
#define H2(g,a) layout(set=p3,binding=g,r32ui)uniform highp coherent uimage2D a
#define Ya(g,a) layout(set=p3,binding=g,rgb10_a2)uniform mediump coherent image2D a
#else
#define Va(g,a) layout(binding=g,rgba8)uniform mediump coherent image2D a
#define H2(g,a) layout(binding=g,r32ui)uniform highp coherent uimage2D a
#define Ya(g,a) layout(binding=g,rgb10_a2)uniform mediump coherent image2D a;
#endif
#define X2(h) imageLoad(h,G).x
#define Y2(h,D) imageStore(h,G,uvec4(D))
#define Wa(h) imageLoad(h,G)
#define Xa(h,D) imageStore(h,G,D)
#define Z4(h,q) imageAtomicMax(h,G,q)
#define a5(h,q) imageAtomicAdd(h,G,q)
#else
#define M1(a) void main()
#define a2
#define R3
#define N1
#endif
#ifdef EXPORTED_PLS_IMPL_ANGLE
#extension GL_ANGLE_shader_pixel_local_storage:require
#define J1
#define y0(g,a) layout(binding=g,rgba8)uniform mediump pixelLocalANGLE a
#define i1(g,a) layout(binding=g,r32ui)uniform highp upixelLocalANGLE a
#define K1
#define J0(h) pixelLocalLoadANGLE(h)
#define Y0(h) pixelLocalLoadANGLE(h).x
#define z0(h,D) pixelLocalStoreANGLE(h,D)
#define c1(h,D) pixelLocalStoreANGLE(h,uvec4(D))
#define y2(h)
#define f2(h)
#define z2
#define A2
#endif
#ifdef TF
#ifdef O
#extension GL_EXT_shader_pixel_local_storage2:require
#else
#extension GL_EXT_shader_pixel_local_storage:require
#endif
#define J1 __pixel_localEXT S1{
#define y0(g,a) layout(rgba8)mediump vec4 a
#define Za(g,a) layout(rgb10_a2)mediump vec4 a
#define i1(g,a) layout(r32ui)highp uint a
#define K1 };
#define J0(h) h
#define Y0(h) h
#define z0(h,D) h=(D)
#define c1(h,D) h=(D)
#define y2(h) h=h
#define f2(h) h=h
#define z2
#define A2
#ifdef O
#define r2(a) layout(location=0,rgba8)out i C1;M1(a)
#endif
#endif
#if defined(DE)||defined(JD)
#define J1
#define K1
#define y0 Va
#define i1 H2
#define Za Ya
#define J0 Wa
#define z0 Xa
#define Y0 X2
#define c1 Y2
#define y2(h)
#define f2(h)
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
#ifdef UF
#define J1
#define v4(g,a) layout(input_attachment_index=g,binding=g,set=p3)uniform mediump subpassInput E7##a
#define Rd(g,a) layout(location=g)out mediump vec4 a
#define y0(g,a) v4(g,a);Rd(g,a)
#define i1(g,a) layout(input_attachment_index=g,binding=g,set=p3)uniform highp usubpassInput E7##a;layout(location=g)out highp uvec4 a
#define K1
#define J0(h) subpassLoad(E7##h)
#define Y0(h) subpassLoad(E7##h).x
#define z0(h,D) h=(D)
#define c1(h,D) h.x=(D)
#define y2(h) z0(h,subpassLoad(E7##h))
#define f2(h) c1(h,subpassLoad(E7##h).x)
#define z2
#define A2
#endif
#ifdef VF
#define J1
#define y0(g,a) layout(location=g)out mediump vec4 a
#define i1(g,a) layout(location=g)out highp uvec4 a
#define K1
#define J0(h) vec4(0)
#define Y0(h) 0u
#define z0(h,D) h=(D)
#define c1(h,D) h.x=(D)
#define y2(h) h=vec4(0)
#define f2(h) h.x=0u
#define z2
#define A2
#endif
#ifndef v4
#define v4 y0
#endif
#ifdef CC
#define gl_VertexID gl_VertexIndex
#endif
#ifdef UE
#ifdef CC
#define T8 gl_InstanceIndex
#else
#ifdef EE
uniform highp int EE;
#define T8 (gl_InstanceID+EE)
#else
#define T8 (gl_InstanceID+gl_BaseInstance)
#endif
#endif
#else
#define T8 0
#endif
#define n6
#define x3
#define g7
#define y5
#define y1(a,f0,F,B,v) void main(){int B=gl_VertexID;int v=T8;
#define T7(a,f0,F,m1,g0,B,v) y1(a,f0,F,B,v)
#define K6(a,j3,k3,z3,A3,m1,g0,B) y1(a,j3,k3,B,v)
#define T(a,Z)
#define a0(a)
#define r(a,Z)
#define z1(P0) gl_Position=P0;}
#define d3(w1,a) layout(location=0)out w1 ih;void main()
#define x6(w1,a) d3(w1,a)
#define y6 gl_FrontFacing
#define L2(D) ih=D
#define c0 gl_FragCoord.xy
#define M6
#define W2
#if defined(DE)||defined(JD)
#define Sd(F7,h,D) if(!(F7)){z0(h,D);}
#define Td(F7,h,D) if(!(F7)){c1(h,D);}
#else
#define Sd(F7,h,D) z0(h,D);
#define Td(F7,h,D) c1(h,D);
#endif
#ifndef r2
#define r2(a) layout(location=0)out i C1;M1(a)
#endif
#define m3 a2
#if defined(CC)&&!defined(CE)
#ifdef TE
#define n5(a) layout(input_attachment_index=0,binding=G2,set=p3)uniform mediump subpassInputMS a
#define z6(a) pa(mat4(subpassLoad(a,0),subpassLoad(a,1),subpassLoad(a,2),subpassLoad(a,3)),gl_SampleMaskIn[0])
#else
#define n5(a) layout(input_attachment_index=0,binding=G2,set=p3)uniform mediump subpassInput a
#define z6(a) subpassLoad(a)
#endif
#else
#define n5(a) c3(e3,eg,a)
#define z6(a) texelFetch(a,ivec2(floor(c0.xy)),0)
#endif
#define N0(C,H) ((C)*(H))
precision highp float;precision highp int;
#if LC<310
e i jh(uint u){X T1=X(u&0xffu,(u>>8)&0xffu,(u>>16)&0xffu,u>>24);return f(T1)*(1./255.);}
#define unpackUnorm4x8 jh
#endif
