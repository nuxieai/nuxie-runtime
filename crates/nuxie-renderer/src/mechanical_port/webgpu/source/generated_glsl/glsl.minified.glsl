#define gb
#ifndef GLSL_VERSION
#define GLSL_VERSION __VERSION__
#endif
#define c vec2
#define M vec3
#define j4 vec3
#define e vec4
#define d mediump float
#define E mediump vec2
#define v mediump vec3
#define i mediump vec4
#define D7 mediump mat3x3
#define E7 mediump mat2x3
#define h5 mediump mat4x4
#define f0 ivec2
#define y6 ivec4
#define U0 uvec2
#define R uvec4
#define O mediump uint
#define g5 bvec2
#define U6 bvec3
#define g8 bvec4
#define X mat2
#define f
#define k1(F2) out F2
#define m7(F2) inout F2
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
#define f8(g,a) layout(binding=g,std140) uniform a{
#else
#define f8(g,a) layout(std140) uniform a{
#endif
#define N9(a) }a;
#define d1(a)
#define K(g,k0,a) layout(location=g) in k0 a
#define e1
#define L(Q9,A,a,k0)
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
#define i3 flat
#define n2
#define N1
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
#define x4
#define y4
#endif
#ifdef FRAGMENT
#define Y3
#define Z3
#endif
#define C5
#define D5
#ifdef TARGET_SPIRV
#define l5(h0,g,a) layout(set=h0,binding=g) uniform highp utexture2D a
#define V6(h0,g,a) layout(set=h0,binding=g) uniform highp texture2D a
#define r3(h0,g,a) layout(set=h0,binding=g) uniform mediump texture2D a
#define O5(h0,g,a) layout(binding=g) uniform mediump texture2D a
#if defined(FRAGMENT)&&defined(RENDER_MODE_DEPTH_STENCIL)
#endif
#elif GLSL_VERSION>=310
#define l5(h0,g,a) layout(binding=g) uniform highp usampler2D a
#define V6(h0,g,a) layout(binding=g) uniform highp sampler2D a
#define r3(h0,g,a) layout(binding=g) uniform mediump sampler2D a
#define O5(h0,g,a) layout(binding=g) uniform mediump sampler2D a
#else
#define l5(h0,g,a) uniform highp usampler2D a
#define V6(h0,g,a) uniform highp sampler2D a
#define r3(h0,g,a) uniform mediump sampler2D a
#define O5(h0,g,a) uniform mediump sampler2D a
#endif
#ifdef TARGET_SPIRV
#define W6(h0,g,a) layout(set=h0,binding=g) uniform mediump sampler a;
#ifdef USE_WEBGPU_SAMPLERS
#define d4(T5,a) layout(set=Nh,binding=T5) uniform mediump sampler a;
#define L6 d4
#define A4(a) W6(B5,Mh,a)
#else
#define d4(T5,a) layout(set=w3,binding=T5) uniform mediump sampler a;
#define L6 d4
#define A4(a) W6(B5,z4,a)
#endif
#define U5(a,p,o) texture(sampler2D(a,p),o)
#define q2(a,p,o,a1) textureLod(sampler2D(a,p),o,a1)
#define V5(a,p,o,i2) texture(sampler2D(a,p),o,i2)
#if defined(FRAGMENT)&&defined(RENDER_MODE_DEPTH_STENCIL)&&defined(MSAA_DST_COLOR)
#extension GL_OES_sample_variables:require
#endif
#else
#define d4(T5,a)
#define L6(T5,a)
#define W6(h0,g,a)
#define A4(a)
#define U5(a,p,o) texture(a,o)
#define q2(a,p,o,a1) textureLod(a,o,a1)
#define V5(a,p,o,i2) texture(a,o,i2)
#endif
#define X8(r0,p,o) U5(r0,p,o)
#define e5(r0,p,o,a1) q2(r0,p,o,a1)
#define h8(r0,p,o,i2) V5(r0,p,o,i2)
#define K6(h0,g,a) O5(h0,g,a)
#define C7(a,p,F,X6,S9,a1) q2(a,p,c(F,S9),a1)
#define Pi(h0,g,a) l5(h0,g,a)
#define e4
#define m1
#define v1(a,o) texelFetch(a,o,0)
#ifdef TARGET_SPIRV
#elif GLSL_VERSION>=310
#else
#endif
#define c5
#define d5
#define l4
#define n4
#ifdef DISABLE_SHADER_STORAGE_BUFFERS
#define m4(g,M1,a) Pi(w3,g,a)
#define k6(g,M1,a) V6(w3,g,a)
#define n0(a,G0) v1(a,f0((G0)&fe,(G0)>>ee))
#else
#ifdef GL_ARB_shader_storage_buffer_object
#extension GL_ARB_shader_storage_buffer_object:require
#endif
#define m4(g,M1,a) layout(std430,binding=g) readonly buffer M1{R x2[];}a
#define k6(g,M1,a) layout(std430,binding=g) readonly buffer M1{e x2[];}a
#define jc(g,M1,a) layout(std430,binding=g) buffer M1{uint x2[];}a
#define n0(a,G0) a.x2[G0]
#define cf(a,G0) a.x2[G0]
#define j8(a,G0,F) atomicMax(a.x2[G0],F)
#define kc(a,G0,F) atomicAdd(a.x2[G0],F)
#define Qi(a,G0,F) atomicOr(a.x2[G0],F)
#endif
#ifdef PLS_IMPL_STORAGE_BUFFER
#define I1(a) void main(){f0 G=ivec2(floor(c0));int O0=int(M9(uvec2(G),(j.T6+(Ib-1u))&~(Ib-1u)));
#define P1 }
#define p4 ,int O0
#define a2 ,O0
#ifdef TARGET_WGSL
#define X2(g,a) layout(std430,set=M3,binding=g) buffer a##df{uint x2[];}a
#elif defined(TARGET_SPIRV)
#define X2(g,a) layout(std430,set=M3,binding=g) coherent buffer a##df{uint x2[];}a
#else
#define X2(g,a) layout(std430,binding=g) coherent buffer a##df{uint x2[];}a
#endif
#define Q5 X2
#define o3(h) h.x2[O0]
#define p3(h,D) h.x2[O0]=D
#define L5(h) unpackUnorm4x8(o3(h))
#define N5(h,D) p3(h,packUnorm4x8(D))
#define z5(h,F) atomicMax(h.x2[O0],F)
#define A5(h,F) atomicAdd(h.x2[O0],F)
#elif defined(PLS_IMPL_STORAGE_TEXTURE)||defined(USING_PLS_STORAGE_TEXTURES)
#ifdef GL_ARB_shader_image_load_store
#extension GL_ARB_shader_image_load_store:require
#endif
#define I1(a) void main(){f0 G=ivec2(floor(c0));
#define P1 }
#define p4 ,f0 G
#define a2 ,G
#ifdef TARGET_SPIRV
#define Q5(g,a) layout(set=M3,binding=g,rgba8) uniform mediump coherent image2D a
#define X2(g,a) layout(set=M3,binding=g,r32ui) uniform highp coherent uimage2D a
#define lc(g,a) layout(set=M3,binding=g,rgb10_a2) uniform mediump coherent image2D a
#else
#define Q5(g,a) layout(binding=g,rgba8) uniform mediump coherent image2D a
#define X2(g,a) layout(binding=g,r32ui) uniform highp coherent uimage2D a
#define lc(g,a) layout(binding=g,rgb10_a2) uniform mediump coherent image2D a;
#endif
#define o3(h) imageLoad(h,G).x
#define p3(h,D) imageStore(h,G,uvec4(D))
#define L5(h) imageLoad(h,G)
#define N5(h,D) imageStore(h,G,D)
#define z5(h,F) imageAtomicMax(h,G,F)
#define A5(h,F) imageAtomicAdd(h,G,F)
#else
#define I1(a) void main()
#define P1
#define p4
#define a2
#endif
#ifdef PLS_IMPL_ANGLE
#extension GL_ANGLE_shader_pixel_local_storage:require
#define F1
#define y0(g,a) layout(binding=g,rgba8) uniform mediump pixelLocalANGLE a
#define q1(g,a) layout(binding=g,r32ui) uniform highp upixelLocalANGLE a
#define G1
#define K0(h) pixelLocalLoadANGLE(h)
#define j1(h) pixelLocalLoadANGLE(h).x
#define x0(h,D) pixelLocalStoreANGLE(h,D)
#define l1(h,D) pixelLocalStoreANGLE(h,uvec4(D))
#define Q2(h)
#define j2(h)
#define R2
#define S2
#endif
#ifdef PLS_IMPL_EXT_NATIVE
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
#extension GL_EXT_shader_pixel_local_storage2:require
#else
#extension GL_EXT_shader_pixel_local_storage:require
#endif
#define F1 __pixel_localEXT k2{
#define y0(g,a) layout(rgba8) mediump vec4 a
#define mc(g,a) layout(rgb10_a2) mediump vec4 a
#define q1(g,a) layout(r32ui) highp uint a
#define G1 };
#define K0(h) h
#define j1(h) h
#define x0(h,D) h=(D)
#define l1(h,D) h=(D)
#define Q2(h) h=h
#define j2(h) h=h
#define R2
#define S2
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
#define I2(a) layout(location=0,rgba8) out i R1;I1(a)
#endif
#endif
#if defined(PLS_IMPL_STORAGE_TEXTURE)||defined(PLS_IMPL_STORAGE_BUFFER)
#define F1
#define G1
#define y0 Q5
#define q1 X2
#define mc lc
#define K0 L5
#define x0 N5
#define j1 o3
#define l1 p3
#define Q2(h)
#define j2(h)
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
#ifdef PLS_IMPL_SUBPASS_LOAD
#define F1
#define U4(g,a) layout(input_attachment_index=g,binding=g,set=M3) uniform mediump subpassInput k8##a
#define ef(g,a) layout(location=g) out mediump vec4 a
#define y0(g,a) U4(g,a);ef(g,a)
#define q1(g,a) layout(input_attachment_index=g,binding=g,set=M3) uniform highp usubpassInput k8##a;layout(location=g) out highp uvec4 a
#define G1
#define K0(h) subpassLoad(k8##h)
#define j1(h) subpassLoad(k8##h).x
#define x0(h,D) h=(D)
#define l1(h,D) h.x=(D)
#define Q2(h) x0(h,subpassLoad(k8##h))
#define j2(h) l1(h,subpassLoad(k8##h).x)
#define R2
#define S2
#endif
#ifdef PLS_IMPL_NONE
#define F1
#define y0(g,a) layout(location=g) out mediump vec4 a
#define q1(g,a) layout(location=g) out highp uvec4 a
#define G1
#define K0(h) vec4(0)
#define j1(h) 0u
#define x0(h,D) h=(D)
#define l1(h,D) h.x=(D)
#define Q2(h) h=vec4(0)
#define j2(h) h.x=0u
#define R2
#define S2
#endif
#ifndef U4
#define U4 y0
#endif
#ifdef TARGET_SPIRV
#define nc gl_VertexIndex
#ifdef ENABLE_INSTANCE_INDEX
#define l8 gl_InstanceIndex
#else
#define l8 0
#endif
#else
#ifdef ENABLE_BASE_VERTEX
uniform highp int ZE;
#define nc (gl_VertexID+ZE)
#else
#define nc gl_VertexID
#endif
#ifdef ENABLE_INSTANCE_INDEX
#ifdef BASE_INSTANCE_UNIFORM_NAME
uniform highp int BASE_INSTANCE_UNIFORM_NAME;
#define l8 (gl_InstanceID+BASE_INSTANCE_UNIFORM_NAME)
#else
#define l8 (gl_InstanceID+gl_BaseInstance)
#endif
#else
#define l8 0
#endif
#endif
#define O6
#define T3
#define x6
#define E4
#define n1(a,e0,A,O2,D6) void main(){int O2=nc;int D6=l8;
#define G8(a,e0,A,E1,j0,O2,D6) n1(a,e0,A,O2,D6)
#define k7(a,F3,G3,W3,k3,E1,j0,O2) n1(a,F3,G3,O2,D6)
#define V(a,k0)
#define a0(a)
#define q(a,k0)
#define A1(T0) gl_Position=T0;}
#define M2(w1,a) layout(location=0) out w1 Ri;void main()
#define Y6(w1,a) M2(w1,a)
#define Z6 gl_FrontFacing
#define N2(D) Ri=D
#define c0 gl_FragCoord.xy
#define n7
#define n3
#if defined(PLS_IMPL_STORAGE_TEXTURE)||defined(PLS_IMPL_STORAGE_BUFFER)
#define ff(m8,h,D) if(!(m8)){x0(h,D);}
#define gf(m8,h,D) if(!(m8)){l1(h,D);}
#else
#define ff(m8,h,D) x0(h,D);
#define gf(m8,h,D) l1(h,D);
#endif
#ifndef I2
#define I2(a) layout(location=0) out i R1;I1(a)
#endif
#define H3 P1
#if defined(TARGET_SPIRV)&&!defined(TARGET_WGSL)
#ifdef MSAA_DST_COLOR
#define P5(a) layout(input_attachment_index=0,binding=z2,set=M3) uniform mediump subpassInputMS a
#define M5(a) rb(mat4(subpassLoad(a,0),subpassLoad(a,1),subpassLoad(a,2),subpassLoad(a,3)),gl_SampleMaskIn[0])
#else
#define P5(a) layout(input_attachment_index=0,binding=z2,set=M3) uniform mediump subpassInput a
#define M5(a) subpassLoad(a)
#endif
#else
#define P5(a) r3(w3,Lh,a)
#define M5(a) texelFetch(a,ivec2(floor(c0.xy)),0)
#endif
#define D0(B,J) ((B)*(J))
precision highp float;precision highp int;
#if GLSL_VERSION<310
f i Si(uint u){R x1=R(u&0xffu,(u>>8)&0xffu,(u>>16)&0xffu,u>>24);return e(x1)*(1./255.);}
#define unpackUnorm4x8 Si
#endif
