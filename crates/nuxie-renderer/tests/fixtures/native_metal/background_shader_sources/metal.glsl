#ifndef _ARE_TOKEN_NAMES_PRESERVED
#define c half
#define E half2
#define A half3
#define i half4
#define N ushort
#define d float2
#define R float3
#define N3 packed_float3
#define g float4
#define F4 bool2
#define q6 bool3
#define y7 bool4
#define a1 uint2
#define G uint4
#define Y int2
#define e6 int4
#define N ushort
#define f0 float2x2
#define Y6 half3x3
#define Z6 half2x3
#define j5 half4x4
#endif
#define e inline
#define Z0(l2) thread l2&
#define W4(l2) thread l2&
#define equal(C,H) ((C)==(H))
#define notEqual(C,H) ((C)!=(H))
#define lessThan(C,H) ((C)<(H))
#define greaterThan(C,H) ((C)>(H))
#define R0(C,H) ((C)*(H))
#define inversesqrt rsqrt
#define x7(f,a) struct a{
#define M8(a) };
#define g1(a) struct a{
#define L(f,d0,a) d0 a
#define h1 };
#define M(P8,F,a,d0) d0 a=F[P8].a
#define m2 struct o0{
#define X(f,d0,a) d0 a
#define Q2 [[flat]]
#define H0 [[center_no_perspective]]
#ifndef NB
#define NB
#endif
#define g2 g O0[[position]][[invariant]];};
#define V(a,d0) thread d0&a=c0.a
#define a0(a)
#define r(a,d0) d0 a=c0.a
#define B4 struct U8{
#define C4 };
#define Q3 struct V8{
#define R3 };
#define N5(f,w1,a) constant a1*a[[buffer(N1(f))]]
#define I4(f,w1,a) constant G*a[[buffer(N1(f))]]
#define O5(f,w1,a) constant g*a[[buffer(N1(f))]]
#define J0(a,A0) r3.a[A0]
#define P5(a,A0) r3.a[A0]
#define U3 struct W8{
#define V3 };
#define E3 struct C5{
#define F3 };
#define d5 struct Va{
#define e5 };
#define E4(U,f,a) [[texture(f)]]texture2d<uint>a
#define g5(U,f,a) [[texture(f)]]texture2d<float>a
#define Z2(U,f,a) [[texture(f)]]texture2d<c>a
#define m5(U,f,a) [[texture(f)]]texture2d<c>a
#define i6(U,f,a) [[texture(f)]]texture1d_array<c>a
#define c4(z7,a) constexpr sampler a(filter::linear,mip_filter::none);
#define r6(U,f,a) [[sampler(f)]]sampler a;
#define X3(a) [[sampler(W3)]]sampler a;
#define q1(k0,l) W0.k0.read(a1(l))
#define v5(k0,p,l) W0.k0.sample(p,l)
#define o2(k0,p,l,S0) W0.k0.sample(p,l,level(S0))
#define w5(k0,p,l,Q1) W0.k0.sample(p,l,bias(Q1))
#define g8(k0,p,l) W0.k0.sample(y6.p,l)
#define T6(k0,p,l,S0) W0.k0.sample(y6.p,l,level(S0))
#define A7(k0,p,l,Q1) W0.k0.sample(y6.p,l,bias(Q1))
#define X6(k0,p,q,v6,R8,S0) W0.k0.sample(p,q,v6)
#define l6 ,constant DC&m,W8 W0,U8 r3
#define w3 ,m,W0,r3
#ifdef PE
#define y1(a,e0,F,B,v) __attribute__((visibility("default")))o0 vertex a(uint B[[vertex_id]],uint v[[instance_id]],constant uint&Tg[[buffer(N1(Nc))]],constant DC&m[[buffer(N1(H4))]],constant e0*F[[buffer(0)]],W8 W0,U8 r3){v+=Tg;o0 c0;
#else
#define y1(a,e0,F,B,v) __attribute__((visibility("default")))o0 vertex a(uint B[[vertex_id]],uint v[[instance_id]],constant DC&m[[buffer(N1(H4))]],constant e0*F[[buffer(0)]],W8 W0,U8 r3){o0 c0;
#endif
#define S7(a,e0,F,n1,g0,B,v) __attribute__((visibility("default")))o0 vertex a(uint B[[vertex_id]],uint v[[instance_id]],constant DC&m[[buffer(N1(H4))]],constant e0*F[[buffer(0)]],const device n1*g0[[buffer(2)]],W8 W0,U8 r3){o0 c0;
#define I6(a,i3,j3,y3,z3,n1,g0,B) __attribute__((visibility("default")))o0 vertex a(uint B[[vertex_id]],uint v[[instance_id]],constant DC&m[[buffer(N1(H4))]],constant i3*j3[[buffer(0)]],constant y3*z3[[buffer(1)]],const device n1*g0[[buffer(2)]]){o0 c0;
#define z1(B5) c0.O0=B5;}return c0;
#define a3(S1,a) S1 __attribute__((visibility("default")))fragment a(o0 c0[[stage_in]],C5 W0){
#define w6(S1,a) S1 __attribute__((visibility("default")))fragment a(o0 c0[[stage_in]],C5 W0,bool x6[[front_facing]]){
#define I2(D) return D;}
#define K6 ,d Z,C5 W0,V8 r3,Va y6
#define U2 ,Z,W0,r3,y6
#define I3 ,C5 W0
#define d1 ,W0
#define e7
#define x5
#ifdef QF
#define I1 struct R1{
#ifdef RF
#define x0(f,a) device uint*a[[buffer(N1(f+f6)),raster_order_group(0)]]
#define j1(f,a) device uint*a[[buffer(N1(f+f6)),raster_order_group(0)]]
#define E2(f,a) device atomic_uint*a[[buffer(N1(f+f6)),raster_order_group(0)]]
#else
#define x0(f,a) device uint*a[[buffer(N1(f+f6))]]
#define j1(f,a) device uint*a[[buffer(N1(f+f6))]]
#define E2(f,a) device atomic_uint*a[[buffer(N1(f+f6))]]
#endif
#define J1 };
#define S3 ,R1 T0,uint E0
#define M1 ,T0,E0
#define I0(h) unpackUnorm4x8(T0.h[E0])
#define Y0(h) T0.h[E0]
#define V2(h) atomic_load_explicit(&T0.h[E0],memory_order::memory_order_relaxed)
#define y0(h,D) T0.h[E0]=packUnorm4x8(D)
#define c1(h,D) T0.h[E0]=(D)
#define W2(h,D) atomic_store_explicit(&T0.h[E0],D,memory_order::memory_order_relaxed)
#define w2(h)
#define e2(h)
#define Y4(h,q) atomic_fetch_max_explicit(&T0.h[E0],q,memory_order::memory_order_relaxed)
#define Z4(h,q) atomic_fetch_add_explicit(&T0.h[E0],q,memory_order::memory_order_relaxed)
#define x2
#define y2
#define X8(a) __attribute__((visibility("default")))fragment a(R1 T0,constant DC&m[[buffer(N1(H4))]],o0 c0[[stage_in]],C5 W0,Va y6,V8 r3){d Z=c0.O0.xy;a1 J=a1(metal::floor(Z));uint E0=J.y*m.p6+J.x;
#define L1(a) void X8(a)
#define Z1 }
#define p2(a) i X8(a){i C1;
#define n3 }return C1;Z1
#else
#define I1 struct R1{
#define x0(f,a) [[color(f)]]i a
#define j1(f,a) [[color(f)]]uint a
#define E2 j1
#define J1 };
#define S3 ,thread R1&D5,thread R1&T0
#define M1 ,D5,T0
#define I0(h) D5.h
#define Y0(h) D5.h
#define V2(h) Y0
#define y0(h,D) T0.h=(D)
#define c1(h,D) T0.h=(D)
#define W2(h) c1
#define w2(h) T0.h=D5.h
#define e2(h) T0.h=D5.h
e uint z5(thread uint&q0,uint x){uint V0=q0;q0=metal::max(V0,x);return V0;}
#define Y4(h,q) z5(T0.h,q)
e uint A5(thread uint&q0,uint x){uint V0=q0;q0=V0+x;return V0;}
#define Z4(h,q) A5(T0.h,q)
#define x2
#define y2
#define X8(a,...) R1 __attribute__((visibility("default")))fragment a(__VA_ARGS__){d Z[[maybe_unused]]=c0.O0.xy;R1 T0;
#define L1(a,...) X8(a,R1 D5,constant DC&m[[buffer(N1(H4))]],o0 c0[[stage_in]],Va y6,C5 W0,V8 r3)
#define Z1 }return T0;
#define Wg(a,...) struct Ug{i Vg[[j(0)]];R1 T0;};Ug __attribute__((visibility("default")))fragment a(__VA_ARGS__){d Z[[maybe_unused]]=c0.O0.xy;i C1;R1 T0;
#define p2(a) Wg(a,R1 D5,constant DC&m[[buffer(N1(H4))]],o0 c0[[stage_in]],C5 W0,V8 r3)
#define n3 }return{.Vg=C1,.T0=T0};
#endif
#define q4 x0
#define discard discard_fragment()
using namespace metal;template<int P1>e vec<uint,P1>floatBitsToUint(vec<float,P1>x){return as_type<vec<uint,P1>>(x);}template<int P1>e vec<int,P1>floatBitsToInt(vec<float,P1>x){return as_type<vec<int,P1>>(x);}e uint floatBitsToUint(float x){return as_type<uint>(x);}e int floatBitsToInt(float x){return as_type<int>(x);}template<int P1>e vec<float,P1>uintBitsToFloat(vec<uint,P1>x){return as_type<vec<float,P1>>(x);}e float uintBitsToFloat(uint x){return as_type<float>(x);}e E unpackHalf2x16(uint x){return as_type<E>(x);}e uint packHalf2x16(E x){return as_type<uint>(x);}e i unpackUnorm4x8(uint x){return unpack_unorm4x8_to_half(x);}e uint packUnorm4x8(i x){return pack_half_to_unorm4x8(x);}e f0 inverse(f0 l1){f0 Wa=f0(l1[1][1],-l1[0][1],-l1[1][0],l1[0][0]);float Xg=(Wa[0][0]*l1[0][0])+(Wa[0][1]*l1[1][0]);return Wa*(1/Xg);}e A mix(A o,A b,q6 G1){A G7;for(int F0=0;F0<3;++F0)G7[F0]=G1[F0]?b[F0]:o[F0];return G7;}e d mix(d o,d b,F4 G1){d G7;for(int F0=0;F0<2;++F0)G7[F0]=G1[F0]?b[F0]:o[F0];return G7;}e d mix(d o,d b,float t){return mix(o,b,d(t));}e float mod(float x,float y){return fmod(x,y);}
