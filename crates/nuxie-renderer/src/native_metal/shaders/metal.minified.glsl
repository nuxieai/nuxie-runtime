#define re
#ifndef _ARE_TOKEN_NAMES_PRESERVED
#define d half
#define C half2
#define v half3
#define i half4
#define Q ushort
#define c float2
#define O float3
#define d4 packed_float3
#define e float4
#define S4 bool2
#define B6 bool3
#define H7 bool4
#define O0 uint2
#define M uint4
#define e0 int2
#define m6 int4
#define Q ushort
#define Y float2x2
#define k7 half3x3
#define l7 half2x3
#define T4 half4x4
#endif
#define f inline
#define i1(w2) thread w2&
#define V6(w2) thread w2&
#define equal(B,J) ((B)==(J))
#define notEqual(B,J) ((B)!=(J))
#define lessThan(B,J) ((B)<(J))
#define greaterThan(B,J) ((B)>(J))
#define M0(B,J) ((B)*(J))
#define inversesqrt rsqrt
#define G7(g,a) struct a{
#define e9(a) };
#define c1(a) struct a{
#define K(g,j0,a) j0 a
#define d1 };
#define L(h9,D,a,j0) j0 a=D[h9].a
#define l2 struct v0{
#define V(g,j0,a) j0 a
#define Z2 [[flat]]
#define E0 [[center_no_perspective]]
#ifndef OPTIONALLY_FLAT
#define OPTIONALLY_FLAT
#endif
#define e2 e U0[[position]][[invariant]];};
#define T(a,j0) thread j0&a=g0.a
#define Z(a)
#define q(a,j0) j0 a=g0.a
#define Q4 struct l9{
#define R4 };
#define g4 struct m9{
#define h4 };
#define Z5(g,F1,a) constant O0*a[[buffer(W1(g))]]
#define X4(g,F1,a) constant M*a[[buffer(W1(g))]]
#define a6(g,F1,a) constant e*a[[buffer(W1(g))]]
#define p0(a,D0) F3.a[D0]
#define l5(a,D0) F3.a[D0]
#define k4 struct n9{
#define l4 };
#define O3 struct Q5{
#define P3 };
#define x5 struct wb{
#define y5 };
#define W4(c0,g,a) [[texture(g)]]texture2d<uint>a
#define C6(c0,g,a) [[texture(g)]]texture2d<float>a
#define i3(c0,g,a) [[texture(g)]]texture2d<d>a
#define D5(c0,g,a) [[texture(g)]]texture2d<d>a
#define q6(c0,g,a) [[texture(g)]]texture1d_array<d>a
#define p4(I7,a) constexpr sampler a(filter::linear,mip_filter::none);
#define D6(c0,g,a) [[sampler(g)]]sampler a;
#define n4(a) [[sampler(m4)]]sampler a;
#define p1(q0,l) f1.q0.read(O0(l))
#define K5(q0,o,l) f1.q0.sample(o,l)
#define o2(q0,o,l,Y0) f1.q0.sample(o,l,level(Y0))
#define L5(q0,o,l,Z1) f1.q0.sample(o,l,bias(Z1))
#define v8(q0,o,l) f1.q0.sample(J6.o,l)
#define i6(q0,o,l,Y0) f1.q0.sample(J6.o,l,level(Y0))
#define J7(q0,o,l,Z1) f1.q0.sample(J6.o,l,bias(Z1))
#define j7(q0,o,F,E6,j9,Y0) f1.q0.sample(o,F,E6)
#define w6 ,constant UB&j,n9 f1,l9 F3
#define H3 ,j,f1,F3
#ifdef ENABLE_INSTANCE_INDEX
#define w1(a,d0,D,G,r) __attribute__((visibility("default"))) v0 vertex a(uint G[[vertex_id]],uint r[[instance_id]],constant uint&bi[[buffer(W1(xd))]],constant UB&j[[buffer(W1(V4))]],constant d0*D[[buffer(0)]],n9 f1,l9 F3){r+=bi;v0 g0;
#else
#define w1(a,d0,D,G,r) __attribute__((visibility("default"))) v0 vertex a(uint G[[vertex_id]],uint r[[instance_id]],constant UB&j[[buffer(W1(V4))]],constant d0*D[[buffer(0)]],n9 f1,l9 F3){v0 g0;
#endif
#define g8(a,d0,D,B1,h0,G,r) __attribute__((visibility("default"))) v0 vertex a(uint G[[vertex_id]],uint r[[instance_id]],constant UB&j[[buffer(W1(V4))]],constant d0*D[[buffer(0)]],const device B1*h0[[buffer(2)]],n9 f1,l9 F3){v0 g0;
#define T6(a,x3,y3,K3,L3,B1,h0,G) __attribute__((visibility("default"))) v0 vertex a(uint G[[vertex_id]],uint r[[instance_id]],constant UB&j[[buffer(W1(V4))]],constant x3*y3[[buffer(0)]],constant K3*L3[[buffer(1)]],const device B1*h0[[buffer(2)]]){v0 g0;
#define x1(P5) g0.U0=P5;}return g0;
#define j3(G1,a) G1 __attribute__((visibility("default"))) fragment a(v0 g0[[stage_in]],Q5 f1){
#define G6(G1,a) G1 __attribute__((visibility("default"))) fragment a(v0 g0[[stage_in]],Q5 f1,bool H6[[front_facing]]){
#define P2(E) return E;}
#define W6 ,constant UB&j,c f0,Q5 f1,m9 F3,wb J6
#define e3 ,j,f0,f1,F3,J6
#define S3 ,Q5 f1
#define k1 ,f1
#define p7
#define Z4
#ifdef PLS_IMPL_DEVICE_BUFFER
#define S1 struct c2{
#ifdef PLS_IMPL_DEVICE_BUFFER_RASTER_ORDERED
#define B0(g,a) device uint*a[[buffer(W1(g+n6)),raster_order_group(0)]]
#define o1(g,a) device uint*a[[buffer(W1(g+n6)),raster_order_group(0)]]
#define L2(g,a) device atomic_uint*a[[buffer(W1(g+n6)),raster_order_group(0)]]
#else
#define B0(g,a) device uint*a[[buffer(W1(g+n6))]]
#define o1(g,a) device uint*a[[buffer(W1(g+n6))]]
#define L2(g,a) device atomic_uint*a[[buffer(W1(g+n6))]]
#endif
#define T1 };
#define i4 ,c2 Z0,uint K0
#define V1 ,Z0,K0
#define N0(h) unpackUnorm4x8(Z0.h[K0])
#define h1(h) Z0.h[K0]
#define f3(h) atomic_load_explicit(&Z0.h[K0],memory_order::memory_order_relaxed)
#define y0(h,E) Z0.h[K0]=packUnorm4x8(E)
#define j1(h,E) Z0.h[K0]=(E)
#define g3(h,E) atomic_store_explicit(&Z0.h[K0],E,memory_order::memory_order_relaxed)
#define D2(h)
#define a2(h)
#define p5(h,F) atomic_fetch_max_explicit(&Z0.h[K0],F,memory_order::memory_order_relaxed)
#define q5(h,F) atomic_fetch_add_explicit(&Z0.h[K0],F,memory_order::memory_order_relaxed)
#define E2
#define F2
#define o9(a) __attribute__((visibility("default"))) fragment a(c2 Z0,constant UB&j[[buffer(W1(V4))]],v0 g0[[stage_in]],Q5 f1,wb J6,m9 F3){c f0=g0.U0.xy;O0 H=O0(metal::floor(f0));uint K0=H.y*j.A6+H.x;
#define U1(a) void o9(a)
#define h2 }
#define z2(a) i o9(a){i L1;
#define A3 }return L1;h2
#else
#define S1 struct c2{
#define B0(g,a) [[color(g)]]i a
#define o1(g,a) [[color(g)]]uint a
#define L2 o1
#define T1 };
#define i4 ,thread c2&R5,thread c2&Z0
#define V1 ,R5,Z0
#define N0(h) R5.h
#define h1(h) R5.h
#define f3(h) h1
#define y0(h,E) Z0.h=(E)
#define j1(h,E) Z0.h=(E)
#define g3(h) j1
#define D2(h) Z0.h=R5.h
#define a2(h) Z0.h=R5.h
f uint N5(thread uint&x0,uint x){uint e1=x0;x0=metal::max(e1,x);return e1;}
#define p5(h,F) N5(Z0.h,F)
f uint O5(thread uint&x0,uint x){uint e1=x0;x0=e1+x;return e1;}
#define q5(h,F) O5(Z0.h,F)
#define E2
#define F2
#define o9(a,...) c2 __attribute__((visibility("default"))) fragment a(__VA_ARGS__){c f0[[maybe_unused]]=g0.U0.xy;c2 Z0;
#define U1(a,...) o9(a,c2 R5,constant UB&j[[buffer(W1(V4))]],v0 g0[[stage_in]],wb J6,Q5 f1,m9 F3)
#define h2 }return Z0;
#define ei(a,...) struct ci{i di[[p(0)]];c2 Z0;};ci __attribute__((visibility("default"))) fragment a(__VA_ARGS__){c f0[[maybe_unused]]=g0.U0.xy;i L1;c2 Z0;
#define z2(a) ei(a,c2 R5,constant UB&j[[buffer(W1(V4))]],v0 g0[[stage_in]],Q5 f1,m9 F3)
#define A3 }return{.di=L1,.Z0=Z0};
#endif
#define J4 B0
#define discard discard_fragment()
using namespace metal;template<int Y1>f vec<uint,Y1>floatBitsToUint(vec<float,Y1>x){return as_type<vec<uint,Y1>>(x);}template<int Y1>f vec<int,Y1>floatBitsToInt(vec<float,Y1>x){return as_type<vec<int,Y1>>(x);}f uint floatBitsToUint(float x){return as_type<uint>(x);}f int floatBitsToInt(float x){return as_type<int>(x);}template<int Y1>f vec<float,Y1>uintBitsToFloat(vec<uint,Y1>x){return as_type<vec<float,Y1>>(x);}f float uintBitsToFloat(uint x){return as_type<float>(x);}f C unpackHalf2x16(uint x){return as_type<C>(x);}f uint packHalf2x16(C x){return as_type<uint>(x);}f i unpackUnorm4x8(uint x){return unpack_unorm4x8_to_half(x);}f uint packUnorm4x8(i x){return pack_half_to_unorm4x8(x);}f c unpackUnorm2x16(uint x){return unpack_unorm2x16_to_float(x);}f Y inverse(Y z1){Y xb=Y(z1[1][1],-z1[0][1],-z1[1][0],z1[0][0]);float fi=(xb[0][0]*z1[0][0])+(xb[0][1]*z1[1][0]);return xb*(1/fi);}f v mix(v k,v b,B6 P1){v P7;for(int L0=0;L0<3;++L0) P7[L0]=P1[L0]?b[L0]:k[L0];return P7;}f c mix(c k,c b,S4 P1){c P7;for(int L0=0;L0<2;++L0) P7[L0]=P1[L0]?b[L0]:k[L0];return P7;}f c mix(c k,c b,float t){return mix(k,b,c(t));}f float mod(float x,float y){return fmod(x,y);}