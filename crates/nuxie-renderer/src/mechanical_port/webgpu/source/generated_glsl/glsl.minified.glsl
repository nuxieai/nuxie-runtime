#define Ac
#ifndef GLSL_VERSION
#define GLSL_VERSION __VERSION__
#endif
#define c vec2
#define Q vec3
#define O3 vec3
#define f vec4
#define d mediump float
#define E mediump vec2
#define A mediump vec3
#define i mediump vec4
#define a7 mediump mat3x3
#define c7 mediump mat2x3
#define G4 mediump mat4x4
#define Y ivec2
#define e6 ivec4
#define a1 uvec2
#define X uvec4
#define L mediump uint
#define F4 bvec2
#define p6 bvec3
#define z7 bvec4
#define d0 mat2
#define e
#define Z0(n2) out n2
#define L6(n2) inout n2
#ifdef GL_ANGLE_base_vertex_base_instance_shader_builtin
#extension GL_ANGLE_base_vertex_base_instance_shader_builtin:require
#endif
#ifdef ENABLE_KHR_BLEND
#extension GL_KHR_blend_equation_advanced:require
#endif
#ifdef ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH
#extension GL_EXT_shader_framebuffer_fetch:require
#elif defined(ATLAS_RENDER_TARGET_R8_PLS_EXT)
#extension GL_EXT_shader_pixel_local_storage:require
#elif defined(ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
#extension GL_ANGLE_shader_pixel_local_storage:require
#elif defined(ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE)
#ifdef GL_ARB_shader_image_load_store
#extension GL_ARB_shader_image_load_store:require
#endif
#ifdef GL_OES_shader_image_atomic
#extension GL_OES_shader_image_atomic:require
#endif
#endif
#if defined(RENDER_MODE_DEPTH_STENCIL)&&defined(ENABLE_CLIP_RECT)&&defined(GL_ES)&&!defined(DISABLE_CLIP_DISTANCE_FOR_UBERSHADERS)
#ifdef GL_EXT_clip_cull_distance
#extension GL_EXT_clip_cull_distance:require
#elif defined(GL_ANGLE_clip_cull_distance)
#extension GL_ANGLE_clip_cull_distance:require
#endif
#endif
#if GLSL_VERSION>=310
#define y7(g,a) layout(binding=g,std140)uniform a{
#else
#define y7(g,a) layout(std140)uniform a{
#endif
#define N8(a) }a;
#define Ld(a) layout(push_constant)uniform a{
#define Md(Z,a) Z a;
#define Nd(a) }a;
#define f1(a)
#define J(g,Z,a) layout(location=g)in Z a
#define g1
#define K(Q8,F,a,Z)
#ifdef VERTEX
#if GLSL_VERSION>=310
#define V(g,Z,a) layout(location=g)out Z a
#else
#define V(g,Z,a) out Z a
#endif
#else
#if GLSL_VERSION>=310
#define V(g,Z,a) layout(location=g)in Z a
#else
#define V(g,Z,a) in Z a
#endif
#endif
#define S2 flat
#define p2
#define h2
#ifdef TARGET_SPIRV
#define H0
#else
#ifdef GL_NV_shader_noperspective_interpolation
#extension GL_NV_shader_noperspective_interpolation:require
#define H0 noperspective
#else
#define H0
#endif
#endif
#ifdef VERTEX
#define V3
#define W3
#endif
#ifdef FRAGMENT
#define F3
#define G3
#endif
#define d5
#define e5
#ifdef TARGET_SPIRV
#define E4(U,g,a) layout(set=U,binding=g)uniform highp utexture2D a
#define g5(U,g,a) layout(set=U,binding=g)uniform highp texture2D a
#define a3(U,g,a) layout(set=U,binding=g)uniform mediump texture2D a
#define l5(U,g,a) layout(binding=g)uniform mediump texture2D a
#if defined(FRAGMENT)&&defined(RENDER_MODE_DEPTH_STENCIL)
#endif
#elif GLSL_VERSION>=310
#define E4(U,g,a) layout(binding=g)uniform highp usampler2D a
#define g5(U,g,a) layout(binding=g)uniform highp sampler2D a
#define a3(U,g,a) layout(binding=g)uniform mediump sampler2D a
#define l5(U,g,a) layout(binding=g)uniform mediump sampler2D a
#else
#define E4(U,g,a) uniform highp usampler2D a
#define g5(U,g,a) uniform highp sampler2D a
#define a3(U,g,a) uniform mediump sampler2D a
#define l5(U,g,a) uniform mediump sampler2D a
#endif
#ifdef TARGET_SPIRV
#define q6(U,g,a) layout(set=U,binding=g)uniform mediump sampler a;
#ifdef USE_WEBGPU_SAMPLERS
#define d4(A7,a) layout(set=ig,binding=A7)uniform mediump sampler a;
#define Y3(a) q6(c5,hg,a)
#else
#define d4(A7,a) layout(set=d3,binding=A7)uniform mediump sampler a;
#define Y3(a) q6(c5,X3,a)
#endif
#define v5(a,p,n) texture(sampler2D(a,p),n)
#define i2(a,p,n,S0) textureLod(sampler2D(a,p),n,S0)
#define w5(a,p,n,R1) texture(sampler2D(a,p),n,R1)
#if defined(FRAGMENT)&&defined(RENDER_MODE_DEPTH_STENCIL)&&defined(MSAA_DST_COLOR)
#extension GL_OES_sample_variables:require
#endif
#else
#define d4(A7,a)
#define q6(U,g,a)
#define Y3(a)
#define v5(a,p,n) texture(a,n)
#define i2(a,p,n,S0) textureLod(a,n,S0)
#define w5(a,p,n,R1) texture(a,n,R1)
#endif
#define h8(l0,p,n) v5(l0,p,n)
#define V6(l0,p,n,S0) i2(l0,p,n,S0)
#define B7(l0,p,n,R1) w5(l0,p,n,R1)
#define i6(U,g,a) l5(U,g,a)
#define Z6(a,p,q,r6,S8,S0) i2(a,p,c(q,S8),S0)
#define ih(U,g,a) E4(U,g,a)
#define I3
#define d1
#define p1(a,n) texelFetch(a,n,0)
#ifdef TARGET_SPIRV
#elif GLSL_VERSION>=310
#else
#endif
#define B4
#define C4
#define Q3
#define R3
#ifdef DISABLE_SHADER_STORAGE_BUFFERS
#define M5(g,v1,a) E4(d3,g,a)
#define J4(g,v1,a) ih(d3,g,a)
#define N5(g,v1,a) g5(d3,g,a)
#define K0(a,B0) p1(a,Y((B0)&Qc,(B0)>>Pc))
#define P5(a,B0) p1(a,Y((B0)&Qc,(B0)>>Pc)).xy
#else
#ifdef GL_ARB_shader_storage_buffer_object
#extension GL_ARB_shader_storage_buffer_object:require
#endif
#define M5(g,v1,a) layout(std430,binding=g)readonly buffer v1{a1 e2[];}a
#define J4(g,v1,a) layout(std430,binding=g)readonly buffer v1{X e2[];}a
#define N5(g,v1,a) layout(std430,binding=g)readonly buffer v1{f e2[];}a
#define Ua(g,v1,a) layout(std430,binding=g)buffer v1{uint e2[];}a
#define K0(a,B0) a.e2[B0]
#define P5(a,B0) a.e2[B0]
#define Rd(a,B0) a.e2[B0]
#define D7(a,B0,q) atomicMax(a.e2[B0],q)
#define Va(a,B0,q) atomicAdd(a.e2[B0],q)
#define jh(a,B0,q) atomicOr(a.e2[B0],q)
#endif
#ifdef PLS_IMPL_STORAGE_BUFFER
#define M1(a) void main(){Y G=ivec2(floor(c0));int F0=int(M8(uvec2(G),(l.o6+(Ba-1u))&~(Ba-1u)));
#define a2 }
#define S3 ,int F0
#define N1 ,F0
#ifdef TARGET_WGSL
#define G2(g,a) layout(std430,set=p3,binding=g)buffer a##Sd{uint e2[];}a
#elif defined(TARGET_SPIRV)
#define G2(g,a) layout(std430,set=p3,binding=g)coherent buffer a##Sd{uint e2[];}a
#else
#define G2(g,a) layout(std430,binding=g)coherent buffer a##Sd{uint e2[];}a
#endif
#define Wa G2
#define W2(h) h.e2[F0]
#define X2(h,D) h.e2[F0]=D
#define Xa(h) unpackUnorm4x8(W2(h))
#define Ya(h,D) X2(h,packUnorm4x8(D))
#define Y4(h,q) atomicMax(h.e2[F0],q)
#define Z4(h,q) atomicAdd(h.e2[F0],q)
#elif defined(PLS_IMPL_STORAGE_TEXTURE)||defined(USING_PLS_STORAGE_TEXTURES)
#ifdef GL_ARB_shader_image_load_store
#extension GL_ARB_shader_image_load_store:require
#endif
#define M1(a) void main(){Y G=ivec2(floor(c0));
#define a2 }
#define S3 ,Y G
#define N1 ,G
#ifdef TARGET_SPIRV
#define Wa(g,a) layout(set=p3,binding=g,rgba8)uniform mediump coherent image2D a
#define G2(g,a) layout(set=p3,binding=g,r32ui)uniform highp coherent uimage2D a
#define Za(g,a) layout(set=p3,binding=g,rgb10_a2)uniform mediump coherent image2D a
#else
#define Wa(g,a) layout(binding=g,rgba8)uniform mediump coherent image2D a
#define G2(g,a) layout(binding=g,r32ui)uniform highp coherent uimage2D a
#define Za(g,a) layout(binding=g,rgb10_a2)uniform mediump coherent image2D a;
#endif
#define W2(h) imageLoad(h,G).x
#define X2(h,D) imageStore(h,G,uvec4(D))
#define Xa(h) imageLoad(h,G)
#define Ya(h,D) imageStore(h,G,D)
#define Y4(h,q) imageAtomicMax(h,G,q)
#define Z4(h,q) imageAtomicAdd(h,G,q)
#else
#define M1(a) void main()
#define a2
#define S3
#define N1
#endif
#ifdef PLS_IMPL_ANGLE
#extension GL_ANGLE_shader_pixel_local_storage:require
#define J1
#define y0(g,a) layout(binding=g,rgba8)uniform mediump pixelLocalANGLE a
#define i1(g,a) layout(binding=g,r32ui)uniform highp upixelLocalANGLE a
#define K1
#define J0(h) pixelLocalLoadANGLE(h)
#define Y0(h) pixelLocalLoadANGLE(h).x
#define z0(h,D) pixelLocalStoreANGLE(h,D)
#define c1(h,D) pixelLocalStoreANGLE(h,uvec4(D))
#define x2(h)
#define f2(h)
#define y2
#define z2
#endif
#ifdef PLS_IMPL_EXT_NATIVE
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
#extension GL_EXT_shader_pixel_local_storage2:require
#else
#extension GL_EXT_shader_pixel_local_storage:require
#endif
#define J1 __pixel_localEXT S1{
#define y0(g,a) layout(rgba8)mediump vec4 a
#define ab(g,a) layout(rgb10_a2)mediump vec4 a
#define i1(g,a) layout(r32ui)highp uint a
#define K1 };
#define J0(h) h
#define Y0(h) h
#define z0(h,D) h=(D)
#define c1(h,D) h=(D)
#define x2(h) h=h
#define f2(h) h=h
#define y2
#define z2
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
#define r2(a) layout(location=0,rgba8)out i C1;M1(a)
#endif
#endif
#if defined(PLS_IMPL_STORAGE_TEXTURE)||defined(PLS_IMPL_STORAGE_BUFFER)
#define J1
#define K1
#define y0 Wa
#define i1 G2
#define ab Za
#define J0 Xa
#define z0 Ya
#define Y0 W2
#define c1 X2
#define x2(h)
#define f2(h)
#if defined(GL_ARB_fragment_shader_interlock)
#extension GL_ARB_fragment_shader_interlock:require
#define y2 beginInvocationInterlockARB()
#define z2 endInvocationInterlockARB()
#elif defined(GL_INTEL_fragment_shader_ordering)
#extension GL_INTEL_fragment_shader_ordering:require
#define y2 beginFragmentShaderOrderingINTEL()
#define z2
#else
#define y2
#define z2
#endif
#endif
#ifdef PLS_IMPL_SUBPASS_LOAD
#define J1
#define v4(g,a) layout(input_attachment_index=g,binding=g,set=p3)uniform mediump subpassInput E7##a
#define Td(g,a) layout(location=g)out mediump vec4 a
#define y0(g,a) v4(g,a);Td(g,a)
#define i1(g,a) layout(input_attachment_index=g,binding=g,set=p3)uniform highp usubpassInput E7##a;layout(location=g)out highp uvec4 a
#define K1
#define J0(h) subpassLoad(E7##h)
#define Y0(h) subpassLoad(E7##h).x
#define z0(h,D) h=(D)
#define c1(h,D) h.x=(D)
#define x2(h) z0(h,subpassLoad(E7##h))
#define f2(h) c1(h,subpassLoad(E7##h).x)
#define y2
#define z2
#endif
#ifdef PLS_IMPL_NONE
#define J1
#define y0(g,a) layout(location=g)out mediump vec4 a
#define i1(g,a) layout(location=g)out highp uvec4 a
#define K1
#define J0(h) vec4(0)
#define Y0(h) 0u
#define z0(h,D) h=(D)
#define c1(h,D) h.x=(D)
#define x2(h) h=vec4(0)
#define f2(h) h.x=0u
#define y2
#define z2
#endif
#ifndef v4
#define v4 y0
#endif
#ifdef TARGET_SPIRV
#define gl_VertexID gl_VertexIndex
#endif
#ifdef ENABLE_INSTANCE_INDEX
#ifdef TARGET_SPIRV
#define T8 gl_InstanceIndex
#else
#ifdef BASE_INSTANCE_UNIFORM_NAME
uniform highp int BASE_INSTANCE_UNIFORM_NAME;
#define T8 (gl_InstanceID+BASE_INSTANCE_UNIFORM_NAME)
#else
#define T8 (gl_InstanceID+gl_BaseInstance)
#endif
#endif
#else
#define T8 0
#endif
#define l6
#define x3
#define g7
#define x5
#define y1(a,f0,F,B,v) void main(){int B=gl_VertexID;int v=T8;
#define T7(a,f0,F,m1,g0,B,v) y1(a,f0,F,B,v)
#define J6(a,j3,k3,z3,A3,m1,g0,B) y1(a,j3,k3,B,v)
#define T(a,Z)
#define a0(a)
#define r(a,Z)
#define z1(P0) gl_Position=P0;}
#define c3(w1,a) layout(location=0)out w1 kh;void main()
#define v6(w1,a) c3(w1,a)
#define w6 gl_FrontFacing
#define K2(D) kh=D
#define c0 gl_FragCoord.xy
#define M6
#define V2
#if defined(PLS_IMPL_STORAGE_TEXTURE)||defined(PLS_IMPL_STORAGE_BUFFER)
#define Ud(F7,h,D) if(!(F7)){z0(h,D);}
#define Vd(F7,h,D) if(!(F7)){c1(h,D);}
#else
#define Ud(F7,h,D) z0(h,D);
#define Vd(F7,h,D) c1(h,D);
#endif
#ifndef r2
#define r2(a) layout(location=0)out i C1;M1(a)
#endif
#define m3 a2
#if defined(TARGET_SPIRV)&&!defined(TARGET_WGSL)
#ifdef MSAA_DST_COLOR
#define m5(a) layout(input_attachment_index=0,binding=F2,set=p3)uniform mediump subpassInputMS a
#define x6(a) qa(mat4(subpassLoad(a,0),subpassLoad(a,1),subpassLoad(a,2),subpassLoad(a,3)),gl_SampleMaskIn[0])
#else
#define m5(a) layout(input_attachment_index=0,binding=F2,set=p3)uniform mediump subpassInput a
#define x6(a) subpassLoad(a)
#endif
#else
#define m5(a) a3(d3,gg,a)
#define x6(a) texelFetch(a,ivec2(floor(c0.xy)),0)
#endif
#define N0(C,H) ((C)*(H))
precision highp float;precision highp int;
#if GLSL_VERSION<310
e i lh(uint u){X T1=X(u&0xffu,(u>>8)&0xffu,(u>>16)&0xffu,u>>24);return f(T1)*(1./255.);}
#define unpackUnorm4x8 lh
#endif
