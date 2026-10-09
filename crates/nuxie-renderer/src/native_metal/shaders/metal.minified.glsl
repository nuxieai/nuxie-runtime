#ifndef _ARE_TOKEN_NAMES_PRESERVED
#define d half
#define D half2
#define v half3
#define i half4
#define P ushort
#define c float2
#define M float3
#define i4 packed_float3
#define f float4
#define c5 bool2
#define Q6 bool3
#define d8 bool4
#define R0 uint2
#define O uint4
#define g0 int2
#define x6 int4
#define P ushort
#define X float2x2
#define z7 half3x3
#define A7 half2x3
#define d5 half4x4
#endif
#define e inline
#define k1(D2) thread D2&
#define i7(D2) thread D2&
#define equal(A,J) ((A)==(J))
#define notEqual(A,J) ((A)!=(J))
#define lessThan(A,J) ((A)<(J))
#define greaterThan(A,J) ((A)>(J))
#define B0(A,J) ((A)*(J))
#define inversesqrt rsqrt
#define c8(g,a) struct a{
#define L9(a) };
#define d1(a) struct a{
#define K(g,k0,a) k0 a
#define e1 };
#define L(O9,B,a,k0) k0 a=B[O9].a
#define v2 struct v0{
#define W(g,k0,a) k0 a
#define g3 [[flat]]
#define F0 [[center_no_perspective]]
#ifndef OPTIONALLY_FLAT
#define OPTIONALLY_FLAT
#endif
#define k2 f X0[[position]][[invariant]];};
#define V(a,k0) thread k0&a=h0.a
#define Z(a)
#define q(a,k0) k0 a=h0.a
#define Y4 struct T9{
#define Z4 };
#define k4 struct U9{
#define l4 };
#define j6(g,G1,a) constant R0*a[[buffer(Z1(g))]]
#define j5(g,G1,a) constant O*a[[buffer(Z1(g))]]
#define k6(g,G1,a) constant f*a[[buffer(Z1(g))]]
#define p0(a,E0) O3.a[E0]
#define w5(a,E0) O3.a[E0]
#define q4 struct V9{
#define r4 };
#define V3 struct X5{
#define W3 };
#define B5 struct lc{
#define C5 };
#define i5(e0,g,a) [[texture(g)]]texture2d<uint>a
#define R6(e0,g,a) [[texture(g)]]texture2d<float>a
#define p3(e0,g,a) [[texture(g)]]texture2d<d>a
#define M5(e0,g,a) [[texture(g)]]texture2d<d>a
#define I6(e0,g,a) [[texture(g)]]texture1d_array<d>a
#define a4(Q5,a) constexpr sampler a(filter::linear,mip_filter::none);
#define J6(Q5,a) constexpr sampler a(filter::linear,mip_filter::none,address::repeat);
#define S6(e0,g,a) [[sampler(g)]]sampler a;
#define w4(a) [[sampler(v4)]]sampler a;
#define q1(q0,o) h1.q0.read(R0(o))
#define R5(q0,p,o) h1.q0.sample(p,o)
#define n2(q0,p,o,a1) h1.q0.sample(p,o,level(a1))
#define S5(q0,p,o,f2) h1.q0.sample(p,o,bias(f2))
#define T8(q0,p,o) h1.q0.sample(W6.p,o)
#define D5(q0,p,o,a1) h1.q0.sample(W6.p,o,level(a1))
#define e8(q0,p,o,f2) h1.q0.sample(W6.p,o,bias(f2))
#define y7(q0,p,E,T6,Q9,a1) h1.q0.sample(p,E,T6)
#define M6 ,constant VB&j,V9 h1,T9 O3
#define Q3 ,j,h1,O3
#ifdef ENABLE_INSTANCE_INDEX
#define w1(a,f0,B,F,r) __attribute__((visibility("default"))) v0 vertex a(uint F[[vertex_id]],uint r[[instance_id]],constant uint&Ki[[buffer(Z1(Zd))]],constant VB&j[[buffer(Z1(e5))]],constant f0*B[[buffer(0)]],V9 h1,T9 O3){r+=Ki;v0 h0;
#else
#define w1(a,f0,B,F,r) __attribute__((visibility("default"))) v0 vertex a(uint F[[vertex_id]],uint r[[instance_id]],constant VB&j[[buffer(Z1(e5))]],constant f0*B[[buffer(0)]],V9 h1,T9 O3){v0 h0;
#endif
#define C8(a,f0,B,C1,j0,F,r) __attribute__((visibility("default"))) v0 vertex a(uint F[[vertex_id]],uint r[[instance_id]],constant VB&j[[buffer(Z1(e5))]],constant f0*B[[buffer(0)]],const device C1*j0[[buffer(2)]],V9 h1,T9 O3){v0 h0;
#define g7(a,C3,D3,T3,i3,C1,j0,F) __attribute__((visibility("default"))) v0 vertex a(uint F[[vertex_id]],uint r[[instance_id]],constant VB&j[[buffer(Z1(e5))]],constant C3*D3[[buffer(0)]],constant T3*i3[[buffer(1)]],const device C1*j0[[buffer(2)]]){v0 h0;
#define x1(W5) h0.X0=W5;}return h0;
#define V2(H1,a) H1 __attribute__((visibility("default"))) fragment a(v0 h0[[stage_in]],X5 h1){
#define U6(H1,a) H1 __attribute__((visibility("default"))) fragment a(v0 h0[[stage_in]],X5 h1,bool V6[[front_facing]]){
#define K2(C) return C;}
#define j7 ,constant VB&j,c d0,X5 h1,U9 O3,lc W6
#define l3 ,j,d0,h1,O3,W6
#define c4 ,X5 h1
#define m1 ,h1
#define E7
#define h5
#ifdef PLS_IMPL_DEVICE_BUFFER
#define U1 struct h2{
#ifdef PLS_IMPL_DEVICE_BUFFER_RASTER_ORDERED
#define C0(g,a) device uint*a[[buffer(Z1(g+z6)),raster_order_group(0)]]
#define p1(g,a) device uint*a[[buffer(Z1(g+z6)),raster_order_group(0)]]
#define U2(g,a) device atomic_uint*a[[buffer(Z1(g+z6)),raster_order_group(0)]]
#else
#define C0(g,a) device uint*a[[buffer(Z1(g+z6))]]
#define p1(g,a) device uint*a[[buffer(Z1(g+z6))]]
#define U2(g,a) device atomic_uint*a[[buffer(Z1(g+z6))]]
#endif
#define V1 };
#define m4 ,h2 c1,uint L0
#define Y1 ,c1,L0
#define Q0(h) unpackUnorm4x8(c1.h[L0])
#define j1(h) c1.h[L0]
#define m3(h) atomic_load_explicit(&c1.h[L0],memory_order::memory_order_relaxed)
#define y0(h,C) c1.h[L0]=packUnorm4x8(C)
#define l1(h,C) c1.h[L0]=(C)
#define n3(h,C) atomic_store_explicit(&c1.h[L0],C,memory_order::memory_order_relaxed)
#define M2(h)
#define g2(h)
#define y5(h,E) atomic_fetch_max_explicit(&c1.h[L0],E,memory_order::memory_order_relaxed)
#define z5(h,E) atomic_fetch_add_explicit(&c1.h[L0],E,memory_order::memory_order_relaxed)
#define N2
#define O2
#define W9(a) __attribute__((visibility("default"))) fragment a(h2 c1,constant VB&j[[buffer(Z1(e5))]],v0 h0[[stage_in]],X5 h1,lc W6,U9 O3){c d0=h0.X0.xy;R0 G=R0(metal::floor(d0));uint L0=G.y*j.P6+G.x;
#define X1(a) void W9(a)
#define o2 }
#define G2(a) i W9(a){i L1;
#define E3 }return L1;o2
#else
#define U1 struct h2{
#define C0(g,a) [[color(g)]]i a
#define p1(g,a) [[color(g)]]uint a
#define U2 p1
#define V1 };
#define m4 ,thread h2&Y5,thread h2&c1
#define Y1 ,Y5,c1
#define Q0(h) Y5.h
#define j1(h) Y5.h
#define m3(h) j1
#define y0(h,C) c1.h=(C)
#define l1(h,C) c1.h=(C)
#define n3(h) l1
#define M2(h) c1.h=Y5.h
#define g2(h) c1.h=Y5.h
e uint U5(thread uint&x0,uint x){uint g1=x0;x0=metal::max(g1,x);return g1;}
#define y5(h,E) U5(c1.h,E)
e uint V5(thread uint&x0,uint x){uint g1=x0;x0=g1+x;return g1;}
#define z5(h,E) V5(c1.h,E)
#define N2
#define O2
#define W9(a,...) h2 __attribute__((visibility("default"))) fragment a(__VA_ARGS__){c d0[[maybe_unused]]=h0.X0.xy;h2 c1;
#define X1(a,...) W9(a,h2 Y5,constant VB&j[[buffer(Z1(e5))]],v0 h0[[stage_in]],lc W6,X5 h1,U9 O3)
#define o2 }return c1;
#define Ni(a,...) struct Li{i Mi[[l(0)]];h2 c1;};Li __attribute__((visibility("default"))) fragment a(__VA_ARGS__){c d0[[maybe_unused]]=h0.X0.xy;i L1;h2 c1;
#define G2(a) Ni(a,h2 Y5,constant VB&j[[buffer(Z1(e5))]],v0 h0[[stage_in]],X5 h1,U9 O3)
#define E3 }return{.Mi=L1,.c1=c1};
#endif
#define P4 C0
#define discard discard_fragment()
using namespace metal;template<int e2>e vec<uint,e2>floatBitsToUint(vec<float,e2>x){return as_type<vec<uint,e2>>(x);}template<int e2>e vec<int,e2>floatBitsToInt(vec<float,e2>x){return as_type<vec<int,e2>>(x);}e uint floatBitsToUint(float x){return as_type<uint>(x);}e int floatBitsToInt(float x){return as_type<int>(x);}template<int e2>e vec<float,e2>uintBitsToFloat(vec<uint,e2>x){return as_type<vec<float,e2>>(x);}e float uintBitsToFloat(uint x){return as_type<float>(x);}e D unpackHalf2x16(uint x){return as_type<D>(x);}e uint packHalf2x16(D x){return as_type<uint>(x);}e i unpackUnorm4x8(uint x){return unpack_unorm4x8_to_half(x);}e uint packUnorm4x8(i x){return pack_half_to_unorm4x8(x);}e c unpackUnorm2x16(uint x){return unpack_unorm2x16_to_float(x);}e X inverse(X A1){X mc=X(A1[1][1],-A1[0][1],-A1[1][0],A1[0][0]);float Oi=(mc[0][0]*A1[0][0])+(mc[0][1]*A1[1][0]);return mc*(1/Oi);}e v mix(v m,v b,Q6 R1){v l8;for(int M0=0;M0<3;++M0) l8[M0]=R1[M0]?b[M0]:m[M0];return l8;}e c mix(c m,c b,c5 R1){c l8;for(int M0=0;M0<2;++M0) l8[M0]=R1[M0]?b[M0]:m[M0];return l8;}e c mix(c m,c b,float t){return mix(m,b,c(t));}e float mod(float x,float y){return fmod(x,y);}