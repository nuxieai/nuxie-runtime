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
#define p6 bool3
#define x7 bool4
#define a1 uint2
#define G uint4
#define Y int2
#define d6 int4
#define N ushort
#define f0 float2x2
#define X6 half3x3
#define Y6 half2x3
#define i5 half4x4
#endif
#define e inline
#define Z0(k2) thread k2&
#define V4(k2) thread k2&
#define equal(C,H) ((C)==(H))
#define notEqual(C,H) ((C)!=(H))
#define lessThan(C,H) ((C)<(H))
#define greaterThan(C,H) ((C)>(H))
#define R0(C,H) ((C)*(H))
#define inversesqrt rsqrt
#define w7(f,a) struct a{
#define L8(a) };
#define g1(a) struct a{
#define L(f,d0,a) d0 a
#define h1 };
#define M(O8,F,a,d0) d0 a=F[O8].a
#define l2 struct o0{
#define X(f,d0,a) d0 a
#define Q2 [[flat]]
#define H0 [[center_no_perspective]]
#ifndef NB
#define NB
#endif
#define f2 g O0[[position]][[invariant]];};
#define V(a,d0) thread d0&a=c0.a
#define a0(a)
#define r(a,d0) d0 a=c0.a
#define B4 struct T8{
#define C4 };
#define Q3 struct U8{
#define R3 };
#define M5(f,w1,a) constant a1*a[[buffer(N1(f))]]
#define I4(f,w1,a) constant G*a[[buffer(N1(f))]]
#define N5(f,w1,a) constant g*a[[buffer(N1(f))]]
#define J0(a,A0) r3.a[A0]
#define O5(a,A0) r3.a[A0]
#define U3 struct V8{
#define V3 };
#define E3 struct B5{
#define F3 };
#define c5 struct Ua{
#define d5 };
#define E4(U,f,a) [[texture(f)]]texture2d<uint>a
#define f5(U,f,a) [[texture(f)]]texture2d<float>a
#define Z2(U,f,a) [[texture(f)]]texture2d<c>a
#define l5(U,f,a) [[texture(f)]]texture2d<c>a
#define h6(U,f,a) [[texture(f)]]texture1d_array<c>a
#define c4(y7,a) constexpr sampler a(filter::linear,mip_filter::none);
#define q6(U,f,a) [[sampler(f)]]sampler a;
#define X3(a) [[sampler(W3)]]sampler a;
#define q1(k0,l) W0.k0.read(a1(l))
#define r5(k0,p,l) W0.k0.sample(p,l)
#define n2(k0,p,l,S0) W0.k0.sample(p,l,level(S0))
#define v5(k0,p,l,Q1) W0.k0.sample(p,l,bias(Q1))
#define f8(k0,p,l) W0.k0.sample(x6.p,l)
#define S6(k0,p,l,S0) W0.k0.sample(x6.p,l,level(S0))
#define z7(k0,p,l,Q1) W0.k0.sample(x6.p,l,bias(Q1))
#define W6(k0,p,q,r6,Q8,S0) W0.k0.sample(p,q,r6)
#define k6 ,constant DC&m,V8 W0,T8 r3
#define w3 ,m,W0,r3
#ifdef PE
#define y1(a,e0,F,B,v) __attribute__((visibility("default")))o0 vertex a(uint B[[vertex_id]],uint v[[instance_id]],constant uint&Sg[[buffer(N1(Mc))]],constant DC&m[[buffer(N1(H4))]],constant e0*F[[buffer(0)]],V8 W0,T8 r3){v+=Sg;o0 c0;
#else
#define y1(a,e0,F,B,v) __attribute__((visibility("default")))o0 vertex a(uint B[[vertex_id]],uint v[[instance_id]],constant DC&m[[buffer(N1(H4))]],constant e0*F[[buffer(0)]],V8 W0,T8 r3){o0 c0;
#endif
#define R7(a,e0,F,n1,g0,B,v) __attribute__((visibility("default")))o0 vertex a(uint B[[vertex_id]],uint v[[instance_id]],constant DC&m[[buffer(N1(H4))]],constant e0*F[[buffer(0)]],constant n1*g0[[buffer(2)]],V8 W0,T8 r3){o0 c0;
#define H6(a,i3,j3,y3,z3,n1,g0,B) __attribute__((visibility("default")))o0 vertex a(uint B[[vertex_id]],uint v[[instance_id]],constant DC&m[[buffer(N1(H4))]],constant i3*j3[[buffer(0)]],constant y3*z3[[buffer(1)]],constant n1*g0[[buffer(2)]]){o0 c0;
#define z1(A5) c0.O0=A5;}return c0;
#define a3(S1,a) S1 __attribute__((visibility("default")))fragment a(o0 c0[[stage_in]],B5 W0){
#define v6(S1,a) S1 __attribute__((visibility("default")))fragment a(o0 c0[[stage_in]],B5 W0,bool w6[[front_facing]]){
#define I2(D) return D;}
#define J6 ,d Z,B5 W0,U8 r3,Ua x6
#define U2 ,Z,W0,r3,x6
#define I3 ,B5 W0
#define d1 ,W0
#define d7
#define w5
#ifdef QF
#define I1 struct R1{
#ifdef RF
#define w0(f,a) device uint*a[[buffer(N1(f+e6)),raster_order_group(0)]]
#define j1(f,a) device uint*a[[buffer(N1(f+e6)),raster_order_group(0)]]
#define E2(f,a) device atomic_uint*a[[buffer(N1(f+e6)),raster_order_group(0)]]
#else
#define w0(f,a) device uint*a[[buffer(N1(f+e6))]]
#define j1(f,a) device uint*a[[buffer(N1(f+e6))]]
#define E2(f,a) device atomic_uint*a[[buffer(N1(f+e6))]]
#endif
#define J1 };
#define S3 ,R1 T0,uint D0
#define M1 ,T0,D0
#define I0(h) unpackUnorm4x8(T0.h[D0])
#define Y0(h) T0.h[D0]
#define V2(h) atomic_load_explicit(&T0.h[D0],memory_order::memory_order_relaxed)
#define x0(h,D) T0.h[D0]=packUnorm4x8(D)
#define c1(h,D) T0.h[D0]=(D)
#define W2(h,D) atomic_store_explicit(&T0.h[D0],D,memory_order::memory_order_relaxed)
#define v2(h)
#define d2(h)
#define X4(h,q) atomic_fetch_max_explicit(&T0.h[D0],q,memory_order::memory_order_relaxed)
#define Y4(h,q) atomic_fetch_add_explicit(&T0.h[D0],q,memory_order::memory_order_relaxed)
#define w2
#define x2
#define W8(a) __attribute__((visibility("default")))fragment a(R1 T0,constant DC&m[[buffer(N1(H4))]],o0 c0[[stage_in]],B5 W0,Ua x6,U8 r3){d Z=c0.O0.xy;a1 J=a1(metal::floor(Z));uint D0=J.y*m.o6+J.x;
#define L1(a) void W8(a)
#define Y1 }
#define o2(a) i W8(a){i C1;
#define n3 }return C1;Y1
#else
#define I1 struct R1{
#define w0(f,a) [[color(f)]]i a
#define j1(f,a) [[color(f)]]uint a
#define E2 j1
#define J1 };
#define S3 ,thread R1&C5,thread R1&T0
#define M1 ,C5,T0
#define I0(h) C5.h
#define Y0(h) C5.h
#define V2(h) Y0
#define x0(h,D) T0.h=(D)
#define c1(h,D) T0.h=(D)
#define W2(h) c1
#define v2(h) T0.h=C5.h
#define d2(h) T0.h=C5.h
e uint y5(thread uint&q0,uint x){uint V0=q0;q0=metal::max(V0,x);return V0;}
#define X4(h,q) y5(T0.h,q)
e uint z5(thread uint&q0,uint x){uint V0=q0;q0=V0+x;return V0;}
#define Y4(h,q) z5(T0.h,q)
#define w2
#define x2
#define W8(a,...) R1 __attribute__((visibility("default")))fragment a(__VA_ARGS__){d Z[[maybe_unused]]=c0.O0.xy;R1 T0;
#define L1(a,...) W8(a,R1 C5,constant DC&m[[buffer(N1(H4))]],o0 c0[[stage_in]],Ua x6,B5 W0,U8 r3)
#define Y1 }return T0;
#define Vg(a,...) struct Tg{i Ug[[j(0)]];R1 T0;};Tg __attribute__((visibility("default")))fragment a(__VA_ARGS__){d Z[[maybe_unused]]=c0.O0.xy;i C1;R1 T0;
#define o2(a) Vg(a,R1 C5,constant DC&m[[buffer(N1(H4))]],o0 c0[[stage_in]],B5 W0,U8 r3)
#define n3 }return{.Ug=C1,.T0=T0};
#endif
#define q4 w0
#define discard discard_fragment()
using namespace metal;template<int P1>e vec<uint,P1>floatBitsToUint(vec<float,P1>x){return as_type<vec<uint,P1>>(x);}template<int P1>e vec<int,P1>floatBitsToInt(vec<float,P1>x){return as_type<vec<int,P1>>(x);}e uint floatBitsToUint(float x){return as_type<uint>(x);}e int floatBitsToInt(float x){return as_type<int>(x);}template<int P1>e vec<float,P1>uintBitsToFloat(vec<uint,P1>x){return as_type<vec<float,P1>>(x);}e float uintBitsToFloat(uint x){return as_type<float>(x);}e E unpackHalf2x16(uint x){return as_type<E>(x);}e uint packHalf2x16(E x){return as_type<uint>(x);}e i unpackUnorm4x8(uint x){return unpack_unorm4x8_to_half(x);}e uint packUnorm4x8(i x){return pack_half_to_unorm4x8(x);}e f0 inverse(f0 l1){f0 Va=f0(l1[1][1],-l1[0][1],-l1[1][0],l1[0][0]);float Wg=(Va[0][0]*l1[0][0])+(Va[0][1]*l1[1][0]);return Va*(1/Wg);}e A mix(A o,A b,p6 G1){A F7;for(int E0=0;E0<3;++E0)F7[E0]=G1[E0]?b[E0]:o[E0];return F7;}e d mix(d o,d b,F4 G1){d F7;for(int E0=0;E0<2;++E0)F7[E0]=G1[E0]?b[E0]:o[E0];return F7;}e d mix(d o,d b,float t){return mix(o,b,d(t));}e float mod(float x,float y){return fmod(x,y);}
