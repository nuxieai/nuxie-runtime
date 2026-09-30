#ifndef _ARE_TOKEN_NAMES_PRESERVED
#define d half
#define E half2
#define A half3
#define i half4
#define L ushort
#define c float2
#define R float3
#define R3 packed_float3
#define f float4
#define I4 bool2
#define r6 bool3
#define A7 bool4
#define c1 uint2
#define Y uint4
#define Z int2
#define g6 int4
#define L ushort
#define e0 float2x2
#define c7 half3x3
#define d7 half2x3
#define J4 half4x4
#endif
#define e inline
#define a1(o2) thread o2&
#define M6(o2) thread o2&
#define equal(C,H) ((C)==(H))
#define notEqual(C,H) ((C)!=(H))
#define lessThan(C,H) ((C)<(H))
#define greaterThan(C,H) ((C)>(H))
#define O0(C,H) ((C)*(H))
#define inversesqrt rsqrt
#define z7(g,a) struct a{
#define P8(a) };
#define h1(a) struct a{
#define J(g,a0,a) a0 a
#define i1 };
#define K(S8,F,a,a0) a0 a=F[S8].a
#define q2 struct p0{
#define W(g,a0,a) a0 a
#define T2 [[flat]]
#define I0 [[center_no_perspective]]
#ifndef OPTIONALLY_FLAT
#define OPTIONALLY_FLAT
#endif
#define i2 f Q0[[position]][[invariant]];};
#define U(a,a0) thread a0&a=f0.a
#define c0(a)
#define r(a,a0) a0 a=f0.a
#define E4 struct X8{
#define F4 };
#define T3 struct Y8{
#define U3 };
#define Q5(g,x1,a) constant c1*a[[buffer(Q1(g))]]
#define M4(g,x1,a) constant Y*a[[buffer(Q1(g))]]
#define R5(g,x1,a) constant f*a[[buffer(Q1(g))]]
#define L0(a,C0) y3.a[C0]
#define T5(a,C0) y3.a[C0]
#define X3 struct Z8{
#define Y3 };
#define I3 struct G5{
#define J3 };
#define h5 struct db{
#define i5 };
#define H4(V,g,a) [[texture(g)]]texture2d<uint>a
#define k5(V,g,a) [[texture(g)]]texture2d<float>a
#define c3(V,g,a) [[texture(g)]]texture2d<d>a
#define p5(V,g,a) [[texture(g)]]texture2d<d>a
#define k6(V,g,a) [[texture(g)]]texture1d_array<d>a
#define f4(B7,a) constexpr sampler a(filter::linear,mip_filter::none);
#define v6(V,g,a) [[sampler(g)]]sampler a;
#define a4(a) [[sampler(Z3)]]sampler a;
#define r1(m0,l) X0.m0.read(c1(l))
#define z5(m0,p,l) X0.m0.sample(p,l)
#define j2(m0,p,l,T0) X0.m0.sample(p,l,level(T0))
#define A5(m0,p,l,T1) X0.m0.sample(p,l,bias(T1))
#define j8(m0,p,l) X0.m0.sample(A6.p,l)
#define W6(m0,p,l,T0) X0.m0.sample(A6.p,l,level(T0))
#define C7(m0,p,l,T1) X0.m0.sample(A6.p,l,bias(T1))
#define a7(m0,p,q,w6,U8,T0) X0.m0.sample(p,q,w6)
#define n6 ,constant AC&j,Z8 X0,X8 y3
#define A3 ,j,X0,y3
#ifdef ENABLE_INSTANCE_INDEX
#define A1(a,g0,F,B,v) __attribute__((visibility("default")))p0 vertex a(uint B[[vertex_id]],uint v[[instance_id]],constant uint&rh[[buffer(Q1(Yc))]],constant AC&j[[buffer(Q1(L4))]],constant g0*F[[buffer(0)]],Z8 X0,X8 y3){v+=rh;p0 f0;
#else
#define A1(a,g0,F,B,v) __attribute__((visibility("default")))p0 vertex a(uint B[[vertex_id]],uint v[[instance_id]],constant AC&j[[buffer(Q1(L4))]],constant g0*F[[buffer(0)]],Z8 X0,X8 y3){p0 f0;
#endif
#define V7(a,g0,F,o1,h0,B,v) __attribute__((visibility("default")))p0 vertex a(uint B[[vertex_id]],uint v[[instance_id]],constant AC&j[[buffer(Q1(L4))]],constant g0*F[[buffer(0)]],const device o1*h0[[buffer(2)]],Z8 X0,X8 y3){p0 f0;
#define K6(a,l3,m3,C3,D3,o1,h0,B) __attribute__((visibility("default")))p0 vertex a(uint B[[vertex_id]],uint v[[instance_id]],constant AC&j[[buffer(Q1(L4))]],constant l3*m3[[buffer(0)]],constant C3*D3[[buffer(1)]],const device o1*h0[[buffer(2)]]){p0 f0;
#define B1(F5) f0.Q0=F5;}return f0;
#define d3(y1,a) y1 __attribute__((visibility("default")))fragment a(p0 f0[[stage_in]],G5 X0){
#define x6(y1,a) y1 __attribute__((visibility("default")))fragment a(p0 f0[[stage_in]],G5 X0,bool y6[[front_facing]]){
#define L2(D) return D;}
#define N6 ,c d0,G5 X0,Y8 y3,db A6
#define W2 ,d0,X0,y3,A6
#define L3 ,G5 X0
#define e1 ,X0
#define h7
#define B5
#ifdef PLS_IMPL_DEVICE_BUFFER
#define L1 struct U1{
#ifdef PLS_IMPL_DEVICE_BUFFER_RASTER_ORDERED
#define z0(g,a) device uint*a[[buffer(Q1(g+h6)),raster_order_group(0)]]
#define k1(g,a) device uint*a[[buffer(Q1(g+h6)),raster_order_group(0)]]
#define H2(g,a) device atomic_uint*a[[buffer(Q1(g+h6)),raster_order_group(0)]]
#else
#define z0(g,a) device uint*a[[buffer(Q1(g+h6))]]
#define k1(g,a) device uint*a[[buffer(Q1(g+h6))]]
#define H2(g,a) device atomic_uint*a[[buffer(Q1(g+h6))]]
#endif
#define M1 };
#define V3 ,U1 U0,uint G0
#define P1 ,U0,G0
#define K0(h) unpackUnorm4x8(U0.h[G0])
#define Z0(h) U0.h[G0]
#define X2(h) atomic_load_explicit(&U0.h[G0],memory_order::memory_order_relaxed)
#define A0(h,D) U0.h[G0]=packUnorm4x8(D)
#define d1(h,D) U0.h[G0]=(D)
#define Y2(h,D) atomic_store_explicit(&U0.h[G0],D,memory_order::memory_order_relaxed)
#define y2(h)
#define h2(h)
#define d5(h,q) atomic_fetch_max_explicit(&U0.h[G0],q,memory_order::memory_order_relaxed)
#define e5(h,q) atomic_fetch_add_explicit(&U0.h[G0],q,memory_order::memory_order_relaxed)
#define z2
#define A2
#define a9(a) __attribute__((visibility("default")))fragment a(U1 U0,constant AC&j[[buffer(Q1(L4))]],p0 f0[[stage_in]],G5 X0,db A6,Y8 y3){c d0=f0.Q0.xy;c1 G=c1(metal::floor(d0));uint G0=G.y*j.q6+G.x;
#define O1(a) void a9(a)
#define d2 }
#define v2(a) i a9(a){i E1;
#define p3 }return E1;d2
#else
#define L1 struct U1{
#define z0(g,a) [[color(g)]]i a
#define k1(g,a) [[color(g)]]uint a
#define H2 k1
#define M1 };
#define V3 ,thread U1&H5,thread U1&U0
#define P1 ,H5,U0
#define K0(h) H5.h
#define Z0(h) H5.h
#define X2(h) Z0
#define A0(h,D) U0.h=(D)
#define d1(h,D) U0.h=(D)
#define Y2(h) d1
#define y2(h) U0.h=H5.h
#define h2(h) U0.h=H5.h
e uint D5(thread uint&w0,uint x){uint W0=w0;w0=metal::max(W0,x);return W0;}
#define d5(h,q) D5(U0.h,q)
e uint E5(thread uint&w0,uint x){uint W0=w0;w0=W0+x;return W0;}
#define e5(h,q) E5(U0.h,q)
#define z2
#define A2
#define a9(a,...) U1 __attribute__((visibility("default")))fragment a(__VA_ARGS__){c d0[[maybe_unused]]=f0.Q0.xy;U1 U0;
#define O1(a,...) a9(a,U1 H5,constant AC&j[[buffer(Q1(L4))]],p0 f0[[stage_in]],db A6,G5 X0,Y8 y3)
#define d2 }return U0;
#define uh(a,...) struct sh{i th[[k(0)]];U1 U0;};sh __attribute__((visibility("default")))fragment a(__VA_ARGS__){c d0[[maybe_unused]]=f0.Q0.xy;i E1;U1 U0;
#define v2(a) uh(a,U1 H5,constant AC&j[[buffer(Q1(L4))]],p0 f0[[stage_in]],G5 X0,Y8 y3)
#define p3 }return{.th=E1,.U0=U0};
#endif
#define x4 z0
#define discard discard_fragment()
using namespace metal;template<int S1>e vec<uint,S1>floatBitsToUint(vec<float,S1>x){return as_type<vec<uint,S1>>(x);}template<int S1>e vec<int,S1>floatBitsToInt(vec<float,S1>x){return as_type<vec<int,S1>>(x);}e uint floatBitsToUint(float x){return as_type<uint>(x);}e int floatBitsToInt(float x){return as_type<int>(x);}template<int S1>e vec<float,S1>uintBitsToFloat(vec<uint,S1>x){return as_type<vec<float,S1>>(x);}e float uintBitsToFloat(uint x){return as_type<float>(x);}e E unpackHalf2x16(uint x){return as_type<E>(x);}e uint packHalf2x16(E x){return as_type<uint>(x);}e i unpackUnorm4x8(uint x){return unpack_unorm4x8_to_half(x);}e uint packUnorm4x8(i x){return pack_half_to_unorm4x8(x);}e e0 inverse(e0 m1){e0 eb=e0(m1[1][1],-m1[0][1],-m1[1][0],m1[0][0]);float vh=(eb[0][0]*m1[0][0])+(eb[0][1]*m1[1][0]);return eb*(1/vh);}e A mix(A n,A b,r6 I1){A H7;for(int H0=0;H0<3;++H0)H7[H0]=I1[H0]?b[H0]:n[H0];return H7;}e c mix(c n,c b,I4 I1){c H7;for(int H0=0;H0<2;++H0)H7[H0]=I1[H0]?b[H0]:n[H0];return H7;}e c mix(c n,c b,float t){return mix(n,b,c(t));}e float mod(float x,float y){return fmod(x,y);}