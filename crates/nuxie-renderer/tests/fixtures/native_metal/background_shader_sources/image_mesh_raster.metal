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
#define wf float(2048)
#define Ac 11
#define la float(512)
#define gc float(0.001953125)
#define ma float(3)
#define jc 0
#define kc 1
#define Bc 3u
#define xf (Bc+1u)
#define yf float(1.0)
#define Cc 7
#define Dc 0x7fu
#define fc 0x80000000u
#define hc 0x40000000u
#define X9 0x20000000u
#define ef (fc|hc|X9)
#define Ec (1u<<31u)
#define zf (1u<<29u)
#define Y3 (7u<<26u)
#define Af (5u<<26u)
#define Bf (4u<<26u)
#define r8 (2u<<26u)
#define v8 (1u<<26u)
#define w8 (1u<<25u)
#define Cf (1u<<24u)
#define E3 (1u<<23u)
#define na (1u<<22u)
#define Fc (1u<<21u)
#define x8 (1u<<20u)
#define Gc (1u<<19u)
#define Hc 0xffffu
#define Df .0
#define y8 0
#define Ic 1
#define Jc 2
#define y8 0
#define Ic 1
#define Jc 2
#define W7 0u
#define Lb 1u
#define M9 2u
#define Ef 3u
#define Ne 0x100u
#define K9 0x200u
#define Oe 0x400u
#define a3 0
#define Z4 1
#define F4 0
#define Kc 1
#define Lc 2
#define Gb 3
#define Hb 4
#define Mc 5
#define oa 6
#define Ff 7
#define Nc 8
#define d7 9
#define Oc 10
#define U3 11
#define Gf 12
#define c6 13
#define Hf 13
#define N1(f) (3+f)
#define F3 2
#define If 3
#define Q2 0
#define R2 1
#define d6 2
#define G6 3
#define Jf 2
#define r9 2
#define v9 3
#define w9 4
#define B9 5
#define Kf 5
#define x9 5
#define y9 6
#define z9 7
#define A9 8
#define mf 1023u
#define o9 6.2e-5
#define N5 0u
#define te 1u
#define ue 2u
#define ve 3u
#define we 4u
#define xe 5u
#define ze 6u
#define Ae 7u
#define Be 8u
#define Ce 9u
#define De 10u
#define se 11u
#define Ee 12u
#define Fe 13u
#define Ge 14u
#define He 15u
#define I9 float(2048)
#define Ib float(0.00048828125)
#define J9 float(1<<16)
#define O9 (1u<<16)
#define Q5 17u
#define c8 0x1ffffu
#define Lf float(1024)
#define pa float(0.0009765625)
#define qa 19u
#define j5 (1u<<(qa-1u))
#define ra ((1u<<qa)-1u)
#define e7 (1u<<qa)
#define Mf 0
#define Nf 1
#define Of 2
#define Pf 3
#define Qf 4
#define Rf 5
#define Sf 6
#define Tf 7
#define Uf 8
#define Vf 9
#define Wf 10
#define Xf 11
#define Yf 12
#define Zf 13
#define Pc 65536.
#define sa 8.
#define ta 32u
#define e6 5u
#ifdef ag
#if ag>=201703
ri(ta==1u<<e6);
#endif
#endif

#ifndef n3
#define n3(e4) float e4;
#endif
#ifndef f4
#define f4(e4) uint e4;
#endif
#ifndef md
#define md(e4) a6 e4;
#endif
#ifndef Ja
#define Ja(e4) d e4;
#endif
#ifndef Eg
#define Eg(e4) g e4;
#endif
#ifndef nd
#define nd CC
#endif
r7(F4,nd)n3(ec)n3(od)n3(ff)n3(gf)f4(m6)f4(Fg)f4(Re)f4(Se)md(R7)Ja(Bg)Ja(pd)f4(a2)n3(Gg)f4(Z5)n3(P2)n3(qd)f4(Me)n3(z3)n3(A3)n3(rd)f4(yg)J8(n)

