#ifndef _ARE_TOKEN_NAMES_PRESERVED
#define d half
#define E half2
#define A half3
#define i half4
#define N ushort
#define c float2
#define R float3
#define R3 packed_float3
#define f float4
#define J4 bool2
#define v6 bool3
#define B7 bool4
#define N0 uint2
#define Y uint4
#define Z int2
#define h6 int4
#define N ushort
#define e0 float2x2
#define d7 half3x3
#define e7 half2x3
#define K4 half4x4
#endif
#define e inline
#define c1(o2) thread o2&
#define N6(o2) thread o2&
#define equal(C,H) ((C)==(H))
#define notEqual(C,H) ((C)!=(H))
#define lessThan(C,H) ((C)<(H))
#define greaterThan(C,H) ((C)>(H))
#define P0(C,H) ((C)*(H))
#define inversesqrt rsqrt
#define A7(g,a) struct a{
#define P8(a) };
#define h1(a) struct a{
#define K(g,a0,a) a0 a
#define i1 };
#define L(S8,F,a,a0) a0 a=F[S8].a
#define q2 struct q0{
#define W(g,a0,a) a0 a
#define V2 [[flat]]
#define I0 [[center_no_perspective]]
#ifndef OPTIONALLY_FLAT
#define OPTIONALLY_FLAT
#endif
#define i2 f R0[[position]][[invariant]];};
#define U(a,a0) thread a0&a=f0.a
#define c0(a)
#define r(a,a0) a0 a=f0.a
#define F4 struct X8{
#define G4 };
#define U3 struct Y8{
#define V3 };
#define R5(g,y1,a) constant N0*a[[buffer(R1(g))]]
#define N4(g,y1,a) constant Y*a[[buffer(R1(g))]]
#define S5(g,y1,a) constant f*a[[buffer(R1(g))]]
#define L0(a,C0) y3.a[C0]
#define U5(a,C0) y3.a[C0]
#define Y3 struct Z8{
#define Z3 };
#define I3 struct H5{
#define J3 };
#define i5 struct fb{
#define j5 };
#define I4(V,g,a) [[texture(g)]]texture2d<uint>a
#define l5(V,g,a) [[texture(g)]]texture2d<float>a
#define e3(V,g,a) [[texture(g)]]texture2d<d>a
#define q5(V,g,a) [[texture(g)]]texture2d<d>a
#define l6(V,g,a) [[texture(g)]]texture1d_array<d>a
#define g4(C7,a) constexpr sampler a(filter::linear,mip_filter::none);
#define w6(V,g,a) [[sampler(g)]]sampler a;
#define c4(a) [[sampler(a4)]]sampler a;
#define v1(n0,m) Y0.n0.read(N0(m))
#define A5(n0,p,m) Y0.n0.sample(p,m)
#define j2(n0,p,m,U0) Y0.n0.sample(p,m,level(U0))
#define B5(n0,p,m,U1) Y0.n0.sample(p,m,bias(U1))
#define j8(n0,p,m) Y0.n0.sample(B6.p,m)
#define X6(n0,p,m,U0) Y0.n0.sample(B6.p,m,level(U0))
#define D7(n0,p,m,U1) Y0.n0.sample(B6.p,m,bias(U1))
#define c7(n0,p,q,x6,U8,U0) Y0.n0.sample(p,q,x6)
#define o6 ,constant SB&j,Z8 Y0,X8 y3
#define A3 ,j,Y0,y3
#ifdef ENABLE_INSTANCE_INDEX
#define B1(a,g0,F,B,v) __attribute__((visibility("default")))q0 vertex a(uint B[[vertex_id]],uint v[[instance_id]],constant uint&xh[[buffer(R1(dd))]],constant SB&j[[buffer(R1(M4))]],constant g0*F[[buffer(0)]],Z8 Y0,X8 y3){v+=xh;q0 f0;
#else
#define B1(a,g0,F,B,v) __attribute__((visibility("default")))q0 vertex a(uint B[[vertex_id]],uint v[[instance_id]],constant SB&j[[buffer(R1(M4))]],constant g0*F[[buffer(0)]],Z8 Y0,X8 y3){q0 f0;
#endif
#define V7(a,g0,F,p1,h0,B,v) __attribute__((visibility("default")))q0 vertex a(uint B[[vertex_id]],uint v[[instance_id]],constant SB&j[[buffer(R1(M4))]],constant g0*F[[buffer(0)]],const device p1*h0[[buffer(2)]],Z8 Y0,X8 y3){q0 f0;
#define L6(a,n3,o3,C3,D3,p1,h0,B) __attribute__((visibility("default")))q0 vertex a(uint B[[vertex_id]],uint v[[instance_id]],constant SB&j[[buffer(R1(M4))]],constant n3*o3[[buffer(0)]],constant C3*D3[[buffer(1)]],const device p1*h0[[buffer(2)]]){q0 f0;
#define C1(G5) f0.R0=G5;}return f0;
#define f3(z1,a) z1 __attribute__((visibility("default")))fragment a(q0 f0[[stage_in]],H5 Y0){
#define y6(z1,a) z1 __attribute__((visibility("default")))fragment a(q0 f0[[stage_in]],H5 Y0,bool z6[[front_facing]]){
#define M2(D) return D;}
#define O6 ,constant SB&j,c d0,H5 Y0,Y8 y3,fb B6
#define Y2 ,j,d0,Y0,y3,B6
#define L3 ,H5 Y0
#define e1 ,Y0
#define i7
#define C5
#ifdef PLS_IMPL_DEVICE_BUFFER
#define M1 struct V1{
#ifdef PLS_IMPL_DEVICE_BUFFER_RASTER_ORDERED
#define z0(g,a) device uint*a[[buffer(R1(g+i6)),raster_order_group(0)]]
#define k1(g,a) device uint*a[[buffer(R1(g+i6)),raster_order_group(0)]]
#define H2(g,a) device atomic_uint*a[[buffer(R1(g+i6)),raster_order_group(0)]]
#else
#define z0(g,a) device uint*a[[buffer(R1(g+i6))]]
#define k1(g,a) device uint*a[[buffer(R1(g+i6))]]
#define H2(g,a) device atomic_uint*a[[buffer(R1(g+i6))]]
#endif
#define N1 };
#define W3 ,V1 V0,uint G0
#define Q1 ,V0,G0
#define K0(h) unpackUnorm4x8(V0.h[G0])
#define a1(h) V0.h[G0]
#define Z2(h) atomic_load_explicit(&V0.h[G0],memory_order::memory_order_relaxed)
#define A0(h,D) V0.h[G0]=packUnorm4x8(D)
#define d1(h,D) V0.h[G0]=(D)
#define a3(h,D) atomic_store_explicit(&V0.h[G0],D,memory_order::memory_order_relaxed)
#define y2(h)
#define h2(h)
#define e5(h,q) atomic_fetch_max_explicit(&V0.h[G0],q,memory_order::memory_order_relaxed)
#define f5(h,q) atomic_fetch_add_explicit(&V0.h[G0],q,memory_order::memory_order_relaxed)
#define z2
#define A2
#define a9(a) __attribute__((visibility("default")))fragment a(V1 V0,constant SB&j[[buffer(R1(M4))]],q0 f0[[stage_in]],H5 Y0,fb B6,Y8 y3){c d0=f0.R0.xy;N0 G=N0(metal::floor(d0));uint G0=G.y*j.r6+G.x;
#define P1(a) void a9(a)
#define d2 }
#define v2(a) i a9(a){i F1;
#define r3 }return F1;d2
#else
#define M1 struct V1{
#define z0(g,a) [[color(g)]]i a
#define k1(g,a) [[color(g)]]uint a
#define H2 k1
#define N1 };
#define W3 ,thread V1&I5,thread V1&V0
#define Q1 ,I5,V0
#define K0(h) I5.h
#define a1(h) I5.h
#define Z2(h) a1
#define A0(h,D) V0.h=(D)
#define d1(h,D) V0.h=(D)
#define a3(h) d1
#define y2(h) V0.h=I5.h
#define h2(h) V0.h=I5.h
e uint E5(thread uint&w0,uint x){uint X0=w0;w0=metal::max(X0,x);return X0;}
#define e5(h,q) E5(V0.h,q)
e uint F5(thread uint&w0,uint x){uint X0=w0;w0=X0+x;return X0;}
#define f5(h,q) F5(V0.h,q)
#define z2
#define A2
#define a9(a,...) V1 __attribute__((visibility("default")))fragment a(__VA_ARGS__){c d0[[maybe_unused]]=f0.R0.xy;V1 V0;
#define P1(a,...) a9(a,V1 I5,constant SB&j[[buffer(R1(M4))]],q0 f0[[stage_in]],fb B6,H5 Y0,Y8 y3)
#define d2 }return V0;
#define Ah(a,...) struct yh{i zh[[k(0)]];V1 V0;};yh __attribute__((visibility("default")))fragment a(__VA_ARGS__){c d0[[maybe_unused]]=f0.R0.xy;i F1;V1 V0;
#define v2(a) Ah(a,V1 I5,constant SB&j[[buffer(R1(M4))]],q0 f0[[stage_in]],H5 Y0,Y8 y3)
#define r3 }return{.zh=F1,.V0=V0};
#endif
#define y4 z0
#define discard discard_fragment()
using namespace metal;template<int T1>e vec<uint,T1>floatBitsToUint(vec<float,T1>x){return as_type<vec<uint,T1>>(x);}template<int T1>e vec<int,T1>floatBitsToInt(vec<float,T1>x){return as_type<vec<int,T1>>(x);}e uint floatBitsToUint(float x){return as_type<uint>(x);}e int floatBitsToInt(float x){return as_type<int>(x);}template<int T1>e vec<float,T1>uintBitsToFloat(vec<uint,T1>x){return as_type<vec<float,T1>>(x);}e float uintBitsToFloat(uint x){return as_type<float>(x);}e E unpackHalf2x16(uint x){return as_type<E>(x);}e uint packHalf2x16(E x){return as_type<uint>(x);}e i unpackUnorm4x8(uint x){return unpack_unorm4x8_to_half(x);}e uint packUnorm4x8(i x){return pack_half_to_unorm4x8(x);}e c unpackUnorm2x16(uint x){return unpack_unorm2x16_to_float(x);}e e0 inverse(e0 n1){e0 gb=e0(n1[1][1],-n1[0][1],-n1[1][0],n1[0][0]);float Bh=(gb[0][0]*n1[0][0])+(gb[0][1]*n1[1][0]);return gb*(1/Bh);}e A mix(A l,A b,v6 J1){A I7;for(int H0=0;H0<3;++H0)I7[H0]=J1[H0]?b[H0]:l[H0];return I7;}e c mix(c l,c b,J4 J1){c I7;for(int H0=0;H0<2;++H0)I7[H0]=J1[H0]?b[H0]:l[H0];return I7;}e c mix(c l,c b,float t){return mix(l,b,c(t));}e float mod(float x,float y){return fmod(x,y);}