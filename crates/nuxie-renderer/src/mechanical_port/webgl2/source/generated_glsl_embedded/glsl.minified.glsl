#define rc
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
#define a7 mediump mat3x3
#define c7 mediump mat2x3
#define H4 mediump mat4x4
#define Y ivec2
#define f6 ivec4
#define a1 uvec2
#define H uvec4
#define N mediump uint
#define G4 bvec2
#define v6 bvec3
#define z7 bvec4
#define f0 mat2
#define e
#define Z0(l2) out l2
#define Y4(l2) inout l2
#ifdef GL_ANGLE_base_vertex_base_instance_shader_builtin
#extension GL_ANGLE_base_vertex_base_instance_shader_builtin:require
#endif
#ifdef JE
#extension GL_KHR_blend_equation_advanced:require
#endif
#ifdef WD
#extension GL_EXT_shader_framebuffer_fetch:require
#elif defined(XD)
#extension GL_EXT_shader_pixel_local_storage:require
#elif defined(EXPORTED_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
#extension GL_ANGLE_shader_pixel_local_storage:require
#elif defined(YD)
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
#define y7(f,a) layout(binding=f,std140)uniform a{
#else
#define y7(f,a) layout(std140)uniform a{
#endif
#define N8(a) }a;
#define Bd(a) layout(push_constant)uniform a{
#define Cd(Z,a) Z a;
#define Dd(a) }a;
#define g1(a)
#define L(f,Z,a) layout(location=f)in Z a
#define h1
#define M(Q8,F,a,Z)
#ifdef DB
#if NC>=310
#define X(f,Z,a) layout(location=f)out Z a
#else
#define X(f,Z,a) out Z a
#endif
#else
#if NC>=310
#define X(f,Z,a) layout(location=f)in Z a
#else
#define X(f,Z,a) in Z a
#endif
#endif
#define Q2 flat
#define m2
#define g2
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
#define f5
#define g5
#ifdef EC
#define F4(U,f,a) layout(set=U,binding=f)uniform highp utexture2D a
#define i5(U,f,a) layout(set=U,binding=f)uniform highp texture2D a
#define Z2(U,f,a) layout(set=U,binding=f)uniform mediump texture2D a
#define n5(U,f,a) layout(binding=f)uniform mediump texture2D a
#if defined(GB)&&defined(CB)
#endif
#elif NC>=310
#define F4(U,f,a) layout(binding=f)uniform highp usampler2D a
#define i5(U,f,a) layout(binding=f)uniform highp sampler2D a
#define Z2(U,f,a) layout(binding=f)uniform mediump sampler2D a
#define n5(U,f,a) layout(binding=f)uniform mediump sampler2D a
#else
#define F4(U,f,a) uniform highp usampler2D a
#define i5(U,f,a) uniform highp sampler2D a
#define Z2(U,f,a) uniform mediump sampler2D a
#define n5(U,f,a) uniform mediump sampler2D a
#endif
#ifdef EC
#define w6(U,f,a) layout(set=U,binding=f)uniform mediump sampler a;
#ifdef LF
#define c4(A7,a) layout(set=Rf,binding=A7)uniform mediump sampler a;
#define X3(a) w6(e5,Qf,a)
#else
#define c4(A7,a) layout(set=d3,binding=A7)uniform mediump sampler a;
#define X3(a) w6(e5,W3,a)
#endif
#define w5(a,p,l) texture(sampler2D(a,p),l)
#define o2(a,p,l,S0) textureLod(sampler2D(a,p),l,S0)
#define x5(a,p,l,R1) texture(sampler2D(a,p),l,R1)
#if defined(GB)&&defined(CB)
#extension GL_OES_sample_variables:require
#endif
#else
#define c4(A7,a)
#define w6(U,f,a)
#define X3(a)
#define w5(a,p,l) texture(a,l)
#define o2(a,p,l,S0) textureLod(a,l,S0)
#define x5(a,p,l,R1) texture(a,l,R1)
#endif
#define h8(k0,p,l) w5(k0,p,l)
#define V6(k0,p,l,S0) o2(k0,p,l,S0)
#define B7(k0,p,l,R1) x5(k0,p,l,R1)
#define j6(U,f,a) n5(U,f,a)
#define Z6(a,p,q,x6,S8,S0) o2(a,p,d(q,S8),S0)
#define Sg(U,f,a) F4(U,f,a)
#define I3
#define d1
#define q1(a,l) texelFetch(a,l,0)
#ifdef EC
#elif NC>=310
#else
#endif
#define C4
#define D4
#define Q3
#define R3
#ifdef MF
#define O5(f,w1,a) F4(d3,f,a)
#define K4(f,w1,a) Sg(d3,f,a)
#define P5(f,w1,a) i5(d3,f,a)
#define J0(a,A0) q1(a,Y((A0)&Hc,(A0)>>Gc))
#define Q5(a,A0) q1(a,Y((A0)&Hc,(A0)>>Gc)).xy
#else
#ifdef GL_ARB_shader_storage_buffer_object
#extension GL_ARB_shader_storage_buffer_object:require
#endif
#define O5(f,w1,a) layout(std430,binding=f)readonly buffer w1{a1 d2[];}a
#define K4(f,w1,a) layout(std430,binding=f)readonly buffer w1{H d2[];}a
#define P5(f,w1,a) layout(std430,binding=f)readonly buffer w1{g d2[];}a
#define Pa(f,w1,a) layout(std430,binding=f)buffer w1{uint d2[];}a
#define J0(a,A0) a.d2[A0]
#define Q5(a,A0) a.d2[A0]
#define Hd(a,A0) a.d2[A0]
#define D7(a,A0,q) atomicMax(a.d2[A0],q)
#define Qa(a,A0,q) atomicAdd(a.d2[A0],q)
#define Tg(a,A0,q) atomicOr(a.d2[A0],q)
#endif
#ifdef GD
#define M1(a) void main(){Y G=ivec2(floor(a0));int E0=int(M8(uvec2(G),(m.r6+(wa-1u))&~(wa-1u)));
#define Z1 }
#define S3 ,int E0
#define N1 ,E0
#ifdef ZD
#define E2(f,a) layout(std430,set=H3,binding=f)buffer a##Id{uint d2[];}a
#elif defined(EC)
#define E2(f,a) layout(std430,set=H3,binding=f)coherent buffer a##Id{uint d2[];}a
#else
#define E2(f,a) layout(std430,binding=f)coherent buffer a##Id{uint d2[];}a
#endif
#define Ra E2
#define V2(h) h.d2[E0]
#define W2(h,D) h.d2[E0]=D
#define Sa(h) unpackUnorm4x8(V2(h))
#define Ta(h,D) W2(h,packUnorm4x8(D))
#define a5(h,q) atomicMax(h.d2[E0],q)
#define c5(h,q) atomicAdd(h.d2[E0],q)
#elif defined(AE)||defined(NF)
#ifdef GL_ARB_shader_image_load_store
#extension GL_ARB_shader_image_load_store:require
#endif
#define M1(a) void main(){Y G=ivec2(floor(a0));
#define Z1 }
#define S3 ,Y G
#define N1 ,G
#ifdef EC
#define Ra(f,a) layout(set=H3,binding=f,rgba8)uniform mediump coherent image2D a
#define E2(f,a) layout(set=H3,binding=f,r32ui)uniform highp coherent uimage2D a
#define Ua(f,a) layout(set=H3,binding=f,rgb10_a2)uniform mediump coherent image2D a
#else
#define Ra(f,a) layout(binding=f,rgba8)uniform mediump coherent image2D a
#define E2(f,a) layout(binding=f,r32ui)uniform highp coherent uimage2D a
#define Ua(f,a) layout(binding=f,rgb10_a2)uniform mediump coherent image2D a;
#endif
#define V2(h) imageLoad(h,G).x
#define W2(h,D) imageStore(h,G,uvec4(D))
#define Sa(h) imageLoad(h,G)
#define Ta(h,D) imageStore(h,G,D)
#define a5(h,q) imageAtomicMax(h,G,q)
#define c5(h,q) imageAtomicAdd(h,G,q)
#else
#define M1(a) void main()
#define Z1
#define S3
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
#ifdef OF
#ifdef Q
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
#ifdef Q
#define p2(a) layout(location=0,rgba8)out i D1;M1(a)
#endif
#endif
#if defined(AE)||defined(GD)
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
#ifdef PF
#define J1
#define r4(f,a) layout(input_attachment_index=f,binding=f,set=H3)uniform mediump subpassInput E7##a
#define Jd(f,a) layout(location=f)out mediump vec4 a
#define x0(f,a) r4(f,a);Jd(f,a)
#define j1(f,a) layout(input_attachment_index=f,binding=f,set=H3)uniform highp usubpassInput E7##a;layout(location=f)out highp uvec4 a
#define K1
#define I0(h) subpassLoad(E7##h)
#define Y0(h) subpassLoad(E7##h).x
#define y0(h,D) h=(D)
#define c1(h,D) h.x=(D)
#define w2(h) y0(h,subpassLoad(E7##h))
#define e2(h) c1(h,subpassLoad(E7##h).x)
#define x2
#define y2
#endif
#ifdef QF
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
#ifdef EC
#define gl_VertexID gl_VertexIndex
#endif
#ifdef PE
#ifdef EC
#define T8 gl_InstanceIndex
#else
#ifdef BE
uniform highp int BE;
#define T8 (gl_InstanceID+BE)
#else
#define T8 (gl_InstanceID+gl_BaseInstance)
#endif
#endif
#else
#define T8 0
#endif
#define n6
#define w3
#define g7
#define y5
#define z1(a,e0,F,B,v) void main(){int B=gl_VertexID;int v=T8;
#define T7(a,e0,F,n1,g0,B,v) z1(a,e0,F,B,v)
#define K6(a,i3,j3,y3,z3,n1,g0,B) z1(a,i3,j3,B,v)
#define V(a,Z)
#define c0(a)
#define r(a,Z)
#define A1(O0) gl_Position=O0;}
#define a3(x1,a) layout(location=0)out x1 Ug;void main()
#define y6(x1,a) a3(x1,a)
#define z6 gl_FrontFacing
#define I2(D) Ug=D
#define a0 gl_FragCoord.xy
#define M6
#define U2
#if defined(AE)||defined(GD)
#define Kd(F7,h,D) if(!(F7)){y0(h,D);}
#define Ld(F7,h,D) if(!(F7)){c1(h,D);}
#else
#define Kd(F7,h,D) y0(h,D);
#define Ld(F7,h,D) c1(h,D);
#endif
#ifndef p2
#define p2(a) layout(location=0)out i D1;M1(a)
#endif
#define n3 Z1
#if defined(EC)&&!defined(ZD)
#define k6(a) layout(input_attachment_index=0,binding=S2,set=H3)uniform mediump subpassInputMS a
#define G7(a) ma(mat4(subpassLoad(a,0),subpassLoad(a,1),subpassLoad(a,2),subpassLoad(a,3)),gl_SampleMaskIn[0])
#else
#define k6(a) Z2(d3,Pf,a)
#define G7(a) texelFetch(a,ivec2(floor(a0.xy)),0)
#endif
#define R0(C,I) ((C)*(I))
precision highp float;precision highp int;
#if NC<310
e i Vg(uint u){H T1=H(u&0xffu,(u>>8)&0xffu,(u>>16)&0xffu,u>>24);return g(T1)*(1./255.);}
#define unpackUnorm4x8 Vg
#endif