#define B3 3.14159265359
#define m8 6.28318530718
#define T6 1.57079632679
#ifndef CB
#define n4 float(.5)
#else
#define n4 float(.0)
#endif
#define K3(l) l8(l,n.ff,n.gf)
#ifdef GF
#define ic(T,f,a) e5(T,f,a)
#define B4 g
#define Y9(q) q
#define V5(q) q
#define Z9(q) uintBitsToFloat(q)
#define f5(q) floatBitsToUint(q)
#else
#define ic(T,f,a) C4(T,f,a)
#define B4 G
#define Y9(q) floatBitsToUint(q)
#define V5(q) uintBitsToFloat(q)
#define Z9(q) q
#define f5(q) q
#endif
#define hf(a,l,n8) q1(a,X(l)+X(-1,0))n8,q1(a,X(l)+X(0,0))n8,q1(a,X(l)+X(0,-1))n8,q1(a,X(l)+X(-1,-1))n8
#define g5(q) U6(XC,aa,q,jc,float(jc),.0).x
#define lc(q) U6(XC,aa,q,kc,float(kc),.0).x
#ifdef mc
e c S4(float x){return x;}e c W5(uint x){return float(x);}e c jf(N x){return float(x);}e c ba(int x){return float(x);}e i Y4(g xyzw){return xyzw;}e E O7(d xy){return xy;}e i dc(G xyzw){return vec4(xyzw);}e N X5(c x){return uint(x);}e N W1(uint x){return x;}
#else
e c S4(float x){return(c)x;}e c W5(uint x){return(c)x;}e c jf(N x){return(c)x;}e c ba(int x){return(c)x;}e i Y4(g xyzw){return(i)xyzw;}e E O7(d xy){return(E)xy;}e i dc(G xyzw){return(i)xyzw;}e N X5(c x){return(N)x;}e N W1(uint x){return(N)x;}
#endif
e c G0(c x){return x;}e E A2(E xy){return xy;}e E A2(c x,c y){E S;S.x=x,S.y=y;return S;}e E A2(c x){E S;S.x=x,S.y=x;return S;}e d J6(float x){return d(x,x);}e v Q0(c x,c y,c z){v S;S.x=x,S.y=y,S.z=z;return S;}e v Q0(c x){v S;S.x=x,S.y=x,S.z=x;return S;}e i C0(c x,c y,c z,c w){i S;S.x=x,S.y=y,S.z=z,S.w=w;return S;}e i C0(v xyz,c w){i S;S.xyz=xyz;S.w=w;return S;}e i C0(c x){i S;S.x=x,S.y=x,S.z=x,S.w=x;return S;}e i C0(i x){return x;}e D4 kf(bool b){return D4(b,b);}e V6 Ph(v m,v b,v G1){V6 S;S[0]=m;S[1]=b;S[2]=G1;return S;}e W6 Qh(v m,v b){W6 S;S[0]=m;S[1]=b;return S;}e h5 Rh(i m,i b,i G1,i lf){h5 S;S[0]=m;S[1]=b;S[2]=G1;S[3]=lf;return S;}e g0 l2(g x){return g0(x.xy,x.zw);}e uint Qb(N x){return x;}e d Y5(d m,d b,float t){return(b-m)*t+m;}e c o8(uint nc,uint Z5){return nc==0u?.0:unpackHalf2x16((nc+mf)*Z5).x;}e float oc(d h2){h2=normalize(h2);float e1=acos(clamp(h2.x,-1.,1.));return h2.y>=.0?e1:-e1;}e i Sh(i j){return C0(j.xyz*j.w,j.w);}e v C6(i ca){return ca.xyz*(ca.w!=.0?1./ca.w:.0);}e c f3(E X6){return min(X6.x,X6.y);}e c f3(v pc){return min(f3(pc.xy),pc.z);}e c f3(i qc){E X6=min(qc.xy,qc.zw);c nf=min(X6.x,X6.y);return nf;}e c J5(E Y6){return max(Y6.x,Y6.y);}e c J5(v rc){return max(J5(rc.xy),rc.z);}e c J5(i sc){E Y6=max(sc.xy,sc.zw);c of=max(Y6.x,Y6.y);return of;}e float E9(d x){return abs(x.x)+abs(x.y);}e c da(c x,c ea,c fa){
#if defined(HF)||defined(DD)
#ifdef DD
if(DD)
#endif
{if(x<fa)if(x>ea)return x;else return ea;else return fa;}
#endif
return clamp(x,ea,fa);}e c tc(d K0,c B2,c m3){c pf=fract(0.06711056*K0.x+0.00583715*K0.y);c qf=fract(52.9829189*pf);return(qf*B2)+m3;}
#if 0
e c Th(d K0,float B2,float m3){int x=int(K0.x);int y=int(K0.y);int uc=(x^y);int b=(y>>1)&1;b|=(uc&2);b|=(y&1)<<2;b|=(uc&1)<<3;float rf=float(b);c sf=S4(rf)/16.0;return(sf*B2)+m3;}e c Uh(d K0,float B2,float m3){K0.y*=0.5;K0.x=fract(K0.x*0.5+K0.y);K0.y=fract(K0.y);float N3=(K0.y*0.5+K0.x);return(N3*B2)+m3;}
#endif
#ifdef LB
e c ga(d K0,c B2,c m3){return LB?tc(K0,B2,m3):.0;}e v E2(v j,c Z6,d K0,c B2,c m3){return(LB&&Z6!=.0)?(tc(K0,B2,m3)+j):j;}e v E2(v j,c Z6,c vc){return(LB&&Z6!=.0)?(vc+j):j;}
#else
e c ga(d K0,float B2,float m3){return 0.;}e v E2(v j,c Z6,d K0,c B2,c m3){return j;}e v E2(v j,c Z6,c vc){return j;}
#endif
#ifdef DB
e g l8(d wc,float tf,float xc){return g(wc.x*tf-1.,wc.y*xc-sign(xc),0.,1.);}
#ifndef CB
e g Q7(g0 X3,d E4,d ha){d ia=abs(X3[0])+abs(X3[1]);if(ia.x!=.0&&ia.y!=.0){d K=1./ia;d i5=U0(X3,ha)+E4;const float uf=.5;return g(i5,-i5)*K.xyxy+K.xyxy+uf;}else{return E4.xyxy;}}
#else
e float ja(uint ka){return 1.-float(ka)*(2./32768.);}
#ifdef BB
e void yc(g0 X3,d E4,d ha a7){
#ifndef LE
if(any(notEqual(g(X3),g(.0,.0,.0,.0)))){d i5=U0(X3,ha)+E4.xy;gl_ClipDistance[0]=i5.x+1.;gl_ClipDistance[1]=i5.y+1.;gl_ClipDistance[2]=1.-i5.x;gl_ClipDistance[3]=1.-i5.y;}else{gl_ClipDistance[0]=gl_ClipDistance[1]=gl_ClipDistance[2]=gl_ClipDistance[3]=E4.x-.5;}
#endif
}
#endif
#endif
#endif
#ifdef GB
#ifdef BC
e c k3(c j){return(j<=0.04045)?j/12.92:pow(abs((j+0.055)/1.055),2.4);}e v k3(v j){return Q0(k3(j.x),k3(j.y),k3(j.z));}e i k3(i j){return C0(k3(j.xyz),j.w);}
#endif
#endif
#if defined(GB)&&defined(CB)&&!defined(Q)
e i zc(h5 c7,int p8){if(p8==0xf){return(c7[0]+c7[1]+c7[2]+c7[3])*.25;}else{i vf=g(notEqual(p8&a6(1,2,4,8),a6(0,0,0,0)));i S=U0(c7,vf);int q8=(p8&5)+((p8>>1)&5);q8=(q8&3)+(q8>>2);S*=1./float(q8);return S;}}
#endif

