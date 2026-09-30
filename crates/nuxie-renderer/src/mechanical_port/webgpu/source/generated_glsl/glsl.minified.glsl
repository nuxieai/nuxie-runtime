#define ya
#ifndef GLSL_VERSION
#define GLSL_VERSION __VERSION__
#endif
#define c vec2
#define P vec3
#define d4 vec3
#define f vec4
#define d mediump float
#define C mediump vec2
#define v mediump vec3
#define i mediump vec4
#define l7 mediump mat3x3
#define m7 mediump mat2x3
#define T4 mediump mat4x4
#define e0 ivec2
#define n6 ivec4
#define O0 uvec2
#define N uvec4
#define R mediump uint
#define S4 bvec2
#define C6 bvec3
#define J7 bvec4
#define Y mat2
#define e
#define i1(x2) out x2
#define W6(x2) inout x2
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
#define I7(g,a) layout(binding=g,std140) uniform a{
#else
#define I7(g,a) layout(std140) uniform a{
#endif
#define f9(a) }a;
#define c1(a)
#define K(g,j0,a) layout(location=g) in j0 a
#define d1
#define L(i9,D,a,j0)
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
#define e2
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
#define k4
#define l4
#endif
#ifdef FRAGMENT
#define O3
#define P3
#endif
#define y5
#define z5
#ifdef TARGET_SPIRV
#define W4(c0,g,a) layout(set=c0,binding=g) uniform highp utexture2D a
#define D6(c0,g,a) layout(set=c0,binding=g) uniform highp texture2D a
#define i3(c0,g,a) layout(set=c0,binding=g) uniform mediump texture2D a
#define E5(c0,g,a) layout(binding=g) uniform mediump texture2D a
#if defined(FRAGMENT)&&defined(RENDER_MODE_DEPTH_STENCIL)
#endif
#elif GLSL_VERSION>=310
#define W4(c0,g,a) layout(binding=g) uniform highp usampler2D a
#define D6(c0,g,a) layout(binding=g) uniform highp sampler2D a
#define i3(c0,g,a) layout(binding=g) uniform mediump sampler2D a
#define E5(c0,g,a) layout(binding=g) uniform mediump sampler2D a
#else
#define W4(c0,g,a) uniform highp usampler2D a
#define D6(c0,g,a) uniform highp sampler2D a
#define i3(c0,g,a) uniform mediump sampler2D a
#define E5(c0,g,a) uniform mediump sampler2D a
#endif
#ifdef TARGET_SPIRV
#define E6(c0,g,a) layout(set=c0,binding=g) uniform mediump sampler a;
#ifdef USE_WEBGPU_SAMPLERS
#define p4(K7,a) layout(set=Sg,binding=K7) uniform mediump sampler a;
#define n4(a) E6(x5,Rg,a)
#else
#define p4(K7,a) layout(set=l3,binding=K7) uniform mediump sampler a;
#define n4(a) E6(x5,m4,a)
#endif
#define L5(a,p,m) texture(sampler2D(a,p),m)
#define o2(a,p,m,Y0) textureLod(sampler2D(a,p),m,Y0)
#define M5(a,p,m,Y1) texture(sampler2D(a,p),m,Y1)
#if defined(FRAGMENT)&&defined(RENDER_MODE_DEPTH_STENCIL)&&defined(MSAA_DST_COLOR)
#extension GL_OES_sample_variables:require
#endif
#else
#define p4(K7,a)
#define E6(c0,g,a)
#define n4(a)
#define L5(a,p,m) texture(a,m)
#define o2(a,p,m,Y0) textureLod(a,m,Y0)
#define M5(a,p,m,Y1) texture(a,m,Y1)
#endif
#define x8(q0,p,m) L5(q0,p,m)
#define j6(q0,p,m,Y0) o2(q0,p,m,Y0)
#define L7(q0,p,m,Y1) M5(q0,p,m,Y1)
#define r6(c0,g,a) E5(c0,g,a)
#define k7(a,p,F,F6,k9,Y0) o2(a,p,c(F,k9),Y0)
#define Rh(c0,g,a) W4(c0,g,a)
#define S3
#define k1
#define p1(a,m) texelFetch(a,m,0)
#ifdef TARGET_SPIRV
#elif GLSL_VERSION>=310
#else
#endif
#define Q4
#define R4
#define g4
#define h4
#ifdef DISABLE_SHADER_STORAGE_BUFFERS
#define a6(g,E1,a) W4(l3,g,a)
#define X4(g,E1,a) Rh(l3,g,a)
#define c6(g,E1,a) D6(l3,g,a)
#define p0(a,D0) p1(a,e0((D0)&pd,(D0)>>od))
#define m5(a,D0) p1(a,e0((D0)&pd,(D0)>>od)).xy
#else
#ifdef GL_ARB_shader_storage_buffer_object
#extension GL_ARB_shader_storage_buffer_object:require
#endif
#define a6(g,E1,a) layout(std430,binding=g) readonly buffer E1{O0 k2[];}a
#define X4(g,E1,a) layout(std430,binding=g) readonly buffer E1{N k2[];}a
#define c6(g,E1,a) layout(std430,binding=g) readonly buffer E1{f k2[];}a
#define nb(g,E1,a) layout(std430,binding=g) buffer E1{uint k2[];}a
#define p0(a,D0) a.k2[D0]
#define m5(a,D0) a.k2[D0]
#define ge(a,D0) a.k2[D0]
#define N7(a,D0,F) atomicMax(a.k2[D0],F)
#define ob(a,D0,F) atomicAdd(a.k2[D0],F)
#define Sh(a,D0,F) atomicOr(a.k2[D0],F)
#endif
#ifdef PLS_IMPL_STORAGE_BUFFER
#define T1(a) void main(){e0 H=ivec2(floor(f0));int K0=int(e9(uvec2(H),(j.B6+(Ua-1u))&~(Ua-1u)));
#define h2 }
#define i4 ,int K0
#define U1 ,K0
#ifdef TARGET_WGSL
#define M2(g,a) layout(std430,set=D3,binding=g) buffer a##he{uint k2[];}a
#elif defined(TARGET_SPIRV)
#define M2(g,a) layout(std430,set=D3,binding=g) coherent buffer a##he{uint k2[];}a
#else
#define M2(g,a) layout(std430,binding=g) coherent buffer a##he{uint k2[];}a
#endif
#define pb M2
#define f3(h) h.k2[K0]
#define g3(h,E) h.k2[K0]=E
#define qb(h) unpackUnorm4x8(f3(h))
#define rb(h,E) g3(h,packUnorm4x8(E))
#define q5(h,F) atomicMax(h.k2[K0],F)
#define r5(h,F) atomicAdd(h.k2[K0],F)
#elif defined(PLS_IMPL_STORAGE_TEXTURE)||defined(USING_PLS_STORAGE_TEXTURES)
#ifdef GL_ARB_shader_image_load_store
#extension GL_ARB_shader_image_load_store:require
#endif
#define T1(a) void main(){e0 H=ivec2(floor(f0));
#define h2 }
#define i4 ,e0 H
#define U1 ,H
#ifdef TARGET_SPIRV
#define pb(g,a) layout(set=D3,binding=g,rgba8) uniform mediump coherent image2D a
#define M2(g,a) layout(set=D3,binding=g,r32ui) uniform highp coherent uimage2D a
#define sb(g,a) layout(set=D3,binding=g,rgb10_a2) uniform mediump coherent image2D a
#else
#define pb(g,a) layout(binding=g,rgba8) uniform mediump coherent image2D a
#define M2(g,a) layout(binding=g,r32ui) uniform highp coherent uimage2D a
#define sb(g,a) layout(binding=g,rgb10_a2) uniform mediump coherent image2D a;
#endif
#define f3(h) imageLoad(h,H).x
#define g3(h,E) imageStore(h,H,uvec4(E))
#define qb(h) imageLoad(h,H)
#define rb(h,E) imageStore(h,H,E)
#define q5(h,F) imageAtomicMax(h,H,F)
#define r5(h,F) imageAtomicAdd(h,H,F)
#else
#define T1(a) void main()
#define h2
#define i4
#define U1
#endif
#ifdef PLS_IMPL_ANGLE
#extension GL_ANGLE_shader_pixel_local_storage:require
#define R1
#define B0(g,a) layout(binding=g,rgba8) uniform mediump pixelLocalANGLE a
#define o1(g,a) layout(binding=g,r32ui) uniform highp upixelLocalANGLE a
#define S1
#define N0(h) pixelLocalLoadANGLE(h)
#define h1(h) pixelLocalLoadANGLE(h).x
#define y0(h,E) pixelLocalStoreANGLE(h,E)
#define j1(h,E) pixelLocalStoreANGLE(h,uvec4(E))
#define E2(h)
#define Z1(h)
#define F2
#define G2
#endif
#ifdef PLS_IMPL_EXT_NATIVE
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
#extension GL_EXT_shader_pixel_local_storage2:require
#else
#extension GL_EXT_shader_pixel_local_storage:require
#endif
#define R1 __pixel_localEXT a2{
#define B0(g,a) layout(rgba8) mediump vec4 a
#define tb(g,a) layout(rgb10_a2) mediump vec4 a
#define o1(g,a) layout(r32ui) highp uint a
#define S1 };
#define N0(h) h
#define h1(h) h
#define y0(h,E) h=(E)
#define j1(h,E) h=(E)
#define E2(h) h=h
#define Z1(h) h=h
#define F2
#define G2
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
#define A2(a) layout(location=0,rgba8) out i K1;T1(a)
#endif
#endif
#if defined(PLS_IMPL_STORAGE_TEXTURE)||defined(PLS_IMPL_STORAGE_BUFFER)
#define R1
#define S1
#define B0 pb
#define o1 M2
#define tb sb
#define N0 qb
#define y0 rb
#define h1 f3
#define j1 g3
#define E2(h)
#define Z1(h)
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
#define R1
#define J4(g,a) layout(input_attachment_index=g,binding=g,set=D3) uniform mediump subpassInput O7##a
#define ie(g,a) layout(location=g) out mediump vec4 a
#define B0(g,a) J4(g,a);ie(g,a)
#define o1(g,a) layout(input_attachment_index=g,binding=g,set=D3) uniform highp usubpassInput O7##a;layout(location=g) out highp uvec4 a
#define S1
#define N0(h) subpassLoad(O7##h)
#define h1(h) subpassLoad(O7##h).x
#define y0(h,E) h=(E)
#define j1(h,E) h.x=(E)
#define E2(h) y0(h,subpassLoad(O7##h))
#define Z1(h) j1(h,subpassLoad(O7##h).x)
#define F2
#define G2
#endif
#ifdef PLS_IMPL_NONE
#define R1
#define B0(g,a) layout(location=g) out mediump vec4 a
#define o1(g,a) layout(location=g) out highp uvec4 a
#define S1
#define N0(h) vec4(0)
#define h1(h) 0u
#define y0(h,E) h=(E)
#define j1(h,E) h.x=(E)
#define E2(h) h=vec4(0)
#define Z1(h) h.x=0u
#define F2
#define G2
#endif
#ifndef J4
#define J4 B0
#endif
#ifdef TARGET_SPIRV
#define ub gl_VertexIndex
#ifdef ENABLE_INSTANCE_INDEX
#define P7 gl_InstanceIndex
#else
#define P7 0
#endif
#else
#ifdef ENABLE_BASE_VERTEX
uniform highp int VE;
#define ub (gl_VertexID+VE)
#else
#define ub gl_VertexID
#endif
#ifdef ENABLE_INSTANCE_INDEX
#ifdef BASE_INSTANCE_UNIFORM_NAME
uniform highp int BASE_INSTANCE_UNIFORM_NAME;
#define P7 (gl_InstanceID+BASE_INSTANCE_UNIFORM_NAME)
#else
#define P7 (gl_InstanceID+gl_BaseInstance)
#endif
#else
#define P7 0
#endif
#endif
#define x6
#define H3
#define q7
#define Z4
#define v1(a,d0,D,p3,G6) void main(){int p3=ub;int G6=P7;
#define i8(a,d0,D,A1,h0,p3,G6) v1(a,d0,D,p3,G6)
#define U6(a,w3,x3,K3,L3,A1,h0,p3) v1(a,w3,x3,p3,G6)
#define T(a,j0)
#define Z(a)
#define q(a,j0)
#define w1(V0) gl_Position=V0;}
#define j3(F1,a) layout(location=0) out F1 Th;void main()
#define H6(F1,a) j3(F1,a)
#define I6 gl_FrontFacing
#define Q2(E) Th=E
#define f0 gl_FragCoord.xy
#define X6
#define e3
#if defined(PLS_IMPL_STORAGE_TEXTURE)||defined(PLS_IMPL_STORAGE_BUFFER)
#define je(Q7,h,E) if(!(Q7)){y0(h,E);}
#define ke(Q7,h,E) if(!(Q7)){j1(h,E);}
#else
#define je(Q7,h,E) y0(h,E);
#define ke(Q7,h,E) j1(h,E);
#endif
#ifndef A2
#define A2(a) layout(location=0) out i K1;T1(a)
#endif
#define A3 h2
#if defined(TARGET_SPIRV)&&!defined(TARGET_WGSL)
#ifdef MSAA_DST_COLOR
#define F5(a) layout(input_attachment_index=0,binding=L2,set=D3) uniform mediump subpassInputMS a
#define J6(a) Ia(mat4(subpassLoad(a,0),subpassLoad(a,1),subpassLoad(a,2),subpassLoad(a,3)),gl_SampleMaskIn[0])
#else
#define F5(a) layout(input_attachment_index=0,binding=L2,set=D3) uniform mediump subpassInput a
#define J6(a) subpassLoad(a)
#endif
#else
#define F5(a) i3(l3,Qg,a)
#define J6(a) texelFetch(a,ivec2(floor(f0.xy)),0)
#endif
#define M0(B,J) ((B)*(J))
precision highp float;precision highp int;
#if GLSL_VERSION<310
e i Uh(uint u){N q1=N(u&0xffu,(u>>8)&0xffu,(u>>16)&0xffu,u>>24);return f(q1)*(1./255.);}
#define unpackUnorm4x8 Uh
#endif
