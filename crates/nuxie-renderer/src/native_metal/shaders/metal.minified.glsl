#ifndef _ARE_TOKEN_NAMES_PRESERVED
#define d half
#define D half2
#define v half3
#define i half4
#define P ushort
#define c float2
#define M float3
#define h4 packed_float3
#define e float4
#define Y4 bool2
#define M6 bool3
#define a8 bool4
#define S0 uint2
#define O uint4
#define g0 int2
#define q6 int4
#define P ushort
#define W float2x2
#define w7 half3x3
#define x7 half2x3
#define Z4 half4x4
#endif
#define f inline
#define c1(D2) thread D2&
#define e7(D2) thread D2&
#define equal(A,J) ((A)==(J))
#define notEqual(A,J) ((A)!=(J))
#define lessThan(A,J) ((A)<(J))
#define greaterThan(A,J) ((A)>(J))
#define y0(A,J) ((A)*(J))
#define inversesqrt rsqrt
#define Z7(g,a) struct a{
#define J9(a) };
#define f1(a) struct a{
#define K(g,k0,a) k0 a
#define g1 };
#define L(M9,B,a,k0) k0 a=B[M9].a
#define w2 struct v0{
#define X(g,k0,a) k0 a
#define g3 [[flat]]
#define F0 [[center_no_perspective]]
#ifndef OPTIONALLY_FLAT
#define OPTIONALLY_FLAT
#endif
#define l2 e Y0[[position]][[invariant]];};
#define V(a,k0) thread k0&a=h0.a
#define Z(a)
#define q(a,k0) k0 a=h0.a
#define W4 struct P9{
#define X4 };
#define k4 struct Q9{
#define l4 };
#define g6(g,H1,a) constant S0*a[[buffer(a2(g))]]
#define g5(g,H1,a) constant O*a[[buffer(a2(g))]]
#define h6(g,H1,a) constant e*a[[buffer(a2(g))]]
#define p0(a,E0) N3.a[E0]
#define q5(a,E0) N3.a[E0]
#define o4 struct R9{
#define p4 };
#define U3 struct U5{
#define V3 };
#define y5 struct fc{
#define z5 };
#define f5(e0,g,a) [[texture(g)]]texture2d<uint>a
#define N6(e0,g,a) [[texture(g)]]texture2d<float>a
#define p3(e0,g,a) [[texture(g)]]texture2d<d>a
#define K5(e0,g,a) [[texture(g)]]texture2d<d>a
#define F6(e0,g,a) [[texture(g)]]texture1d_array<d>a
#define y4(c8,a) constexpr sampler a(filter::linear,mip_filter::none);
#define O6(e0,g,a) [[sampler(g)]]sampler a;
#define r4(a) [[sampler(q4)]]sampler a;
#define r1(q0,o) j1.q0.read(S0(o))
#define O5(q0,p,o) j1.q0.sample(p,o)
#define o2(q0,p,o,d1) j1.q0.sample(p,o,level(d1))
#define P5(q0,p,o,g2) j1.q0.sample(p,o,bias(g2))
#define O8(q0,p,o) j1.q0.sample(S6.p,o)
#define A5(q0,p,o,d1) j1.q0.sample(S6.p,o,level(d1))
#define d8(q0,p,o,g2) j1.q0.sample(S6.p,o,bias(g2))
#define v7(q0,p,E,P6,O9,d1) j1.q0.sample(p,E,P6)
#define I6 ,constant VB&j,R9 j1,P9 N3
#define P3 ,j,j1,N3
#ifdef ENABLE_INSTANCE_INDEX
#define x1(a,f0,B,F,r) __attribute__((visibility("default"))) v0 vertex a(uint F[[vertex_id]],uint r[[instance_id]],constant uint&Mi[[buffer(a2(Yd))]],constant VB&j[[buffer(a2(a5))]],constant f0*B[[buffer(0)]],R9 j1,P9 N3){r+=Mi;v0 h0;
#else
#define x1(a,f0,B,F,r) __attribute__((visibility("default"))) v0 vertex a(uint F[[vertex_id]],uint r[[instance_id]],constant VB&j[[buffer(a2(a5))]],constant f0*B[[buffer(0)]],R9 j1,P9 N3){v0 h0;
#endif
#define A8(a,f0,B,D1,j0,F,r) __attribute__((visibility("default"))) v0 vertex a(uint F[[vertex_id]],uint r[[instance_id]],constant VB&j[[buffer(a2(a5))]],constant f0*B[[buffer(0)]],const device D1*j0[[buffer(2)]],R9 j1,P9 N3){v0 h0;
#define c7(a,B3,C3,S3,x2,D1,j0,F) __attribute__((visibility("default"))) v0 vertex a(uint F[[vertex_id]],uint r[[instance_id]],constant VB&j[[buffer(a2(a5))]],constant B3*C3[[buffer(0)]],constant S3*x2[[buffer(1)]],const device D1*j0[[buffer(2)]]){v0 h0;
#define y1(T5) h0.Y0=T5;}return h0;
#define W2(I1,a) I1 __attribute__((visibility("default"))) fragment a(v0 h0[[stage_in]],U5 j1){
#define Q6(I1,a) I1 __attribute__((visibility("default"))) fragment a(v0 h0[[stage_in]],U5 j1,bool R6[[front_facing]]){
#define K2(C) return C;}
#define f7 ,constant VB&j,c d0,U5 j1,Q9 N3,fc S6
#define l3 ,j,d0,j1,N3,S6
#define a4 ,U5 j1
#define n1 ,j1
#define B7
#define e5
#ifdef PLS_IMPL_DEVICE_BUFFER
#define V1 struct i2{
#ifdef PLS_IMPL_DEVICE_BUFFER_RASTER_ORDERED
#define C0(g,a) device uint*a[[buffer(a2(g+v6)),raster_order_group(0)]]
#define q1(g,a) device uint*a[[buffer(a2(g+v6)),raster_order_group(0)]]
#define V2(g,a) device atomic_uint*a[[buffer(a2(g+v6)),raster_order_group(0)]]
#else
#define C0(g,a) device uint*a[[buffer(a2(g+v6))]]
#define q1(g,a) device uint*a[[buffer(a2(g+v6))]]
#define V2(g,a) device atomic_uint*a[[buffer(a2(g+v6))]]
#endif
#define W1 };
#define m4 ,i2 e1,uint M0
#define Z1 ,e1,M0
#define R0(h) unpackUnorm4x8(e1.h[M0])
#define l1(h) e1.h[M0]
#define m3(h) atomic_load_explicit(&e1.h[M0],memory_order::memory_order_relaxed)
#define z0(h,C) e1.h[M0]=packUnorm4x8(C)
#define m1(h,C) e1.h[M0]=(C)
#define n3(h,C) atomic_store_explicit(&e1.h[M0],C,memory_order::memory_order_relaxed)
#define N2(h)
#define h2(h)
#define v5(h,E) atomic_fetch_max_explicit(&e1.h[M0],E,memory_order::memory_order_relaxed)
#define w5(h,E) atomic_fetch_add_explicit(&e1.h[M0],E,memory_order::memory_order_relaxed)
#define O2
#define P2
#define S9(a) __attribute__((visibility("default"))) fragment a(i2 e1,constant VB&j[[buffer(a2(a5))]],v0 h0[[stage_in]],U5 j1,fc S6,Q9 N3){c d0=h0.Y0.xy;S0 G=S0(metal::floor(d0));uint M0=G.y*j.L6+G.x;
#define Y1(a) void S9(a)
#define p2 }
#define G2(a) i S9(a){i N1;
#define D3 }return N1;p2
#else
#define V1 struct i2{
#define C0(g,a) [[color(g)]]i a
#define q1(g,a) [[color(g)]]uint a
#define V2 q1
#define W1 };
#define m4 ,thread i2&V5,thread i2&e1
#define Z1 ,V5,e1
#define R0(h) V5.h
#define l1(h) V5.h
#define m3(h) l1
#define z0(h,C) e1.h=(C)
#define m1(h,C) e1.h=(C)
#define n3(h) m1
#define N2(h) e1.h=V5.h
#define h2(h) e1.h=V5.h
f uint R5(thread uint&x0,uint x){uint i1=x0;x0=metal::max(i1,x);return i1;}
#define v5(h,E) R5(e1.h,E)
f uint S5(thread uint&x0,uint x){uint i1=x0;x0=i1+x;return i1;}
#define w5(h,E) S5(e1.h,E)
#define O2
#define P2
#define S9(a,...) i2 __attribute__((visibility("default"))) fragment a(__VA_ARGS__){c d0[[maybe_unused]]=h0.Y0.xy;i2 e1;
#define Y1(a,...) S9(a,i2 V5,constant VB&j[[buffer(a2(a5))]],v0 h0[[stage_in]],fc S6,U5 j1,Q9 N3)
#define p2 }return e1;
#define Pi(a,...) struct Ni{i Oi[[n(0)]];i2 e1;};Ni __attribute__((visibility("default"))) fragment a(__VA_ARGS__){c d0[[maybe_unused]]=h0.Y0.xy;i N1;i2 e1;
#define G2(a) Pi(a,i2 V5,constant VB&j[[buffer(a2(a5))]],v0 h0[[stage_in]],U5 j1,Q9 N3)
#define D3 }return{.Oi=N1,.e1=e1};
#endif
#define N4 C0
#define discard discard_fragment()
using namespace metal;template<int f2>f vec<uint,f2>floatBitsToUint(vec<float,f2>x){return as_type<vec<uint,f2>>(x);}template<int f2>f vec<int,f2>floatBitsToInt(vec<float,f2>x){return as_type<vec<int,f2>>(x);}f uint floatBitsToUint(float x){return as_type<uint>(x);}f int floatBitsToInt(float x){return as_type<int>(x);}template<int f2>f vec<float,f2>uintBitsToFloat(vec<uint,f2>x){return as_type<vec<float,f2>>(x);}f float uintBitsToFloat(uint x){return as_type<float>(x);}f D unpackHalf2x16(uint x){return as_type<D>(x);}f uint packHalf2x16(D x){return as_type<uint>(x);}f i unpackUnorm4x8(uint x){return unpack_unorm4x8_to_half(x);}f uint packUnorm4x8(i x){return pack_half_to_unorm4x8(x);}f c unpackUnorm2x16(uint x){return unpack_unorm2x16_to_float(x);}f W inverse(W B1){W gc=W(B1[1][1],-B1[0][1],-B1[1][0],B1[0][0]);float Qi=(gc[0][0]*B1[0][0])+(gc[0][1]*B1[1][0]);return gc*(1/Qi);}f v mix(v k,v b,M6 S1){v j8;for(int N0=0;N0<3;++N0) j8[N0]=S1[N0]?b[N0]:k[N0];return j8;}f c mix(c k,c b,Y4 S1){c j8;for(int N0=0;N0<2;++N0) j8[N0]=S1[N0]?b[N0]:k[N0];return j8;}f c mix(c k,c b,float t){return mix(k,b,c(t));}f float mod(float x,float y){return fmod(x,y);}