#ifdef DB
g1(g3)L(0,d,OC);h1 g1(x3)L(1,d,PC);h1 g1(n1)L(r9,g,WB);L(v9,g,QB);L(w9,g,NB);
#ifdef M3
L(x9,uint,XB);L(y9,uint,YB);L(z9,uint,ZB);L(A9,uint,AC);
#else
L(B9,G,IB);
#endif
h1
#endif
k2 J0 W(0,d,E5);
#ifdef I
MB W(1,c,I3);
#endif
#if defined(BB)&&!defined(CB)
J0 W(2,g,L0);
#endif
MB W(3,c,H1);
#ifdef AB
O2 W(4,N,A1);
#endif
f2
#ifdef DB
S3 T3 F6(GC,g3,h3,x3,y3,n1,f0,B){M(B,h3,OC,d);M(B,y3,PC,d);M(r,f0,WB,g);M(r,f0,QB,g);M(r,f0,NB,g);
#ifdef M3
M(r,f0,XB,uint);M(r,f0,YB,uint);M(r,f0,ZB,uint);M(r,f0,AC,uint);G IB=G(XB,YB,ZB,AC);
#else
M(r,f0,IB,G);
#endif
V(E5,d);
#ifdef I
V(I3,c);
#endif
#if defined(BB)&&!defined(CB)
V(L0,g);
#endif
V(H1,c);
#ifdef AB
V(A1,N);
#endif
d l0=U0(l2(WB),OC)+NB.xy;E5=PC;
#ifdef I
if(I){I3=o8(IB.y,n.Z5);}
#endif
#ifdef BB
if(BB){
#ifndef CB
L0=Q7(l2(QB),NB.zw,l0 v5);
#else
yc(l2(QB),NB.zw,l0 v5);
#endif
}
#endif
g U=K3(l0);
#ifdef RC
U.y=-U.y;
#endif
#ifdef CB
U.z=ja(IB.w);
#endif
H1=uintBitsToFloat(IB.x);
#ifdef AB
A1=W1(IB.z);
#endif
a0(E5);
#ifdef I
a0(I3);
#endif
#if defined(BB)&&!defined(CB)
a0(L0);
#endif
a0(H1);
#ifdef AB
a0(A1);
#endif
z1(U);}
#endif

