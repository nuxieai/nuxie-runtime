#define xa
#ifndef GLSL_VERSION
#define GLSL_VERSION __VERSION__
#endif
#define c vec2
#define P vec3
#define c4 vec3
#define e vec4
#define d mediump float
#define C mediump vec2
#define v mediump vec3
#define i mediump vec4
#define k7 mediump mat3x3
#define l7 mediump mat2x3
#define S4 mediump mat4x4
#define e0 ivec2
#define l6 ivec4
#define O0 uvec2
#define N uvec4
#define R mediump uint
#define R4 bvec2
#define A6 bvec3
#define I7 bvec4
#define Y mat2
#define f
#define i1(x2) out x2
#define U6(x2) inout x2
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
#define H7(g,a) layout(binding=g,std140) uniform a{
#else
#define H7(g,a) layout(std140) uniform a{
#endif
#define d9(a) }a;
#define c1(a)
#define K(g,j0,a) layout(location=g) in j0 a
#define d1
#define L(g9,D,a,j0)
#ifdef VERTEX
#if GLSL_VERSION>=310
#define W(g,j0,a) layout(location=g) out j0 a
#else
#define W(g,j0,a) out j0 a
#endif
#else
#if GLSL_VERSION>=310
#define W(g,j0,a) layout(location=g) in j0 a
#else
#define W(g,j0,a) in j0 a
#endif
#endif
#define a3 flat
#define l2
#define d2
#ifdef TARGET_SPIRV
#define E0
#else
#ifdef GL_NV_shader_noperspective_interpolation
#extension GL_NV_shader_noperspective_interpolation:require
#define E0 noperspective
#else
#define E0
#endif
#endif
#ifdef VERTEX
#define j4
#define k4
#endif
#ifdef FRAGMENT
#define O3
#define P3
#endif
#define v5
#define w5
#ifdef TARGET_SPIRV
#define V4(c0,g,a) layout(set=c0,binding=g) uniform highp utexture2D a
#define B6(c0,g,a) layout(set=c0,binding=g) uniform highp texture2D a
#define i3(c0,g,a) layout(set=c0,binding=g) uniform mediump texture2D a
#define B5(c0,g,a) layout(binding=g) uniform mediump texture2D a
#if defined(FRAGMENT)&&defined(RENDER_MODE_DEPTH_STENCIL)
#endif
#elif GLSL_VERSION>=310
#define V4(c0,g,a) layout(binding=g) uniform highp usampler2D a
#define B6(c0,g,a) layout(binding=g) uniform highp sampler2D a
#define i3(c0,g,a) layout(binding=g) uniform mediump sampler2D a
#define B5(c0,g,a) layout(binding=g) uniform mediump sampler2D a
#else
#define V4(c0,g,a) uniform highp usampler2D a
#define B6(c0,g,a) uniform highp sampler2D a
#define i3(c0,g,a) uniform mediump sampler2D a
#define B5(c0,g,a) uniform mediump sampler2D a
#endif
#ifdef TARGET_SPIRV
#define C6(c0,g,a) layout(set=c0,binding=g) uniform mediump sampler a;
#ifdef USE_WEBGPU_SAMPLERS
#define o4(J7,a) layout(set=Ig,binding=J7) uniform mediump sampler a;
#define m4(a) C6(r5,Hg,a)
#else
#define o4(J7,a) layout(set=l3,binding=J7) uniform mediump sampler a;
#define m4(a) C6(r5,l4,a)
#endif
#define I5(a,p,m) texture(sampler2D(a,p),m)
#define o2(a,p,m,Y0) textureLod(sampler2D(a,p),m,Y0)
#define J5(a,p,m,Y1) texture(sampler2D(a,p),m,Y1)
#if defined(FRAGMENT)&&defined(RENDER_MODE_DEPTH_STENCIL)&&defined(MSAA_DST_COLOR)
#extension GL_OES_sample_variables:require
#endif
#else
#define o4(J7,a)
#define C6(c0,g,a)
#define m4(a)
#define I5(a,p,m) texture(a,m)
#define o2(a,p,m,Y0) textureLod(a,m,Y0)
#define J5(a,p,m,Y1) texture(a,m,Y1)
#endif
#define v8(q0,p,m) I5(q0,p,m)
#define f7(q0,p,m,Y0) o2(q0,p,m,Y0)
#define K7(q0,p,m,Y1) J5(q0,p,m,Y1)
#define p6(c0,g,a) B5(c0,g,a)
#define j7(a,p,F,D6,i9,Y0) o2(a,p,c(F,i9),Y0)
#define Hh(c0,g,a) V4(c0,g,a)
#define S3
#define k1
#define p1(a,m) texelFetch(a,m,0)
#ifdef TARGET_SPIRV
#elif GLSL_VERSION>=310
#else
#endif
#define P4
#define Q4
#define f4
#define g4
#ifdef DISABLE_SHADER_STORAGE_BUFFERS
#define X5(g,D1,a) V4(l3,g,a)
#define W4(g,D1,a) Hh(l3,g,a)
#define Y5(g,D1,a) B6(l3,g,a)
#define p0(a,D0) p1(a,e0((D0)&od,(D0)>>nd))
#define k5(a,D0) p1(a,e0((D0)&od,(D0)>>nd)).xy
#else
#ifdef GL_ARB_shader_storage_buffer_object
#extension GL_ARB_shader_storage_buffer_object:require
#endif
#define X5(g,D1,a) layout(std430,binding=g) readonly buffer D1{O0 j2[];}a
#define W4(g,D1,a) layout(std430,binding=g) readonly buffer D1{N j2[];}a
#define Y5(g,D1,a) layout(std430,binding=g) readonly buffer D1{e j2[];}a
#define mb(g,D1,a) layout(std430,binding=g) buffer D1{uint j2[];}a
#define p0(a,D0) a.j2[D0]
#define k5(a,D0) a.j2[D0]
#define fe(a,D0) a.j2[D0]
#define M7(a,D0,F) atomicMax(a.j2[D0],F)
#define nb(a,D0,F) atomicAdd(a.j2[D0],F)
#define Ih(a,D0,F) atomicOr(a.j2[D0],F)
#endif
#ifdef PLS_IMPL_STORAGE_BUFFER
#define T1(a) void main(){e0 H=ivec2(floor(f0));int I0=int(c9(uvec2(H),(j.z6+(Ta-1u))&~(Ta-1u)));
#define g2 }
#define h4 ,int I0
#define U1 ,I0
#ifdef TARGET_WGSL
#define M2(g,a) layout(std430,set=D3,binding=g) buffer a##ge{uint j2[];}a
#elif defined(TARGET_SPIRV)
#define M2(g,a) layout(std430,set=D3,binding=g) coherent buffer a##ge{uint j2[];}a
#else
#define M2(g,a) layout(std430,binding=g) coherent buffer a##ge{uint j2[];}a
#endif
#define ob M2
#define f3(h) h.j2[I0]
#define g3(h,E) h.j2[I0]=E
#define pb(h) unpackUnorm4x8(f3(h))
#define qb(h,E) g3(h,packUnorm4x8(E))
#define o5(h,F) atomicMax(h.j2[I0],F)
#define p5(h,F) atomicAdd(h.j2[I0],F)
#elif defined(PLS_IMPL_STORAGE_TEXTURE)||defined(USING_PLS_STORAGE_TEXTURES)
#ifdef GL_ARB_shader_image_load_store
#extension GL_ARB_shader_image_load_store:require
#endif
#define T1(a) void main(){e0 H=ivec2(floor(f0));
#define g2 }
#define h4 ,e0 H
#define U1 ,H
#ifdef TARGET_SPIRV
#define ob(g,a) layout(set=D3,binding=g,rgba8) uniform mediump coherent image2D a
#define M2(g,a) layout(set=D3,binding=g,r32ui) uniform highp coherent uimage2D a
#define rb(g,a) layout(set=D3,binding=g,rgb10_a2) uniform mediump coherent image2D a
#else
#define ob(g,a) layout(binding=g,rgba8) uniform mediump coherent image2D a
#define M2(g,a) layout(binding=g,r32ui) uniform highp coherent uimage2D a
#define rb(g,a) layout(binding=g,rgb10_a2) uniform mediump coherent image2D a;
#endif
#define f3(h) imageLoad(h,H).x
#define g3(h,E) imageStore(h,H,uvec4(E))
#define pb(h) imageLoad(h,H)
#define qb(h,E) imageStore(h,H,E)
#define o5(h,F) imageAtomicMax(h,H,F)
#define p5(h,F) imageAtomicAdd(h,H,F)
#else
#define T1(a) void main()
#define g2
#define h4
#define U1
#endif
#ifdef PLS_IMPL_ANGLE
#extension GL_ANGLE_shader_pixel_local_storage:require
#define Q1
#define A0(g,a) layout(binding=g,rgba8) uniform mediump pixelLocalANGLE a
#define o1(g,a) layout(binding=g,r32ui) uniform highp upixelLocalANGLE a
#define R1
#define N0(h) pixelLocalLoadANGLE(h)
#define h1(h) pixelLocalLoadANGLE(h).x
#define B0(h,E) pixelLocalStoreANGLE(h,E)
#define j1(h,E) pixelLocalStoreANGLE(h,uvec4(E))
#define E2(h)
#define k2(h)
#define F2
#define G2
#endif
#ifdef PLS_IMPL_EXT_NATIVE
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
#extension GL_EXT_shader_pixel_local_storage2:require
#else
#extension GL_EXT_shader_pixel_local_storage:require
#endif
#define Q1 __pixel_localEXT Z1{
#define A0(g,a) layout(rgba8) mediump vec4 a
#define sb(g,a) layout(rgb10_a2) mediump vec4 a
#define o1(g,a) layout(r32ui) highp uint a
#define R1 };
#define N0(h) h
#define h1(h) h
#define B0(h,E) h=(E)
#define j1(h,E) h=(E)
#define E2(h) h=h
#define k2(h) h=h
#define F2
#define G2
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
#define A2(a) layout(location=0,rgba8) out i J1;T1(a)
#endif
#endif
#if defined(PLS_IMPL_STORAGE_TEXTURE)||defined(PLS_IMPL_STORAGE_BUFFER)
#define Q1
#define R1
#define A0 ob
#define o1 M2
#define sb rb
#define N0 pb
#define B0 qb
#define h1 f3
#define j1 g3
#define E2(h)
#define k2(h)
#if defined(GL_ARB_fragment_shader_interlock)
#extension GL_ARB_fragment_shader_interlock:require
#define F2 beginInvocationInterlockARB()
#define G2 endInvocationInterlockARB()
#elif defined(GL_INTEL_fragment_shader_ordering)
#extension GL_INTEL_fragment_shader_ordering:require
#define F2 beginFragmentShaderOrderingINTEL()
#define G2
#else
#define F2
#define G2
#endif
#endif
#ifdef PLS_IMPL_SUBPASS_LOAD
#define Q1
#define I4(g,a) layout(input_attachment_index=g,binding=g,set=D3) uniform mediump subpassInput N7##a
#define he(g,a) layout(location=g) out mediump vec4 a
#define A0(g,a) I4(g,a);he(g,a)
#define o1(g,a) layout(input_attachment_index=g,binding=g,set=D3) uniform highp usubpassInput N7##a;layout(location=g) out highp uvec4 a
#define R1
#define N0(h) subpassLoad(N7##h)
#define h1(h) subpassLoad(N7##h).x
#define B0(h,E) h=(E)
#define j1(h,E) h.x=(E)
#define E2(h) B0(h,subpassLoad(N7##h))
#define k2(h) j1(h,subpassLoad(N7##h).x)
#define F2
#define G2
#endif
#ifdef PLS_IMPL_NONE
#define Q1
#define A0(g,a) layout(location=g) out mediump vec4 a
#define o1(g,a) layout(location=g) out highp uvec4 a
#define R1
#define N0(h) vec4(0)
#define h1(h) 0u
#define B0(h,E) h=(E)
#define j1(h,E) h.x=(E)
#define E2(h) h=vec4(0)
#define k2(h) h.x=0u
#define F2
#define G2
#endif
#ifndef I4
#define I4 A0
#endif
#ifdef TARGET_SPIRV
#define tb gl_VertexIndex
#ifdef ENABLE_INSTANCE_INDEX
#define O7 gl_InstanceIndex
#else
#define O7 0
#endif
#else
#ifdef ENABLE_BASE_VERTEX
uniform highp int VE;
#define tb (gl_VertexID+VE)
#else
#define tb gl_VertexID
#endif
#ifdef ENABLE_INSTANCE_INDEX
#ifdef BASE_INSTANCE_UNIFORM_NAME
uniform highp int BASE_INSTANCE_UNIFORM_NAME;
#define O7 (gl_InstanceID+BASE_INSTANCE_UNIFORM_NAME)
#else
#define O7 (gl_InstanceID+gl_BaseInstance)
#endif
#else
#define O7 0
#endif
#endif
#define v6
#define H3
#define p7
#define Y4
#define r1(a,d0,D,p3,E6) void main(){int p3=tb;int E6=O7;
#define g8(a,d0,D,z1,h0,p3,E6) r1(a,d0,D,p3,E6)
#define S6(a,w3,x3,K3,L3,z1,h0,p3) r1(a,w3,x3,p3,E6)
#define T(a,j0)
#define Z(a)
#define q(a,j0)
#define v1(U0) gl_Position=U0;}
#define j3(E1,a) layout(location=0) out E1 Jh;void main()
#define F6(E1,a) j3(E1,a)
#define G6 gl_FrontFacing
#define Q2(E) Jh=E
#define f0 gl_FragCoord.xy
#define V6
#define e3
#if defined(PLS_IMPL_STORAGE_TEXTURE)||defined(PLS_IMPL_STORAGE_BUFFER)
#define ie(P7,h,E) if(!(P7)){B0(h,E);}
#define je(P7,h,E) if(!(P7)){j1(h,E);}
#else
#define ie(P7,h,E) B0(h,E);
#define je(P7,h,E) j1(h,E);
#endif
#ifndef A2
#define A2(a) layout(location=0) out i J1;T1(a)
#endif
#define A3 g2
#if defined(TARGET_SPIRV)&&!defined(TARGET_WGSL)
#ifdef MSAA_DST_COLOR
#define C5(a) layout(input_attachment_index=0,binding=L2,set=D3) uniform mediump subpassInputMS a
#define H6(a) Ha(mat4(subpassLoad(a,0),subpassLoad(a,1),subpassLoad(a,2),subpassLoad(a,3)),gl_SampleMaskIn[0])
#else
#define C5(a) layout(input_attachment_index=0,binding=L2,set=D3) uniform mediump subpassInput a
#define H6(a) subpassLoad(a)
#endif
#else
#define C5(a) i3(l3,Gg,a)
#define H6(a) texelFetch(a,ivec2(floor(f0.xy)),0)
#endif
#define K0(B,J) ((B)*(J))
precision highp float;precision highp int;
#if GLSL_VERSION<310
f i Kh(uint u){N q1=N(u&0xffu,(u>>8)&0xffu,(u>>16)&0xffu,u>>24);return e(q1)*(1./255.);}
#define unpackUnorm4x8 Kh
#endif
