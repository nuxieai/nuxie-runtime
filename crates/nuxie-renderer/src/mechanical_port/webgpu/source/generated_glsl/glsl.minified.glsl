#define da
#ifndef GLSL_VERSION
#define GLSL_VERSION __VERSION__
#endif
#define c vec2
#define S vec3
#define Q3 vec3
#define f vec4
#define d mediump float
#define D mediump vec2
#define v mediump vec3
#define i mediump vec4
#define a7 mediump mat3x3
#define c7 mediump mat2x3
#define J4 mediump mat4x4
#define d0 ivec2
#define d6 ivec4
#define O0 uvec2
#define R uvec4
#define N mediump uint
#define I4 bvec2
#define p6 bvec3
#define x7 bvec4
#define Y mat2
#define e
#define c1(p2) out p2
#define L6(p2) inout p2
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
#define w7(g,a) layout(binding=g,std140) uniform a{
#else
#define w7(g,a) layout(std140) uniform a{
#endif
#define M8(a) }a;
#define Kd(a) layout(push_constant) uniform a{
#define Ld(Z,a) Z a;
#define Md(a) }a;
#define g1(a)
#define I(g,Z,a) layout(location=g) in Z a
#define h1
#define J(P8,F,a,Z)
#ifdef VERTEX
#if GLSL_VERSION>=310
#define W(g,Z,a) layout(location=g) out Z a
#else
#define W(g,Z,a) out Z a
#endif
#else
#if GLSL_VERSION>=310
#define W(g,Z,a) layout(location=g) in Z a
#else
#define W(g,Z,a) in Z a
#endif
#endif
#define W2 flat
#define r2
#define i2
#ifdef TARGET_SPIRV
#define I0
#else
#ifdef GL_NV_shader_noperspective_interpolation
#extension GL_NV_shader_noperspective_interpolation:require
#define I0 noperspective
#else
#define I0
#endif
#endif
#ifdef VERTEX
#define Y3
#define Z3
#endif
#ifdef FRAGMENT
#define H3
#define I3
#endif
#define j5
#define k5
#ifdef TARGET_SPIRV
#define M4(a0,g,a) layout(set=a0,binding=g) uniform highp utexture2D a
#define q6(a0,g,a) layout(set=a0,binding=g) uniform highp texture2D a
#define e3(a0,g,a) layout(set=a0,binding=g) uniform mediump texture2D a
#define p5(a0,g,a) layout(binding=g) uniform mediump texture2D a
#if defined(FRAGMENT)&&defined(RENDER_MODE_DEPTH_STENCIL)
#endif
#elif GLSL_VERSION>=310
#define M4(a0,g,a) layout(binding=g) uniform highp usampler2D a
#define q6(a0,g,a) layout(binding=g) uniform highp sampler2D a
#define e3(a0,g,a) layout(binding=g) uniform mediump sampler2D a
#define p5(a0,g,a) layout(binding=g) uniform mediump sampler2D a
#else
#define M4(a0,g,a) uniform highp usampler2D a
#define q6(a0,g,a) uniform highp sampler2D a
#define e3(a0,g,a) uniform mediump sampler2D a
#define p5(a0,g,a) uniform mediump sampler2D a
#endif
#ifdef TARGET_SPIRV
#define r6(a0,g,a) layout(set=a0,binding=g) uniform mediump sampler a;
#ifdef USE_WEBGPU_SAMPLERS
#define g4(y7,a) layout(set=mg,binding=y7) uniform mediump sampler a;
#define c4(a) r6(i5,lg,a)
#else
#define g4(y7,a) layout(set=h3,binding=y7) uniform mediump sampler a;
#define c4(a) r6(i5,a4,a)
#endif
#define y5(a,p,m) texture(sampler2D(a,p),m)
#define j2(a,p,m,U0) textureLod(sampler2D(a,p),m,U0)
#define z5(a,p,m,U1) texture(sampler2D(a,p),m,U1)
#if defined(FRAGMENT)&&defined(RENDER_MODE_DEPTH_STENCIL)&&defined(MSAA_DST_COLOR)
#extension GL_OES_sample_variables:require
#endif
#else
#define g4(y7,a)
#define r6(a0,g,a)
#define c4(a)
#define y5(a,p,m) texture(a,m)
#define j2(a,p,m,U0) textureLod(a,m,U0)
#define z5(a,p,m,U1) texture(a,m,U1)
#endif
#define f8(n0,p,m) y5(n0,p,m)
#define V6(n0,p,m,U0) j2(n0,p,m,U0)
#define z7(n0,p,m,U1) z5(n0,p,m,U1)
#define h6(a0,g,a) p5(a0,g,a)
#define Z6(a,p,E,v6,R8,U0) j2(a,p,c(E,R8),U0)
#define mh(a0,g,a) M4(a0,g,a)
#define K3
#define e1
#define F1(a,m) texelFetch(a,m,0)
#ifdef TARGET_SPIRV
#elif GLSL_VERSION>=310
#else
#endif
#define G4
#define H4
#define T3
#define U3
#ifdef DISABLE_SHADER_STORAGE_BUFFERS
#define O5(g,x1,a) M4(h3,g,a)
#define N4(g,x1,a) mh(h3,g,a)
#define P5(g,x1,a) q6(h3,g,a)
#define L0(a,C0) F1(a,d0((C0)&Qc,(C0)>>Pc))
#define R5(a,C0) F1(a,d0((C0)&Qc,(C0)>>Pc)).xy
#else
#ifdef GL_ARB_shader_storage_buffer_object
#extension GL_ARB_shader_storage_buffer_object:require
#endif
#define O5(g,x1,a) layout(std430,binding=g) readonly buffer x1{O0 g2[];}a
#define N4(g,x1,a) layout(std430,binding=g) readonly buffer x1{R g2[];}a
#define P5(g,x1,a) layout(std430,binding=g) readonly buffer x1{f g2[];}a
#define Sa(g,x1,a) layout(std430,binding=g) buffer x1{uint g2[];}a
#define L0(a,C0) a.g2[C0]
#define R5(a,C0) a.g2[C0]
#define Qd(a,C0) a.g2[C0]
#define B7(a,C0,E) atomicMax(a.g2[C0],E)
#define Ta(a,C0,E) atomicAdd(a.g2[C0],E)
#define nh(a,C0,E) atomicOr(a.g2[C0],E)
#endif
#ifdef PLS_IMPL_STORAGE_BUFFER
#define P1(a) void main(){d0 G=ivec2(floor(e0));int G0=int(L8(uvec2(G),(j.o6+(xa-1u))&~(xa-1u)));
#define d2 }
#define V3 ,int G0
#define Q1 ,G0
#ifdef TARGET_WGSL
#define I2(g,a) layout(std430,set=w3,binding=g) buffer a##Rd{uint g2[];}a
#elif defined(TARGET_SPIRV)
#define I2(g,a) layout(std430,set=w3,binding=g) coherent buffer a##Rd{uint g2[];}a
#else
#define I2(g,a) layout(std430,binding=g) coherent buffer a##Rd{uint g2[];}a
#endif
#define Ua I2
#define a3(h) h.g2[G0]
#define c3(h,C) h.g2[G0]=C
#define Va(h) unpackUnorm4x8(a3(h))
#define Wa(h,C) c3(h,packUnorm4x8(C))
#define f5(h,E) atomicMax(h.g2[G0],E)
#define g5(h,E) atomicAdd(h.g2[G0],E)
#elif defined(PLS_IMPL_STORAGE_TEXTURE)||defined(USING_PLS_STORAGE_TEXTURES)
#ifdef GL_ARB_shader_image_load_store
#extension GL_ARB_shader_image_load_store:require
#endif
#define P1(a) void main(){d0 G=ivec2(floor(e0));
#define d2 }
#define V3 ,d0 G
#define Q1 ,G
#ifdef TARGET_SPIRV
#define Ua(g,a) layout(set=w3,binding=g,rgba8) uniform mediump coherent image2D a
#define I2(g,a) layout(set=w3,binding=g,r32ui) uniform highp coherent uimage2D a
#define Xa(g,a) layout(set=w3,binding=g,rgb10_a2) uniform mediump coherent image2D a
#else
#define Ua(g,a) layout(binding=g,rgba8) uniform mediump coherent image2D a
#define I2(g,a) layout(binding=g,r32ui) uniform highp coherent uimage2D a
#define Xa(g,a) layout(binding=g,rgb10_a2) uniform mediump coherent image2D a;
#endif
#define a3(h) imageLoad(h,G).x
#define c3(h,C) imageStore(h,G,uvec4(C))
#define Va(h) imageLoad(h,G)
#define Wa(h,C) imageStore(h,G,C)
#define f5(h,E) imageAtomicMax(h,G,E)
#define g5(h,E) imageAtomicAdd(h,G,E)
#else
#define P1(a) void main()
#define d2
#define V3
#define Q1
#endif
#ifdef PLS_IMPL_ANGLE
#extension GL_ANGLE_shader_pixel_local_storage:require
#define M1
#define z0(g,a) layout(binding=g,rgba8) uniform mediump pixelLocalANGLE a
#define j1(g,a) layout(binding=g,r32ui) uniform highp upixelLocalANGLE a
#define N1
#define K0(h) pixelLocalLoadANGLE(h)
#define a1(h) pixelLocalLoadANGLE(h).x
#define A0(h,C) pixelLocalStoreANGLE(h,C)
#define d1(h,C) pixelLocalStoreANGLE(h,uvec4(C))
#define z2(h)
#define h2(h)
#define A2
#define B2
#endif
#ifdef PLS_IMPL_EXT_NATIVE
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
#extension GL_EXT_shader_pixel_local_storage2:require
#else
#extension GL_EXT_shader_pixel_local_storage:require
#endif
#define M1 __pixel_localEXT V1{
#define z0(g,a) layout(rgba8) mediump vec4 a
#define Ya(g,a) layout(rgb10_a2) mediump vec4 a
#define j1(g,a) layout(r32ui) highp uint a
#define N1 };
#define K0(h) h
#define a1(h) h
#define A0(h,C) h=(C)
#define d1(h,C) h=(C)
#define z2(h) h=h
#define h2(h) h=h
#define A2
#define B2
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
#define w2(a) layout(location=0,rgba8) out i E1;P1(a)
#endif
#endif
#if defined(PLS_IMPL_STORAGE_TEXTURE)||defined(PLS_IMPL_STORAGE_BUFFER)
#define M1
#define N1
#define z0 Ua
#define j1 I2
#define Ya Xa
#define K0 Va
#define A0 Wa
#define a1 a3
#define d1 c3
#define z2(h)
#define h2(h)
#if defined(GL_ARB_fragment_shader_interlock)
#extension GL_ARB_fragment_shader_interlock:require
#define A2 beginInvocationInterlockARB()
#define B2 endInvocationInterlockARB()
#elif defined(GL_INTEL_fragment_shader_ordering)
#extension GL_INTEL_fragment_shader_ordering:require
#define A2 beginFragmentShaderOrderingINTEL()
#define B2
#else
#define A2
#define B2
#endif
#endif
#ifdef PLS_IMPL_SUBPASS_LOAD
#define M1
#define z4(g,a) layout(input_attachment_index=g,binding=g,set=w3) uniform mediump subpassInput C7##a
#define Sd(g,a) layout(location=g) out mediump vec4 a
#define z0(g,a) z4(g,a);Sd(g,a)
#define j1(g,a) layout(input_attachment_index=g,binding=g,set=w3) uniform highp usubpassInput C7##a;layout(location=g) out highp uvec4 a
#define N1
#define K0(h) subpassLoad(C7##h)
#define a1(h) subpassLoad(C7##h).x
#define A0(h,C) h=(C)
#define d1(h,C) h.x=(C)
#define z2(h) A0(h,subpassLoad(C7##h))
#define h2(h) d1(h,subpassLoad(C7##h).x)
#define A2
#define B2
#endif
#ifdef PLS_IMPL_NONE
#define M1
#define z0(g,a) layout(location=g) out mediump vec4 a
#define j1(g,a) layout(location=g) out highp uvec4 a
#define N1
#define K0(h) vec4(0)
#define a1(h) 0u
#define A0(h,C) h=(C)
#define d1(h,C) h.x=(C)
#define z2(h) h=vec4(0)
#define h2(h) h.x=0u
#define A2
#define B2
#endif
#ifndef z4
#define z4 z0
#endif
#ifdef TARGET_SPIRV
#define gl_VertexID gl_VertexIndex
#endif
#ifdef ENABLE_INSTANCE_INDEX
#ifdef TARGET_SPIRV
#define S8 gl_InstanceIndex
#else
#ifdef BASE_INSTANCE_UNIFORM_NAME
uniform highp int BASE_INSTANCE_UNIFORM_NAME;
#define S8 (gl_InstanceID+BASE_INSTANCE_UNIFORM_NAME)
#else
#define S8 (gl_InstanceID+gl_BaseInstance)
#endif
#endif
#else
#define S8 0
#endif
#define l6
#define A3
#define g7
#define A5
#define A1(a,h0,F,A,q) void main(){int A=gl_VertexID;int q=S8;
#define R7(a,h0,F,p1,g0,A,q) A1(a,h0,F,A,q)
#define J6(a,n3,o3,C3,D3,p1,g0,A) A1(a,n3,o3,A,q)
#define V(a,Z)
#define c0(a)
#define r(a,Z)
#define B1(R0) gl_Position=R0;}
#define f3(y1,a) layout(location=0) out y1 oh;void main()
#define w6(y1,a) f3(y1,a)
#define x6 gl_FrontFacing
#define N2(C) oh=C
#define e0 gl_FragCoord.xy
#define M6
#define Z2
#if defined(PLS_IMPL_STORAGE_TEXTURE)||defined(PLS_IMPL_STORAGE_BUFFER)
#define Td(D7,h,C) if(!(D7)){A0(h,C);}
#define Ud(D7,h,C) if(!(D7)){d1(h,C);}
#else
#define Td(D7,h,C) A0(h,C);
#define Ud(D7,h,C) d1(h,C);
#endif
#ifndef w2
#define w2(a) layout(location=0) out i E1;P1(a)
#endif
#define r3 d2
#if defined(TARGET_SPIRV)&&!defined(TARGET_WGSL)
#ifdef MSAA_DST_COLOR
#define q5(a) layout(input_attachment_index=0,binding=H2,set=w3) uniform mediump subpassInputMS a
#define y6(a) na(mat4(subpassLoad(a,0),subpassLoad(a,1),subpassLoad(a,2),subpassLoad(a,3)),gl_SampleMaskIn[0])
#else
#define q5(a) layout(input_attachment_index=0,binding=H2,set=w3) uniform mediump subpassInput a
#define y6(a) subpassLoad(a)
#endif
#else
#define q5(a) e3(h3,kg,a)
#define y6(a) texelFetch(a,ivec2(floor(e0.xy)),0)
#endif
#define N0(B,H) ((B)*(H))
precision highp float;precision highp int;
#if GLSL_VERSION<310
e i ph(uint u){R k1=R(u&0xffu,(u>>8)&0xffu,(u>>16)&0xffu,u>>24);return f(k1)*(1./255.);}
#define unpackUnorm4x8 ph
#endif