#ifdef GB
#if(defined(Q)&&!defined(I))||defined(SB)
#undef vb
#else
#define vb
#endif
I1
#ifndef Q
w0(Q2,j0);
#endif
#ifndef SB
j1(R2,h0);
#ifndef Q
w0(d6,i4);
#endif
j1(G6,P0);
#else
w0(R2,h0);
#endif
J1
#ifdef OB
C3 X2(Z4,U3,IC);D3 a5 V3(S5)c5 O3 P3
#endif
#ifdef Q
#ifdef OB
o2(JB)
#else
o2(JB)
#endif
#else
#ifdef OB
L1(JB)
#else
L1(JB)
#endif
#endif
{
#ifdef FB
A(f1,g);A(C2,d);
#endif
#ifdef I
A(I3,c);
#endif
#ifdef BB
A(L0,g);
#endif
#if defined(FB)&&defined(AB)
A(e2,c);
#endif
#ifdef OB
A(E5,d);A(H1,c);
#ifdef AB
A(A1,N);
#endif
#endif
#ifdef FB
i j=J7(f1,1. S2);c o=clamp(n2(BD,Q9,C2,.0).x,G0(.0),G0(1.));
#endif
#ifdef OB
i j=x7(IC,S5,E5,n.qd);c o=1.;
#endif
#ifdef BB
if(BB){c U4=max(f3(Y4(L0)),G0(.0));o=min(U4,o);}
#endif
#ifdef vb
w2;
#endif
#if defined(I)
if(I&&I3!=.0){c r3;
#ifndef SB
E M0=unpackHalf2x16(Y0(h0));c B6=M0.y;r3=max(B6==I3?M0.x:G0(.0),G0(.0));
#else
r3=H0(h0).x;
#endif
r3=max(r3,G0(.0));o=min(o,r3);}
#endif
#ifdef OB
o*=H1;
#endif
#if!defined(Q)
i K1=H0(j0);
#ifdef AB
if(AB){
#ifdef FB
N R3=X5(e2);
#endif
#ifdef OB
j.xyz=C6(j);N R3=A1;
#endif
if(R3!=N5){j.xyz=Q4(j.xyz,K1,R3);}j.w*=o;j.xyz*=j.w;}else
#endif
{j*=o;}
#ifdef BC
if(BC){j=k3(j);}
#endif
j.xyz=E2(j.xyz,j.w,Y.xy,n.z3,n.A3);
#ifndef SB
j=K1*(1.-j.w)+j;
#endif
x0(j0,j);
#endif
#ifndef SB
d2(h0);d2(P0);
#else
x0(h0,C0(.0));
#endif
#ifdef vb
x2;
#endif
#ifdef Q
j=(j*o);j.xyz=E2(j.xyz,j.w,Y.xy,n.z3,n.A3);C1=j;l3
#else
Y1;
#endif
}
#endif

