#ifndef _ARE_TOKEN_NAMES_PRESERVED
#define c half
#define E half2
#define v half3
#define i half4
#define N ushort
#define d float2
#define c0 float3
#define L3 packed_float3
#define g float4
#define D4 bool2
#define n6 bool3
#define v7 bool4
#define a1 uint2
#define G uint4
#define X int2
#define a6 int4
#define N ushort
#define g0 float2x2
#define V6 half3x3
#define W6 half2x3
#define h5 half4x4
#endif
#define e inline
#define Z0(j2) thread j2&
#define T4(j2) thread j2&
#define equal(C,H) ((C)==(H))
#define notEqual(C,H) ((C)!=(H))
#define lessThan(C,H) ((C)<(H))
#define greaterThan(C,H) ((C)>(H))
#define U0(C,H) ((C)*(H))
#define inversesqrt rsqrt
#define r7(f,a) struct a{
#define J8(a) };
#define g1(a) struct a{
#define L(f,d0,a) d0 a
#define h1 };
#define M(M8,F,a,d0) d0 a=F[M8].a
#define k2 struct n0{
#define W(f,d0,a) d0 a
#define O2 [[flat]]
#define J0 [[center_no_perspective]]
#ifndef MB
#define MB
#endif
#define f2 g O0[[position]][[invariant]];};
#define V(a,d0) thread d0&a=Z.a
#define a0(a)
#define A(a,d0) d0 a=Z.a
#define z4 struct R8{
#define A4 };
#define O3 struct S8{
#define P3 };
#define K5(f,w1,a) constant a1*a[[buffer(N1(f))]]
#define G4(f,w1,a) constant G*a[[buffer(N1(f))]]
#define L5(f,w1,a) constant g*a[[buffer(N1(f))]]
#define N0(a,A0) p3.a[A0]
#define M5(a,A0) p3.a[A0]
#define S3 struct T8{
#define T3 };
#define C3 struct A5{
#define D3 };
#define a5 struct Ua{
#define c5 };
#define C4(T,f,a) [[texture(f)]]texture2d<uint>a
#define e5(T,f,a) [[texture(f)]]texture2d<float>a
#define X2(T,f,a) [[texture(f)]]texture2d<c>a
#define k5(T,f,a) [[texture(f)]]texture2d<c>a
#define f6(T,f,a) [[texture(f)]]texture1d_array<c>a
#define Z3(w7,a) constexpr sampler a(filter::linear,mip_filter::none);
#define o6(T,f,a) [[sampler(f)]]sampler a;
#define V3(a) [[sampler(U3)]]sampler a;
#define q1(k0,l) W0.k0.read(a1(l))
#define q5(k0,p,l) W0.k0.sample(p,l)
#define n2(k0,p,l,R0) W0.k0.sample(p,l,level(R0))
#define r5(k0,p,l,Q1) W0.k0.sample(p,l,bias(Q1))
#define d8(k0,p,l) W0.k0.sample(v6.p,l)
#define Q6(k0,p,l,R0) W0.k0.sample(v6.p,l,level(R0))
#define x7(k0,p,l,Q1) W0.k0.sample(v6.p,l,bias(Q1))
#define U6(k0,p,q,p6,O8,R0) W0.k0.sample(p,q,p6)
#define i6 ,constant CC&n,T8 W0,R8 p3
#define v3 ,n,W0,p3
#ifdef NE
#define y1(a,e0,F,B,r) __attribute__((visibility("default")))n0 vertex a(uint B[[vertex_id]],uint r[[instance_id]],constant uint&Mg[[buffer(N1(Kc))]],constant CC&n[[buffer(N1(F4))]],constant e0*F[[buffer(0)]],T8 W0,R8 p3){r+=Mg;n0 Z;
#else
#define y1(a,e0,F,B,r) __attribute__((visibility("default")))n0 vertex a(uint B[[vertex_id]],uint r[[instance_id]],constant CC&n[[buffer(N1(F4))]],constant e0*F[[buffer(0)]],T8 W0,R8 p3){n0 Z;
#endif
#define P7(a,e0,F,n1,f0,B,r) __attribute__((visibility("default")))n0 vertex a(uint B[[vertex_id]],uint r[[instance_id]],constant CC&n[[buffer(N1(F4))]],constant e0*F[[buffer(0)]],constant n1*f0[[buffer(2)]],T8 W0,R8 p3){n0 Z;
#define F6(a,g3,h3,x3,y3,n1,f0,B) __attribute__((visibility("default")))n0 vertex a(uint B[[vertex_id]],uint r[[instance_id]],constant CC&n[[buffer(N1(F4))]],constant g3*h3[[buffer(0)]],constant x3*y3[[buffer(1)]],constant n1*f0[[buffer(2)]]){n0 Z;
#define z1(z5) Z.O0=z5;}return Z;
#define Y2(S1,a) S1 __attribute__((visibility("default")))fragment a(n0 Z[[stage_in]],A5 W0){
#define q6(S1,a) S1 __attribute__((visibility("default")))fragment a(n0 Z[[stage_in]],A5 W0,bool r6[[front_facing]]){
#define G2(D) return D;}
#define H6 ,d Y,A5 W0,S8 p3,Ua v6
#define S2 ,Y,W0,p3,v6
#define G3 ,A5 W0
#define d1 ,W0
#define a7
#define v5
#ifdef OF
#define I1 struct R1{
#ifdef PF
#define w0(f,a) device uint*a[[buffer(N1(f+c6)),raster_order_group(0)]]
#define j1(f,a) device uint*a[[buffer(N1(f+c6)),raster_order_group(0)]]
#define D2(f,a) device atomic_uint*a[[buffer(N1(f+c6)),raster_order_group(0)]]
#else
#define w0(f,a) device uint*a[[buffer(N1(f+c6))]]
#define j1(f,a) device uint*a[[buffer(N1(f+c6))]]
#define D2(f,a) device atomic_uint*a[[buffer(N1(f+c6))]]
#endif
#define J1 };
#define Q3 ,R1 S0,uint D0
#define M1 ,S0,D0
#define H0(h) unpackUnorm4x8(S0.h[D0])
#define Y0(h) S0.h[D0]
#define T2(h) atomic_load_explicit(&S0.h[D0],memory_order::memory_order_relaxed)
#define x0(h,D) S0.h[D0]=packUnorm4x8(D)
#define c1(h,D) S0.h[D0]=(D)
#define U2(h,D) atomic_store_explicit(&S0.h[D0],D,memory_order::memory_order_relaxed)
#define v2(h)
#define d2(h)
#define W4(h,q) atomic_fetch_max_explicit(&S0.h[D0],q,memory_order::memory_order_relaxed)
#define X4(h,q) atomic_fetch_add_explicit(&S0.h[D0],q,memory_order::memory_order_relaxed)
#define w2
#define x2
#define U8(a) __attribute__((visibility("default")))fragment a(R1 S0,constant CC&n[[buffer(N1(F4))]],n0 Z[[stage_in]],A5 W0,Ua v6,S8 p3){d Y=Z.O0.xy;a1 J=a1(metal::floor(Y));uint D0=J.y*n.m6+J.x;
#define L1(a) void U8(a)
#define Y1 }
#define o2(a) i U8(a){i C1;
#define l3 }return C1;Y1
#else
#define I1 struct R1{
#define w0(f,a) [[color(f)]]i a
#define j1(f,a) [[color(f)]]uint a
#define D2 j1
#define J1 };
#define Q3 ,thread R1&B5,thread R1&S0
#define M1 ,B5,S0
#define H0(h) B5.h
#define Y0(h) B5.h
#define T2(h) Y0
#define x0(h,D) S0.h=(D)
#define c1(h,D) S0.h=(D)
#define U2(h) c1
#define v2(h) S0.h=B5.h
#define d2(h) S0.h=B5.h
e uint x5(thread uint&q0,uint x){uint V0=q0;q0=metal::max(V0,x);return V0;}
#define W4(h,q) x5(S0.h,q)
e uint y5(thread uint&q0,uint x){uint V0=q0;q0=V0+x;return V0;}
#define X4(h,q) y5(S0.h,q)
#define w2
#define x2
#define U8(a,...) R1 __attribute__((visibility("default")))fragment a(__VA_ARGS__){d Y[[maybe_unused]]=Z.O0.xy;R1 S0;
#define L1(a,...) U8(a,R1 B5,constant CC&n[[buffer(N1(F4))]],n0 Z[[stage_in]],Ua v6,A5 W0,S8 p3)
#define Y1 }return S0;
#define Pg(a,...) struct Ng{i Og[[j(0)]];R1 S0;};Ng __attribute__((visibility("default")))fragment a(__VA_ARGS__){d Y[[maybe_unused]]=Z.O0.xy;i C1;R1 S0;
#define o2(a) Pg(a,R1 B5,constant CC&n[[buffer(N1(F4))]],n0 Z[[stage_in]],A5 W0,S8 p3)
#define l3 }return{.Og=C1,.S0=S0};
#endif
#define o4 w0
#define discard discard_fragment()
using namespace metal;template<int P1>e vec<uint,P1>floatBitsToUint(vec<float,P1>x){return as_type<vec<uint,P1>>(x);}template<int P1>e vec<int,P1>floatBitsToInt(vec<float,P1>x){return as_type<vec<int,P1>>(x);}e uint floatBitsToUint(float x){return as_type<uint>(x);}e int floatBitsToInt(float x){return as_type<int>(x);}template<int P1>e vec<float,P1>uintBitsToFloat(vec<uint,P1>x){return as_type<vec<float,P1>>(x);}e float uintBitsToFloat(uint x){return as_type<float>(x);}e E unpackHalf2x16(uint x){return as_type<E>(x);}e uint packHalf2x16(E x){return as_type<uint>(x);}e i unpackUnorm4x8(uint x){return unpack_unorm4x8_to_half(x);}e uint packUnorm4x8(i x){return pack_half_to_unorm4x8(x);}e g0 inverse(g0 l1){g0 Va=g0(l1[1][1],-l1[0][1],-l1[1][0],l1[0][0]);float Qg=(Va[0][0]*l1[0][0])+(Va[0][1]*l1[1][0]);return Va*(1/Qg);}e v mix(v m,v b,n6 G1){v D7;for(int E0=0;E0<3;++E0)D7[E0]=G1[E0]?b[E0]:m[E0];return D7;}e d mix(d m,d b,D4 G1){d D7;for(int E0=0;E0<2;++E0)D7[E0]=G1[E0]?b[E0]:m[E0];return D7;}e d mix(d m,d b,float t){return mix(m,b,d(t));}e float mod(float x,float y){return fmod(x,y);}
