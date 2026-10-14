#define bb
#ifndef GLSL_VERSION
#define GLSL_VERSION __VERSION__
#endif
#define c vec2
#define M vec3
#define i4 vec3
#define f vec4
#define d mediump float
#define D mediump vec2
#define v mediump vec3
#define i mediump vec4
#define A7 mediump mat3x3
#define B7 mediump mat2x3
#define d5 mediump mat4x4
#define g0 ivec2
#define x6 ivec4
#define R0 uvec2
#define O uvec4
#define P mediump uint
#define c5 bvec2
#define Q6 bvec3
#define e8 bvec4
#define X mat2
#define e
#define k1(D2) out D2
#define j7(D2) inout D2
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
#define d8(g,a) layout(binding=g,std140) uniform a{
#else
#define d8(g,a) layout(std140) uniform a{
#endif
#define M9(a) }a;
#define d1(a)
#define K(g,k0,a) layout(location=g) in k0 a
#define e1
#define L(P9,B,a,k0)
#ifdef VERTEX
#if GLSL_VERSION>=310
#define W(g,k0,a) layout(location=g) out k0 a
#else
#define W(g,k0,a) out k0 a
#endif
#else
#if GLSL_VERSION>=310
#define W(g,k0,a) layout(location=g) in k0 a
#else
#define W(g,k0,a) in k0 a
#endif
#endif
#define h3 flat
#define v2
#define k2
#ifdef TARGET_SPIRV
#define F0
#else
#ifdef GL_NV_shader_noperspective_interpolation
#extension GL_NV_shader_noperspective_interpolation:require
#define F0 noperspective
#else
#define F0
#endif
#endif
#ifdef VERTEX
#define q4
#define r4
#endif
#ifdef FRAGMENT
#define V3
#define W3
#endif
#define C5
#define D5
#ifdef TARGET_SPIRV
#define i5(e0,g,a) layout(set=e0,binding=g) uniform highp utexture2D a
#define R6(e0,g,a) layout(set=e0,binding=g) uniform highp texture2D a
#define q3(e0,g,a) layout(set=e0,binding=g) uniform mediump texture2D a
#define N5(e0,g,a) layout(binding=g) uniform mediump texture2D a
#if defined(FRAGMENT)&&defined(RENDER_MODE_DEPTH_STENCIL)
#endif
#elif GLSL_VERSION>=310
#define i5(e0,g,a) layout(binding=g) uniform highp usampler2D a
#define R6(e0,g,a) layout(binding=g) uniform highp sampler2D a
#define q3(e0,g,a) layout(binding=g) uniform mediump sampler2D a
#define N5(e0,g,a) layout(binding=g) uniform mediump sampler2D a
#else
#define i5(e0,g,a) uniform highp usampler2D a
#define R6(e0,g,a) uniform highp sampler2D a
#define q3(e0,g,a) uniform mediump sampler2D a
#define N5(e0,g,a) uniform mediump sampler2D a
#endif
#ifdef TARGET_SPIRV
#define S6(e0,g,a) layout(set=e0,binding=g) uniform mediump sampler a;
#ifdef USE_WEBGPU_SAMPLERS
#define a4(R5,a) layout(set=Dh,binding=R5) uniform mediump sampler a;
#define J6 a4
#define w4(a) S6(B5,Ch,a)
#else
#define a4(R5,a) layout(set=r3,binding=R5) uniform mediump sampler a;
#define J6 a4
#define w4(a) S6(B5,v4,a)
#endif
#define S5(a,p,o) texture(sampler2D(a,p),o)
#define n2(a,p,o,a1) textureLod(sampler2D(a,p),o,a1)
#define T5(a,p,o,f2) texture(sampler2D(a,p),o,f2)
#if defined(FRAGMENT)&&defined(RENDER_MODE_DEPTH_STENCIL)&&defined(MSAA_DST_COLOR)
#extension GL_OES_sample_variables:require
#endif
#else
#define a4(R5,a)
#define J6(R5,a)
#define S6(e0,g,a)
#define w4(a)
#define S5(a,p,o) texture(a,o)
#define n2(a,p,o,a1) textureLod(a,o,a1)
#define T5(a,p,o,f2) texture(a,o,f2)
#endif
#define U8(q0,p,o) S5(q0,p,o)
#define E5(q0,p,o,a1) n2(q0,p,o,a1)
#define f8(q0,p,o,f2) T5(q0,p,o,f2)
#define I6(e0,g,a) N5(e0,g,a)
#define z7(a,p,E,T6,R9,a1) n2(a,p,c(E,R9),a1)
#define Ei(e0,g,a) i5(e0,g,a)
#define c4
#define m1
#define q1(a,o) texelFetch(a,o,0)
#ifdef TARGET_SPIRV
#elif GLSL_VERSION>=310
#else
#endif
#define Y4
#define Z4
#define k4
#define l4
#ifdef DISABLE_SHADER_STORAGE_BUFFERS
#define j6(g,G1,a) i5(r3,g,a)
#define j5(g,G1,a) Ei(r3,g,a)
#define k6(g,G1,a) R6(r3,g,a)
#define p0(a,E0) q1(a,g0((E0)&Ud,(E0)>>Td))
#define x5(a,E0) q1(a,g0((E0)&Ud,(E0)>>Td)).xy
#else
#ifdef GL_ARB_shader_storage_buffer_object
#extension GL_ARB_shader_storage_buffer_object:require
#endif
#define j6(g,G1,a) layout(std430,binding=g) readonly buffer G1{R0 r2[];}a
#define j5(g,G1,a) layout(std430,binding=g) readonly buffer G1{O r2[];}a
#define k6(g,G1,a) layout(std430,binding=g) readonly buffer G1{f r2[];}a
#define Zb(g,G1,a) layout(std430,binding=g) buffer G1{uint r2[];}a
#define p0(a,E0) a.r2[E0]
#define x5(a,E0) a.r2[E0]
#define Oe(a,E0) a.r2[E0]
#define h8(a,E0,E) atomicMax(a.r2[E0],E)
#define ac(a,E0,E) atomicAdd(a.r2[E0],E)
#define Fi(a,E0,E) atomicOr(a.r2[E0],E)
#endif
#ifdef PLS_IMPL_STORAGE_BUFFER
#define X1(a) void main(){g0 G=ivec2(floor(d0));int L0=int(L9(uvec2(G),(j.P6+(Bb-1u))&~(Bb-1u)));
#define o2 }
#define m4 ,int L0
#define Y1 ,L0
#ifdef TARGET_WGSL
#define U2(g,a) layout(std430,set=K3,binding=g) buffer a##Pe{uint r2[];}a
#elif defined(TARGET_SPIRV)
#define U2(g,a) layout(std430,set=K3,binding=g) coherent buffer a##Pe{uint r2[];}a
#else
#define U2(g,a) layout(std430,binding=g) coherent buffer a##Pe{uint r2[];}a
#endif
#define bc U2
#define n3(h) h.r2[L0]
#define o3(h,C) h.r2[L0]=C
#define cc(h) unpackUnorm4x8(n3(h))
#define dc(h,C) o3(h,packUnorm4x8(C))
#define z5(h,E) atomicMax(h.r2[L0],E)
#define A5(h,E) atomicAdd(h.r2[L0],E)
#elif defined(PLS_IMPL_STORAGE_TEXTURE)||defined(USING_PLS_STORAGE_TEXTURES)
#ifdef GL_ARB_shader_image_load_store
#extension GL_ARB_shader_image_load_store:require
#endif
#define X1(a) void main(){g0 G=ivec2(floor(d0));
#define o2 }
#define m4 ,g0 G
#define Y1 ,G
#ifdef TARGET_SPIRV
#define bc(g,a) layout(set=K3,binding=g,rgba8) uniform mediump coherent image2D a
#define U2(g,a) layout(set=K3,binding=g,r32ui) uniform highp coherent uimage2D a
#define ec(g,a) layout(set=K3,binding=g,rgb10_a2) uniform mediump coherent image2D a
#else
#define bc(g,a) layout(binding=g,rgba8) uniform mediump coherent image2D a
#define U2(g,a) layout(binding=g,r32ui) uniform highp coherent uimage2D a
#define ec(g,a) layout(binding=g,rgb10_a2) uniform mediump coherent image2D a;
#endif
#define n3(h) imageLoad(h,G).x
#define o3(h,C) imageStore(h,G,uvec4(C))
#define cc(h) imageLoad(h,G)
#define dc(h,C) imageStore(h,G,C)
#define z5(h,E) imageAtomicMax(h,G,E)
#define A5(h,E) imageAtomicAdd(h,G,E)
#else
#define X1(a) void main()
#define o2
#define m4
#define Y1
#endif
#ifdef PLS_IMPL_ANGLE
#extension GL_ANGLE_shader_pixel_local_storage:require
#define U1
#define C0(g,a) layout(binding=g,rgba8) uniform mediump pixelLocalANGLE a
#define p1(g,a) layout(binding=g,r32ui) uniform highp upixelLocalANGLE a
#define V1
#define Q0(h) pixelLocalLoadANGLE(h)
#define j1(h) pixelLocalLoadANGLE(h).x
#define y0(h,C) pixelLocalStoreANGLE(h,C)
#define l1(h,C) pixelLocalStoreANGLE(h,uvec4(C))
#define M2(h)
#define g2(h)
#define N2
#define O2
#endif
#ifdef PLS_IMPL_EXT_NATIVE
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
#extension GL_EXT_shader_pixel_local_storage2:require
#else
#extension GL_EXT_shader_pixel_local_storage:require
#endif
#define U1 __pixel_localEXT h2{
#define C0(g,a) layout(rgba8) mediump vec4 a
#define fc(g,a) layout(rgb10_a2) mediump vec4 a
#define p1(g,a) layout(r32ui) highp uint a
#define V1 };
#define Q0(h) h
#define j1(h) h
#define y0(h,C) h=(C)
#define l1(h,C) h=(C)
#define M2(h) h=h
#define g2(h) h=h
#define N2
#define O2
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
#define G2(a) layout(location=0,rgba8) out i L1;X1(a)
#endif
#endif
#if defined(PLS_IMPL_STORAGE_TEXTURE)||defined(PLS_IMPL_STORAGE_BUFFER)
#define U1
#define V1
#define C0 bc
#define p1 U2
#define fc ec
#define Q0 cc
#define y0 dc
#define j1 n3
#define l1 o3
#define M2(h)
#define g2(h)
#if defined(GL_ARB_fragment_shader_interlock)
#extension GL_ARB_fragment_shader_interlock:require
#define N2 beginInvocationInterlockARB()
#define O2 endInvocationInterlockARB()
#elif defined(GL_INTEL_fragment_shader_ordering)
#extension GL_INTEL_fragment_shader_ordering:require
#define N2 beginFragmentShaderOrderingINTEL()
#define O2
#else
#define N2
#define O2
#endif
#endif
#ifdef PLS_IMPL_SUBPASS_LOAD
#define U1
#define P4(g,a) layout(input_attachment_index=g,binding=g,set=K3) uniform mediump subpassInput i8##a
#define Qe(g,a) layout(location=g) out mediump vec4 a
#define C0(g,a) P4(g,a);Qe(g,a)
#define p1(g,a) layout(input_attachment_index=g,binding=g,set=K3) uniform highp usubpassInput i8##a;layout(location=g) out highp uvec4 a
#define V1
#define Q0(h) subpassLoad(i8##h)
#define j1(h) subpassLoad(i8##h).x
#define y0(h,C) h=(C)
#define l1(h,C) h.x=(C)
#define M2(h) y0(h,subpassLoad(i8##h))
#define g2(h) l1(h,subpassLoad(i8##h).x)
#define N2
#define O2
#endif
#ifdef PLS_IMPL_NONE
#define U1
#define C0(g,a) layout(location=g) out mediump vec4 a
#define p1(g,a) layout(location=g) out highp uvec4 a
#define V1
#define Q0(h) vec4(0)
#define j1(h) 0u
#define y0(h,C) h=(C)
#define l1(h,C) h.x=(C)
#define M2(h) h=vec4(0)
#define g2(h) h.x=0u
#define N2
#define O2
#endif
#ifndef P4
#define P4 C0
#endif
#ifdef TARGET_SPIRV
#define gc gl_VertexIndex
#ifdef ENABLE_INSTANCE_INDEX
#define j8 gl_InstanceIndex
#else
#define j8 0
#endif
#else
#ifdef ENABLE_BASE_VERTEX
uniform highp int WE;
#define gc (gl_VertexID+WE)
#else
#define gc gl_VertexID
#endif
#ifdef ENABLE_INSTANCE_INDEX
#ifdef BASE_INSTANCE_UNIFORM_NAME
uniform highp int BASE_INSTANCE_UNIFORM_NAME;
#define j8 (gl_InstanceID+BASE_INSTANCE_UNIFORM_NAME)
#else
#define j8 (gl_InstanceID+gl_BaseInstance)
#endif
#else
#define j8 0
#endif
#endif
#define M6
#define Q3
#define F7
#define h5
#define w1(a,f0,B,L2,C6) void main(){int L2=gc;int C6=j8;
#define D8(a,f0,B,C1,j0,L2,C6) w1(a,f0,B,L2,C6)
#define h7(a,C3,D3,T3,j3,C1,j0,L2) w1(a,C3,D3,L2,C6)
#define V(a,k0)
#define Z(a)
#define q(a,k0)
#define x1(X0) gl_Position=X0;}
#define V2(H1,a) layout(location=0) out H1 Gi;void main()
#define U6(H1,a) V2(H1,a)
#define V6 gl_FrontFacing
#define K2(C) Gi=C
#define d0 gl_FragCoord.xy
#define k7
#define m3
#if defined(PLS_IMPL_STORAGE_TEXTURE)||defined(PLS_IMPL_STORAGE_BUFFER)
#define Re(k8,h,C) if(!(k8)){y0(h,C);}
#define Se(k8,h,C) if(!(k8)){l1(h,C);}
#else
#define Re(k8,h,C) y0(h,C);
#define Se(k8,h,C) l1(h,C);
#endif
#ifndef G2
#define G2(a) layout(location=0) out i L1;X1(a)
#endif
#define E3 o2
#if defined(TARGET_SPIRV)&&!defined(TARGET_WGSL)
#ifdef MSAA_DST_COLOR
#define O5(a) layout(input_attachment_index=0,binding=T2,set=K3) uniform mediump subpassInputMS a
#define M5(a) mb(mat4(subpassLoad(a,0),subpassLoad(a,1),subpassLoad(a,2),subpassLoad(a,3)),gl_SampleMaskIn[0])
#else
#define O5(a) layout(input_attachment_index=0,binding=T2,set=K3) uniform mediump subpassInput a
#define M5(a) subpassLoad(a)
#endif
#else
#define O5(a) q3(r3,Bh,a)
#define M5(a) texelFetch(a,ivec2(floor(d0.xy)),0)
#endif
#define B0(A,J) ((A)*(J))
precision highp float;precision highp int;
#if GLSL_VERSION<310
e i Hi(uint u){O r1=O(u&0xffu,(u>>8)&0xffu,(u>>16)&0xffu,u>>24);return f(r1)*(1./255.);}
#define unpackUnorm4x8 Hi
#endif
