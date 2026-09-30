#ifndef _ARE_TOKEN_NAMES_PRESERVED
#define d half
#define E half2
#define A half3
#define i half4
#define L ushort
#define c float2
#define Q float3
#define N3 packed_float3
#define f float4
#define F4 bool2
#define r6 bool3
#define z7 bool4
#define a1 uint2
#define X uint4
#define Y int2
#define g6 int4
#define L ushort
#define d0 float2x2
#define a7 half3x3
#define c7 half2x3
#define G4 half4x4
#endif
#define e inline
#define Z0(n2) thread n2&
#define X4(n2) thread n2&
#define equal(C,H) ((C)==(H))
#define notEqual(C,H) ((C)!=(H))
#define lessThan(C,H) ((C)<(H))
#define greaterThan(C,H) ((C)>(H))
#define N0(C,H) ((C)*(H))
#define inversesqrt rsqrt
#define y7(g,a) struct a{
#define N8(a) };
#define f1(a) struct a{
#define J(g,Z,a) Z a
#define g1 };
#define K(Q8,F,a,Z) Z a=F[Q8].a
#define p2 struct o0{
#define V(g,Z,a) Z a
#define T2 [[flat]]
#define H0 [[center_no_perspective]]
#ifndef OPTIONALLY_FLAT
#define OPTIONALLY_FLAT
#endif
#define h2 f P0[[position]][[invariant]];};
#define T(a,Z) thread Z&a=e0.a
#define a0(a)
#define r(a,Z) Z a=e0.a
#define B4 struct V8{
#define C4 };
#define P3 struct W8{
#define Q3 };
#define O5(g,v1,a) constant a1*a[[buffer(O1(g))]]
#define J4(g,v1,a) constant X*a[[buffer(O1(g))]]
#define P5(g,v1,a) constant f*a[[buffer(O1(g))]]
#define K0(a,B0) v3.a[B0]
#define R5(a,B0) v3.a[B0]
#define U3 struct X8{
#define V3 };
#define F3 struct D5{
#define G3 };
#define e5 struct bb{
#define f5 };
#define E4(U,g,a) [[texture(g)]]texture2d<uint>a
#define h5(U,g,a) [[texture(g)]]texture2d<float>a
#define c3(U,g,a) [[texture(g)]]texture2d<d>a
#define m5(U,g,a) [[texture(g)]]texture2d<d>a
#define k6(U,g,a) [[texture(g)]]texture1d_array<d>a
#define c4(A7,a) constexpr sampler a(filter::linear,mip_filter::none);
#define v6(U,g,a) [[sampler(g)]]sampler a;
#define X3(a) [[sampler(W3)]]sampler a;
#define p1(l0,l) W0.l0.read(a1(l))
#define w5(l0,p,l) W0.l0.sample(p,l)
#define i2(l0,p,l,S0) W0.l0.sample(p,l,level(S0))
#define x5(l0,p,l,R1) W0.l0.sample(p,l,bias(R1))
#define h8(l0,p,l) W0.l0.sample(A6.p,l)
#define V6(l0,p,l,S0) W0.l0.sample(A6.p,l,level(S0))
#define B7(l0,p,l,R1) W0.l0.sample(A6.p,l,bias(R1))
#define Z6(l0,p,q,w6,S8,S0) W0.l0.sample(p,q,w6)
#define n6 ,constant BC&n,X8 W0,V8 v3
#define x3 ,n,W0,v3
#ifdef ENABLE_INSTANCE_INDEX
#define y1(a,f0,F,B,v) __attribute__((visibility("default")))o0 vertex a(uint B[[vertex_id]],uint v[[instance_id]],constant uint&oh[[buffer(O1(Vc))]],constant BC&n[[buffer(O1(I4))]],constant f0*F[[buffer(0)]],X8 W0,V8 v3){v+=oh;o0 e0;
#else
#define y1(a,f0,F,B,v) __attribute__((visibility("default")))o0 vertex a(uint B[[vertex_id]],uint v[[instance_id]],constant BC&n[[buffer(O1(I4))]],constant f0*F[[buffer(0)]],X8 W0,V8 v3){o0 e0;
#endif
#define T7(a,f0,F,m1,g0,B,v) __attribute__((visibility("default")))o0 vertex a(uint B[[vertex_id]],uint v[[instance_id]],constant BC&n[[buffer(O1(I4))]],constant f0*F[[buffer(0)]],const device m1*g0[[buffer(2)]],X8 W0,V8 v3){o0 e0;
#define K6(a,j3,k3,z3,A3,m1,g0,B) __attribute__((visibility("default")))o0 vertex a(uint B[[vertex_id]],uint v[[instance_id]],constant BC&n[[buffer(O1(I4))]],constant j3*k3[[buffer(0)]],constant z3*A3[[buffer(1)]],const device m1*g0[[buffer(2)]]){o0 e0;
#define z1(C5) e0.P0=C5;}return e0;
#define d3(w1,a) w1 __attribute__((visibility("default")))fragment a(o0 e0[[stage_in]],D5 W0){
#define x6(w1,a) w1 __attribute__((visibility("default")))fragment a(o0 e0[[stage_in]],D5 W0,bool y6[[front_facing]]){
#define L2(D) return D;}
#define M6 ,c c0,D5 W0,W8 v3,bb A6
#define W2 ,c0,W0,v3,A6
#define I3 ,D5 W0
#define d1 ,W0
#define g7
#define y5
#ifdef PLS_IMPL_DEVICE_BUFFER
#define J1 struct S1{
#ifdef PLS_IMPL_DEVICE_BUFFER_RASTER_ORDERED
#define y0(g,a) device uint*a[[buffer(O1(g+h6)),raster_order_group(0)]]
#define i1(g,a) device uint*a[[buffer(O1(g+h6)),raster_order_group(0)]]
#define H2(g,a) device atomic_uint*a[[buffer(O1(g+h6)),raster_order_group(0)]]
#else
#define y0(g,a) device uint*a[[buffer(O1(g+h6))]]
#define i1(g,a) device uint*a[[buffer(O1(g+h6))]]
#define H2(g,a) device atomic_uint*a[[buffer(O1(g+h6))]]
#endif
#define K1 };
#define R3 ,S1 T0,uint F0
#define N1 ,T0,F0
#define J0(h) unpackUnorm4x8(T0.h[F0])
#define Y0(h) T0.h[F0]
#define X2(h) atomic_load_explicit(&T0.h[F0],memory_order::memory_order_relaxed)
#define z0(h,D) T0.h[F0]=packUnorm4x8(D)
#define c1(h,D) T0.h[F0]=(D)
#define Y2(h,D) atomic_store_explicit(&T0.h[F0],D,memory_order::memory_order_relaxed)
#define y2(h)
#define f2(h)
#define Z4(h,q) atomic_fetch_max_explicit(&T0.h[F0],q,memory_order::memory_order_relaxed)
#define a5(h,q) atomic_fetch_add_explicit(&T0.h[F0],q,memory_order::memory_order_relaxed)
#define z2
#define A2
#define Y8(a) __attribute__((visibility("default")))fragment a(S1 T0,constant BC&n[[buffer(O1(I4))]],o0 e0[[stage_in]],D5 W0,bb A6,W8 v3){c c0=e0.P0.xy;a1 G=a1(metal::floor(c0));uint F0=G.y*n.q6+G.x;
#define M1(a) void Y8(a)
#define a2 }
#define r2(a) i Y8(a){i C1;
#define m3 }return C1;a2
#else
#define J1 struct S1{
#define y0(g,a) [[color(g)]]i a
#define i1(g,a) [[color(g)]]uint a
#define H2 i1
#define K1 };
#define R3 ,thread S1&E5,thread S1&T0
#define N1 ,E5,T0
#define J0(h) E5.h
#define Y0(h) E5.h
#define X2(h) Y0
#define z0(h,D) T0.h=(D)
#define c1(h,D) T0.h=(D)
#define Y2(h) c1
#define y2(h) T0.h=E5.h
#define f2(h) T0.h=E5.h
e uint A5(thread uint&r0,uint x){uint V0=r0;r0=metal::max(V0,x);return V0;}
#define Z4(h,q) A5(T0.h,q)
e uint B5(thread uint&r0,uint x){uint V0=r0;r0=V0+x;return V0;}
#define a5(h,q) B5(T0.h,q)
#define z2
#define A2
#define Y8(a,...) S1 __attribute__((visibility("default")))fragment a(__VA_ARGS__){c c0[[maybe_unused]]=e0.P0.xy;S1 T0;
#define M1(a,...) Y8(a,S1 E5,constant BC&n[[buffer(O1(I4))]],o0 e0[[stage_in]],bb A6,D5 W0,W8 v3)
#define a2 }return T0;
#define rh(a,...) struct ph{i qh[[j(0)]];S1 T0;};ph __attribute__((visibility("default")))fragment a(__VA_ARGS__){c c0[[maybe_unused]]=e0.P0.xy;i C1;S1 T0;
#define r2(a) rh(a,S1 E5,constant BC&n[[buffer(O1(I4))]],o0 e0[[stage_in]],D5 W0,W8 v3)
#define m3 }return{.qh=C1,.T0=T0};
#endif
#define v4 y0
#define discard discard_fragment()
using namespace metal;template<int Q1>e vec<uint,Q1>floatBitsToUint(vec<float,Q1>x){return as_type<vec<uint,Q1>>(x);}template<int Q1>e vec<int,Q1>floatBitsToInt(vec<float,Q1>x){return as_type<vec<int,Q1>>(x);}e uint floatBitsToUint(float x){return as_type<uint>(x);}e int floatBitsToInt(float x){return as_type<int>(x);}template<int Q1>e vec<float,Q1>uintBitsToFloat(vec<uint,Q1>x){return as_type<vec<float,Q1>>(x);}e float uintBitsToFloat(uint x){return as_type<float>(x);}e E unpackHalf2x16(uint x){return as_type<E>(x);}e uint packHalf2x16(E x){return as_type<uint>(x);}e i unpackUnorm4x8(uint x){return unpack_unorm4x8_to_half(x);}e uint packUnorm4x8(i x){return pack_half_to_unorm4x8(x);}e d0 inverse(d0 k1){d0 cb=d0(k1[1][1],-k1[0][1],-k1[1][0],k1[0][0]);float sh=(cb[0][0]*k1[0][0])+(cb[0][1]*k1[1][0]);return cb*(1/sh);}e A mix(A m,A b,r6 G1){A G7;for(int G0=0;G0<3;++G0)G7[G0]=G1[G0]?b[G0]:m[G0];return G7;}e c mix(c m,c b,F4 G1){c G7;for(int G0=0;G0<2;++G0)G7[G0]=G1[G0]?b[G0]:m[G0];return G7;}e c mix(c m,c b,float t){return mix(m,b,c(t));}e float mod(float x,float y){return fmod(x,y);}