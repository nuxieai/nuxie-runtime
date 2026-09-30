#define Gc
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
#define d7 mediump mat3x3
#define e7 mediump mat2x3
#define K4 mediump mat4x4
#define Z ivec2
#define h6 ivec4
#define N0 uvec2
#define Y uvec4
#define N mediump uint
#define J4 bvec2
#define v6 bvec3
#define B7 bvec4
#define e0 mat2
#define e
#define c1(o2) out o2
#define N6(o2) inout o2
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
#define A7(g,a) layout(binding=g,std140)uniform a{
#else
#define A7(g,a) layout(std140)uniform a{
#endif
#define P8(a) }a;
#define Rd(a) layout(push_constant)uniform a{
#define Sd(a0,a) a0 a;
#define Td(a) }a;
#define h1(a)
#define K(g,a0,a) layout(location=g)in a0 a
#define i1
#define L(S8,F,a,a0)
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
#define V2 flat
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
#define Y3
#define Z3
#endif
#ifdef EB
#define I3
#define J3
#endif
#define i5
#define j5
#ifdef BC
#define I4(V,g,a) layout(set=V,binding=g)uniform highp utexture2D a
#define l5(V,g,a) layout(set=V,binding=g)uniform highp texture2D a
#define e3(V,g,a) layout(set=V,binding=g)uniform mediump texture2D a
#define q5(V,g,a) layout(binding=g)uniform mediump texture2D a
#if defined(EB)&&defined(BB)
#endif
#elif KC>=310
#define I4(V,g,a) layout(binding=g)uniform highp usampler2D a
#define l5(V,g,a) layout(binding=g)uniform highp sampler2D a
#define e3(V,g,a) layout(binding=g)uniform mediump sampler2D a
#define q5(V,g,a) layout(binding=g)uniform mediump sampler2D a
#else
#define I4(V,g,a) uniform highp usampler2D a
#define l5(V,g,a) uniform highp sampler2D a
#define e3(V,g,a) uniform mediump sampler2D a
#define q5(V,g,a) uniform mediump sampler2D a
#endif
#ifdef BC
#define w6(V,g,a) layout(set=V,binding=g)uniform mediump sampler a;
#ifdef PF
#define g4(C7,a) layout(set=pg,binding=C7)uniform mediump sampler a;
#define c4(a) w6(h5,og,a)
#else
#define g4(C7,a) layout(set=h3,binding=C7)uniform mediump sampler a;
#define c4(a) w6(h5,a4,a)
#endif
#define A5(a,p,m) texture(sampler2D(a,p),m)
#define j2(a,p,m,U0) textureLod(sampler2D(a,p),m,U0)
#define B5(a,p,m,U1) texture(sampler2D(a,p),m,U1)
#if defined(EB)&&defined(BB)&&defined(SE)
#extension GL_OES_sample_variables:require
#endif
#else
#define g4(C7,a)
#define w6(V,g,a)
#define c4(a)
#define A5(a,p,m) texture(a,m)
#define j2(a,p,m,U0) textureLod(a,m,U0)
#define B5(a,p,m,U1) texture(a,m,U1)
#endif
#define j8(n0,p,m) A5(n0,p,m)
#define X6(n0,p,m,U0) j2(n0,p,m,U0)
#define D7(n0,p,m,U1) B5(n0,p,m,U1)
#define l6(V,g,a) q5(V,g,a)
#define c7(a,p,q,x6,U8,U0) j2(a,p,c(q,U8),U0)
#define ph(V,g,a) I4(V,g,a)
#define L3
#define e1
#define v1(a,m) texelFetch(a,m,0)
#ifdef BC
#elif KC>=310
#else
#endif
#define F4
#define G4
#define U3
#define V3
#ifdef QF
#define R5(g,y1,a) I4(h3,g,a)
#define N4(g,y1,a) ph(h3,g,a)
#define S5(g,y1,a) l5(h3,g,a)
#define L0(a,C0) v1(a,Z((C0)&Wc,(C0)>>Vc))
#define U5(a,C0) v1(a,Z((C0)&Wc,(C0)>>Vc)).xy
#else
#ifdef GL_ARB_shader_storage_buffer_object
#extension GL_ARB_shader_storage_buffer_object:require
#endif
#define R5(g,y1,a) layout(std430,binding=g)readonly buffer y1{N0 g2[];}a
#define N4(g,y1,a) layout(std430,binding=g)readonly buffer y1{Y g2[];}a
#define S5(g,y1,a) layout(std430,binding=g)readonly buffer y1{f g2[];}a
#define Xa(g,y1,a) layout(std430,binding=g)buffer y1{uint g2[];}a
#define L0(a,C0) a.g2[C0]
#define U5(a,C0) a.g2[C0]
#define Xd(a,C0) a.g2[C0]
#define F7(a,C0,q) atomicMax(a.g2[C0],q)
#define Ya(a,C0,q) atomicAdd(a.g2[C0],q)
#define qh(a,C0,q) atomicOr(a.g2[C0],q)
#endif
#ifdef ID
#define P1(a) void main(){Z G=ivec2(floor(d0));int G0=int(O8(uvec2(G),(j.r6+(Da-1u))&~(Da-1u)));
#define d2 }
#define W3 ,int G0
#define Q1 ,G0
#ifdef BE
#define H2(g,a) layout(std430,set=w3,binding=g)buffer a##Yd{uint g2[];}a
#elif defined(BC)
#define H2(g,a) layout(std430,set=w3,binding=g)coherent buffer a##Yd{uint g2[];}a
#else
#define H2(g,a) layout(std430,binding=g)coherent buffer a##Yd{uint g2[];}a
#endif
#define Za H2
#define Z2(h) h.g2[G0]
#define a3(h,D) h.g2[G0]=D
#define ab(h) unpackUnorm4x8(Z2(h))
#define bb(h,D) a3(h,packUnorm4x8(D))
#define e5(h,q) atomicMax(h.g2[G0],q)
#define f5(h,q) atomicAdd(h.g2[G0],q)
#elif defined(CE)||defined(RF)
#ifdef GL_ARB_shader_image_load_store
#extension GL_ARB_shader_image_load_store:require
#endif
#define P1(a) void main(){Z G=ivec2(floor(d0));
#define d2 }
#define W3 ,Z G
#define Q1 ,G
#ifdef BC
#define Za(g,a) layout(set=w3,binding=g,rgba8)uniform mediump coherent image2D a
#define H2(g,a) layout(set=w3,binding=g,r32ui)uniform highp coherent uimage2D a
#define cb(g,a) layout(set=w3,binding=g,rgb10_a2)uniform mediump coherent image2D a
#else
#define Za(g,a) layout(binding=g,rgba8)uniform mediump coherent image2D a
#define H2(g,a) layout(binding=g,r32ui)uniform highp coherent uimage2D a
#define cb(g,a) layout(binding=g,rgb10_a2)uniform mediump coherent image2D a;
#endif
#define Z2(h) imageLoad(h,G).x
#define a3(h,D) imageStore(h,G,uvec4(D))
#define ab(h) imageLoad(h,G)
#define bb(h,D) imageStore(h,G,D)
#define e5(h,q) imageAtomicMax(h,G,q)
#define f5(h,q) imageAtomicAdd(h,G,q)
#else
#define P1(a) void main()
#define d2
#define W3
#define Q1
#endif
#ifdef EXPORTED_PLS_IMPL_ANGLE
#extension GL_ANGLE_shader_pixel_local_storage:require
#define M1
#define z0(g,a) layout(binding=g,rgba8)uniform mediump pixelLocalANGLE a
#define k1(g,a) layout(binding=g,r32ui)uniform highp upixelLocalANGLE a
#define N1
#define K0(h) pixelLocalLoadANGLE(h)
#define a1(h) pixelLocalLoadANGLE(h).x
#define A0(h,D) pixelLocalStoreANGLE(h,D)
#define d1(h,D) pixelLocalStoreANGLE(h,uvec4(D))
#define y2(h)
#define h2(h)
#define z2
#define A2
#endif
#ifdef SF
#ifdef Q
#extension GL_EXT_shader_pixel_local_storage2:require
#else
#extension GL_EXT_shader_pixel_local_storage:require
#endif
#define M1 __pixel_localEXT V1{
#define z0(g,a) layout(rgba8)mediump vec4 a
#define db(g,a) layout(rgb10_a2)mediump vec4 a
#define k1(g,a) layout(r32ui)highp uint a
#define N1 };
#define K0(h) h
#define a1(h) h
#define A0(h,D) h=(D)
#define d1(h,D) h=(D)
#define y2(h) h=h
#define h2(h) h=h
#define z2
#define A2
#ifdef Q
#define v2(a) layout(location=0,rgba8)out i F1;P1(a)
#endif
#endif
#if defined(CE)||defined(ID)
#define M1
#define N1
#define z0 Za
#define k1 H2
#define db cb
#define K0 ab
#define A0 bb
#define a1 Z2
#define d1 a3
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
#define M1
#define y4(g,a) layout(input_attachment_index=g,binding=g,set=w3)uniform mediump subpassInput G7##a
#define Zd(g,a) layout(location=g)out mediump vec4 a
#define z0(g,a) y4(g,a);Zd(g,a)
#define k1(g,a) layout(input_attachment_index=g,binding=g,set=w3)uniform highp usubpassInput G7##a;layout(location=g)out highp uvec4 a
#define N1
#define K0(h) subpassLoad(G7##h)
#define a1(h) subpassLoad(G7##h).x
#define A0(h,D) h=(D)
#define d1(h,D) h.x=(D)
#define y2(h) A0(h,subpassLoad(G7##h))
#define h2(h) d1(h,subpassLoad(G7##h).x)
#define z2
#define A2
#endif
#ifdef UF
#define M1
#define z0(g,a) layout(location=g)out mediump vec4 a
#define k1(g,a) layout(location=g)out highp uvec4 a
#define N1
#define K0(h) vec4(0)
#define a1(h) 0u
#define A0(h,D) h=(D)
#define d1(h,D) h.x=(D)
#define y2(h) h=vec4(0)
#define h2(h) h.x=0u
#define z2
#define A2
#endif
#ifndef y4
#define y4 z0
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
#define o6
#define A3
#define i7
#define C5
#define B1(a,g0,F,B,v) void main(){int B=gl_VertexID;int v=V8;
#define V7(a,g0,F,p1,h0,B,v) B1(a,g0,F,B,v)
#define L6(a,n3,o3,C3,D3,p1,h0,B) B1(a,n3,o3,B,v)
#define U(a,a0)
#define c0(a)
#define r(a,a0)
#define C1(R0) gl_Position=R0;}
#define f3(z1,a) layout(location=0)out z1 rh;void main()
#define y6(z1,a) f3(z1,a)
#define z6 gl_FrontFacing
#define M2(D) rh=D
#define d0 gl_FragCoord.xy
#define O6
#define Y2
#if defined(CE)||defined(ID)
#define ae(H7,h,D) if(!(H7)){A0(h,D);}
#define be(H7,h,D) if(!(H7)){d1(h,D);}
#else
#define ae(H7,h,D) A0(h,D);
#define be(H7,h,D) d1(h,D);
#endif
#ifndef v2
#define v2(a) layout(location=0)out i F1;P1(a)
#endif
#define r3 d2
#if defined(BC)&&!defined(BE)
#ifdef SE
#define r5(a) layout(input_attachment_index=0,binding=G2,set=w3)uniform mediump subpassInputMS a
#define A6(a) sa(mat4(subpassLoad(a,0),subpassLoad(a,1),subpassLoad(a,2),subpassLoad(a,3)),gl_SampleMaskIn[0])
#else
#define r5(a) layout(input_attachment_index=0,binding=G2,set=w3)uniform mediump subpassInput a
#define A6(a) subpassLoad(a)
#endif
#else
#define r5(a) e3(h3,ng,a)
#define A6(a) texelFetch(a,ivec2(floor(d0.xy)),0)
#endif
#define P0(C,H) ((C)*(H))
precision highp float;precision highp int;
#if KC<310
e i sh(uint u){Y l1=Y(u&0xffu,(u>>8)&0xffu,(u>>16)&0xffu,u>>24);return f(l1)*(1./255.);}
#define unpackUnorm4x8 sh
#endif
