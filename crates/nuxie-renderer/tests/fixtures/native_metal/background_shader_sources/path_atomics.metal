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

#define f7 -2.
#define Qc -1.5
#define Rc .25
#define z8 1e3
#define Sc (z8*z8)
#ifdef DB
S3 ic(a3,Ff,LC);
#ifdef HB
f6(a3,d7,XC);
#endif
T3 z4 G4(Lc,bg,PB);K5(Gb,Je,AD);L5(Hb,Ke,RB);G4(Mc,cg,ED);A4
#endif
#if defined(HB)||defined(FB)
Z3(d7,aa)
#endif
#ifdef GB
C3 X2(a3,Nc,KD);
#if defined(HB)||defined(FB)
f6(a3,d7,XC);
#endif
#ifdef FB
k5(a3,Oc,BD);
#endif
X2(Z4,U3,IC);
#if defined(CB)&&defined(AB)&&!defined(Q)
g7(SD);
#endif
D3 Z3(Nc,Mb)
#ifdef FB
Z3(Oc,Q9)
#endif
a5 V3(S5)c5
#endif
#ifdef GB
e bool R5(g P){return P.y>=.0;}e bool R5(E P){return P.y>=.0;}
#endif
#if defined(GB)&&defined(HB)
e bool Nb(g P){return P.x<Qc;}e bool Ob(g P){return P.y<Qc;}
#endif
#ifdef DB
g Tc(float ua,d A8,float D1){d g6=(1.-A8*abs(D1))*.5;float a4,l5;if(abs(ua-T6)<1./z8){a4=.0;l5=.0;}else{float va=tan(ua);a4=sign(T6-ua)/max(abs(va),1./Sc);l5=a4>=.0?g6.y-(1.-g6.x)*va:g6.y+g6.x*va;}g P;P.x=max(g6.x,.0)+Rc;P.y=-g6.y+f7;P.z=a4;P.w=l5;return P;}
#endif
#ifdef HB
e c Z7(g P G3){c a4=P.z;c l5=max(P.w,.0);c h6=a4>=.0?g5(l5):.0;if(abs(a4)<z8){c x=abs(P.x)-Rc;c y=-P.y+f7;c V2=(y-l5)*0.5984134206;i t=l5+V2*C0(0.20888568955,0.62665706865,1.04442844776,1.46219982687);i u=t*-a4+(y*a4+x);i dg=C0(g5(u[0]),g5(u[1]),g5(u[2]),g5(u[3]));i Uc=t*5.09593080173+-2.54796540086;i eg=exp2(-Uc*Uc);h6+=dot(dg,eg)*V2;}return h6*sign(P.x);}e c v4(g P G3){float h6=1.;float fg=(1.-f7)+P.x;h6-=g5(fg);float gg=1.-P.y;h6-=g5(gg);return h6;}
#endif
#if defined(DB)&&defined(ID)
e X m5(int Vc){return X(Vc&((1<<Ac)-1),Vc>>Ac);}e float Wc(g0 T0,d hg){d h2=U0(T0,hg);return(abs(h2.x)+abs(h2.y))*(1./dot(h2,h2));}e bool q9(g h7,g wa,int r,Z0(uint)c3,Z0(d)ig
#ifndef CB
,Z0(g)O1
#else
,Z0(N)i7
#endif
i6){int B8=int(h7.x);float D1=h7.y;float xa=h7.z;int Xc=floatBitsToInt(h7.w)>>2;int j7=floatBitsToInt(h7.w)&3;int ya=min(B8,Xc-1);int H4=r*Xc+ya;B4 n5=q1(LC,m5(H4));uint i0=f5(n5.w);uint C8=max(i0&Hc,1u);G za=N0(ED,C8-1u);d Yc=uintBitsToFloat(za.xy);c3=za.z&0xffffu;uint Zc=za.w;g0 T0=l2(uintBitsToFloat(N0(PB,c3*4u)));G I4=N0(PB,c3*4u+1u);d i3=uintBitsToFloat(I4.xy);float H2=uintBitsToFloat(I4.z);float I2=uintBitsToFloat(I4.w);uint ad=i0&E3;if(ad!=0u){B8=int(wa.x);D1=wa.y;xa=wa.z;}if(B8!=ya){int bd=H4+B8-ya;B4 cd=q1(LC,m5(bd));if((f5(cd.w)&(E3|0xffffu))!=(i0&(E3|0xffffu))){bool jg=H2==.0||Yc.x!=.0;if(jg){H4=int(Zc);n5=q1(LC,m5(H4));}}else{H4=bd;n5=cd;}i0=(f5(n5.w)&~E3)|ad;}float e1;
#ifdef HB
float k7;float v1;if((i0&Y3)==v8&&j7==y8){uint dd=f5(n5.z);float c4=float(dd&0xffffu);float i2=float(dd>>16);X D8=X(-c4-1.,i2-c4+1.);if((i0&E3)!=0u)D8=-D8;B4 ed=q1(LC,m5(H4+D8.x));B4 Aa=q1(LC,m5(H4+D8.y));if((f5(Aa.w)&(E3|0xffffu))!=(f5(ed.w)&(E3|0xffffu))){Aa=q1(LC,m5(int(Zc)));}k7=V5(ed.z);float fd=V5(Aa.z);v1=fd-k7;if(abs(v1)>B3)v1-=m8*sign(v1);float Ba=i2+1.-float(Bc);float gd=clamp(round(abs(v1)/B3*Ba),1.,Ba-1.);float l7=Ba-gd;if(c4<=l7){v1=-(B3*sign(v1)-v1);i2=l7;if(c4==l7)D1=-D1;}else if(c4==l7+1.){c4=.0;i2=.0;D1=.0;}else{c4-=l7+2.;i2=gd;}if(c4==i2){e1=fd;}else{e1=k7+v1*(c4/i2);}}else
#endif
{e1=V5(n5.z);}d W2=d(sin(e1),-cos(e1));d hd=V5(n5.xy);d E8=d(0,0);if(I2!=.0){I2=max(I2,(ma/3.)/length(U0(T0,W2)));}if(H2!=.0){D1*=sign(determinant(T0));if((i0&x8)!=0u)D1=min(D1,.0);if((i0&Gc)!=0u)D1=max(D1,.0);float J4=I2!=.0?I2:Wc(T0,W2)*n4;c id=1.;if(J4>H2&&I2==.0){id=S4(H2)/S4(J4);H2=J4;}d o5=W2*(H2+J4);
#ifndef CB
float x=D1*(H2+J4);O1.xy=(1./(J4*2.))*(d(x,-x)+H2)+.5;O1.zw=J6(.0);
#endif
uint Ca=i0&Y3;if(Ca>r8){int m7=2;if((i0&na)==0u)m7=-m7;if((i0&E3)!=0u)m7=-m7;X kg=m5(H4+m7);B4 lg=q1(LC,kg);float mg=V5(lg.z);float n7=abs(mg-e1);if(n7>B3)n7=m8-n7;bool F8=(i0&na)!=0u;bool ng=(i0&x8)!=0u;float jd=n7*(F8==ng?-.5:.5)+e1;d G8=d(sin(jd),-cos(jd));float Da=Wc(T0,G8);float o7=cos(n7*.5);float Ea;if((Ca==Af)||(Ca==Bf&&o7>=.25)){float og=(i0&w8)!=0u?1.:.25;Ea=H2*(1./max(o7,og));}else{Ea=H2*o7+Da*.5;}float Fa=Ea+Da*n4;if((i0&Fc)!=0u){float kd=H2+J4;float pg=J4*.125;if(kd<=Fa*o7+pg){float qg=kd*(1./o7);o5=G8*qg;}else{d Ga=G8*Fa;d rg=d(dot(o5,o5),dot(Ga,Ga));o5=U0(rg,inverse(g0(o5,Ga)));}}d sg=abs(D1)*o5;float ld=(Fa-dot(sg,G8))/(Da*(n4*2.));
#ifndef CB
if((i0&x8)!=0u)O1.y=ld;else O1.x=ld;
#endif
}
#ifndef CB
O1.xy*=id;O1.y=max(O1.y,1e-4);if(I2!=.0){O1.x=f7-O1.x;}
#endif
E8=U0(T0,D1*o5);if(j7!=y8)return false;}else{
#ifndef CB
O1=g(xa,-1.,.0,.0);
#ifdef HB
if(I2!=.0){O1.y=f7;O1.z=Sc;O1.w=xa;if((i0&Y3)==v8&&j7==y8){if(v1<.0){k7+=v1;v1=-v1;}float d4=e1-k7;d4=mod(d4+T6,m8)-T6;d4=clamp(d4,.0,v1);if(d4>v1*.5){d4=v1-d4;}d A8=d(sin(d4),cos(d4));
#if 0
float P1=1.+.33*log2(T6/(B3-min(v1,B3-B3/16.)));g tg=Tc(v1,A8,.5*(P1/3.));float ug=Z7(tg d1);float vg=lc(ug);float wg=(.5-vg)*(ma*2.);float xg=P1/max(wg,P1);D1*=xg;
#endif
O1=Tc(v1,A8,D1);}E8=U0(T0,(D1*I2)*W2);}else
#endif
{E8=sign(U0(D1*W2,inverse(T0)))*n4;}if(bool(i0&E3)!=bool(i0&Cf)){O1*=g(-1.,+1.,+1.,+1.);}
#endif
if(j7==Jc)hd=Yc;if((i0&Ec)!=0u&&j7!=Ic){return false;}}ig=U0(T0,hd)+E8+i3;
#ifdef CB
G K4=N0(PB,c3*4u+2u);i7=W1(K4.x);
#else
O1.xy=mix(O1.xy,d(1.,-1.),kf(n.yg!=0u));
#endif
return true;}
#endif
#if defined(DB)&&defined(EB)
e d Eb(c0 j6,Z0(uint)c3
#ifdef CB
,Z0(N)i7
#else
,Z0(c)zg
#endif
i6){c3=floatBitsToUint(j6.z)&0xffffu;
#ifdef CB
G K4=N0(PB,c3*4u+2u);i7=W1(K4.x);
#else
zg=ba(floatBitsToInt(j6.z)>>16);
#endif
d k6=j6.xy;g0 T0=l2(uintBitsToFloat(N0(PB,c3*4u)));G I4=N0(PB,c3*4u+1u);d i3=uintBitsToFloat(I4.xy);k6=U0(T0,k6)+i3;return k6;}
#endif
#if defined(DB)&&defined(FB)
e d Db(c0 j6,Z0(uint)c3,
#ifdef CB
Z0(N)i7,
#endif
Z0(d)Ag i6){c3=floatBitsToUint(j6.z)&0xffffu;G K4=N0(PB,c3*4u+2u);
#ifdef CB
i7=W1(K4.x);
#endif
d k6=j6.xy;c0 p7=uintBitsToFloat(K4.yzw);Ag=(k6*p7.x+p7.yz)*n.Bg;return k6;}
#endif
e c H8(c Z1,c E1,c r2){return(E1-Z1)/max(1.-Z1*r2,o9);}e uint I8(a1 l6,uint Cg){uint Ha=(l6.y>>e6)*(Cg<<e6)+((l6.x>>e6)<<(e6<<1));Ha+=((l6.x&0x1cu)<<e6)+((l6.y&0x1cu)<<2);Ha+=((l6.y&0x3u)<<2)+(l6.x&0x3u);return Ha;}
#ifdef SB
#ifdef Q
#define d5 o2
#define W3(p5) C1=p5;l3
#else
#define d5 L1
#define W3(p5) x0(j0,p5);Y1;
#endif
e c Ia(uint Dg){return ba(int((Dg&ra)-j5))*pa;}e uint q7(c o){return uint(o*Lf+.5);}
#endif

#ifdef ID
#ifdef DB
g1(e0)L(0,g,UB);L(1,g,VB);h1
#endif
k2
#ifdef HB
J0 W(0,g,O);
#else
J0 W(0,E,O);
#endif
O2 W(1,N,B0);f2
#ifdef DB
y1(GC,e0,F,B,r){M(B,F,UB,g);M(B,F,VB,g);
#ifdef HB
V(O,g);
#else
V(O,E);
#endif
V(B0,N);g U;uint o0;d l0;g P;if(q9(UB,VB,r,o0,l0,P v3)){
#ifdef HB
O=P;
#else
O.xy=O7(P.xy);
#endif
B0=W1(o0);U=K3(l0);}else{U=g(n.P2,n.P2,n.P2,n.P2);}a0(O);a0(B0);z1(U);}
#endif
#endif
#if defined(EB)||defined(FB)
#ifdef DB
g1(e0)L(0,L3,KB);h1
#endif
k2
#ifdef FB
J0 W(0,d,C2);
#else
MB W(0,c,i1);
#endif
O2 W(1,N,B0);f2
#ifdef DB
y1(GC,e0,F,B,r){M(B,F,KB,c0);
#ifdef FB
V(C2,d);
#else
V(i1,c);
#endif
V(B0,N);uint o0;d l0;
#ifdef FB
l0=Db(KB,o0,C2 v3);
#else
l0=Eb(KB,o0,i1 v3);
#endif
B0=W1(o0);g U=K3(l0);
#ifdef FB
a0(C2);
#else
a0(i1);
#endif
a0(B0);z1(U);}
#endif
#endif
#ifdef JD
#ifdef DB
g1(e0)L(0,g,HC);h1 g1(n1)L(r9,g,WB);L(v9,g,QB);L(w9,g,NB);
#ifdef M3
L(x9,uint,XB);L(y9,uint,YB);L(z9,uint,ZB);L(A9,uint,AC);
#else
L(B9,G,IB);
#endif
h1
#endif
k2 J0 W(0,d,X1);J0 W(1,c,R4);
#ifdef BB
J0 W(2,g,L0);
#endif
MB W(3,c,H1);
#ifdef I
O2 W(4,N,w3);
#endif
#ifdef AB
O2 W(5,N,A1);
#endif
f2
#ifdef DB
P7(GC,e0,F,n1,f0,B,r){M(B,F,HC,g);M(r,f0,WB,g);M(r,f0,QB,g);M(r,f0,NB,g);
#ifdef M3
M(r,f0,XB,uint);M(r,f0,YB,uint);M(r,f0,ZB,uint);M(r,f0,AC,uint);G IB=G(XB,YB,ZB,AC);
#else
M(r,f0,IB,G);
#endif
V(X1,d);V(R4,c);
#ifdef BB
V(L0,g);
#endif
V(H1,c);
#ifdef I
V(w3,N);
#endif
#ifdef AB
V(A1,N);
#endif
bool C9=HC.z==.0||HC.w==.0;R4=C9?.0:1.;d l0=HC.xy;g0 T0=l2(WB);g0 E6=transpose(inverse(T0));if(!C9){float D9=n4*E9(E6[1])/dot(T0[1],E6[1]);if(D9>=.5){l0.x=.5;R4*=S4(.5/D9);}else{l0.x+=D9*HC.z;}float F9=n4*E9(E6[0])/dot(T0[0],E6[0]);if(F9>=.5){l0.y=.5;R4*=S4(.5/F9);}else{l0.y+=F9*HC.w;}}X1=l0;l0=U0(T0,l0)+NB.xy;if(C9){d N3=U0(E6,HC.zw);N3*=E9(N3)/dot(N3,N3);l0+=n4*N3;}
#ifdef BB
if(BB){L0=Q7(l2(QB),NB.zw,l0);}
#endif
H1=uintBitsToFloat(IB.x);
#ifdef I
w3=W1(IB.y);
#endif
#ifdef AB
A1=W1(IB.z);
#endif
g U=K3(l0);a0(X1);a0(R4);
#ifdef BB
a0(L0);
#endif
a0(H1);
#ifdef I
a0(w3);
#endif
#ifdef AB
a0(A1);
#endif
z1(U);}
#endif
#elif defined(OB)
#ifdef DB
g1(g3)L(0,d,OC);h1 g1(x3)L(1,d,PC);h1 g1(n1)L(r9,g,WB);L(v9,g,QB);L(w9,g,NB);
#ifdef M3
L(x9,uint,XB);L(y9,uint,YB);L(z9,uint,ZB);L(A9,uint,AC);
#else
L(B9,G,IB);
#endif
h1
#endif
k2 J0 W(0,d,X1);
#ifdef BB
J0 W(1,g,L0);
#endif
MB W(3,c,H1);
#ifdef I
O2 W(4,N,w3);
#endif
#ifdef AB
O2 W(5,N,A1);
#endif
f2
#ifdef DB
F6(GC,g3,h3,x3,y3,n1,f0,B){M(B,h3,OC,d);M(B,y3,PC,d);M(r,f0,WB,g);M(r,f0,QB,g);M(r,f0,NB,g);
#ifdef M3
M(r,f0,XB,uint);M(r,f0,YB,uint);M(r,f0,ZB,uint);M(r,f0,AC,uint);G IB=G(XB,YB,ZB,AC);
#else
M(r,f0,IB,G);
#endif
V(X1,d);
#ifdef BB
V(L0,g);
#endif
V(H1,c);
#ifdef I
V(w3,N);
#endif
#ifdef AB
V(A1,N);
#endif
g0 T0=l2(WB);d l0=U0(T0,OC)+NB.xy;X1=PC;
#ifdef BB
if(BB){L0=Q7(l2(QB),NB.zw,l0);}
#endif
H1=uintBitsToFloat(IB.x);
#ifdef I
w3=W1(IB.y);
#endif
#ifdef AB
A1=W1(IB.z);
#endif
g U=K3(l0);a0(X1);
#ifdef BB
a0(L0);
#endif
a0(H1);
#ifdef I
a0(w3);
#endif
#ifdef AB
a0(A1);
#endif
z1(U);}
#endif
#endif
#ifdef AF
#ifdef DB
g1(e0)h1
#endif
k2 f2
#ifdef DB
y1(GC,e0,F,B,r){X m2;m2.x=(B&1)==0?n.R7.x:n.R7.z;m2.y=(B&2)==0?n.R7.y:n.R7.w;g U=K3(d(m2));z1(U);}
#endif
#endif
#ifdef HE
#endif
#if defined(IE)&&!defined(Q)
#endif
#ifdef GB
I1
#ifndef Q
#ifdef JE
#define G9 JE
#else
#define G9 Q2
#endif
#ifdef ZC
o4(G9,j0);
#else
w0(G9,j0);
#endif
#endif
#ifdef VC
#define p4 i
#define H9 H0
#define S7 C0(.0)
#define Fb(q) ((q).w!=.0)
#ifdef I
#ifndef QC
w0(R2,h0);
#else
o4(R2,h0);
#endif
#endif
#else
#define p4 uint
#define S7 0u
#define H9 Y0
#define Fb(q) ((q)!=0u)
#ifdef I
j1(R2,h0);
#endif
#endif
D2(G6,q4);J1 O3 K5(Gb,Je,AD);L5(Hb,Ke,RB);P3 e uint Le(float x){return uint(round(x*I9+J9));}e c T7(uint x){return S4(float(x)*Ib+(-J9*Ib));}N U7(N o0){
#ifdef BF
o0=min(o0,n.Me);
#endif
return o0;}
#ifdef I
e void Jb(uint k1,p4 M0,T4(c)o){
#ifdef VC
if(all(lessThan(abs(M0.xy-unpackUnorm4x8(k1).xy),A2(.25/255.))))o=min(o,M0.z);else o=.0;
#else
if(k1==M0>>16)o=min(o,unpackHalf2x16(M0).x);else o=.0;
#endif
}
#endif
e void V7(uint o0,c p0,Z0(i)R
#if defined(I)&&!defined(QC)
,T4(p4)o1
#endif
H6 Q3){a1 p1=M5(AD,o0);c o=p0;if((p1.x&(Ne|K9))!=0u){o=abs(o);
#ifdef WC
if(WC&&(p1.x&K9)!=0u){o=1.-abs(fract(o*.5)*2.+-1.);}
#endif
}o=clamp(o,G0(.0),G0(1.));
#ifdef I
if(I){uint k1=p1.x>>16u;if(k1!=0u){Jb(k1,H9(h0),o);}}
#endif
#ifdef BB
if(BB&&(p1.x&Oe)!=0u){g0 T0=l2(N0(RB,o0*4u+2u));g i3=N0(RB,o0*4u+3u);d Pe=U0(T0,Y)+i3.xy;E Kb=O7(abs(Pe)*i3.zw-i3.zw);c U4=clamp(min(Kb.x,Kb.y)+.5,.0,1.);o=min(o,U4);}
#endif
uint j3=p1.x&0xfu;if(j3<=Lb){R=unpackUnorm4x8(p1.y);
#ifdef I
if(I&&j3==W7){
#ifndef QC
#ifdef VC
o1.xy=R.zw;o1.z=o;o1.w=1.;
#else
o1=p1.y|packHalf2x16(A2(o,.0));
#endif
#endif
R=C0(.0);}
#endif
}else{g0 T0=l2(N0(RB,o0*4u));g i3=N0(RB,o0*4u+1u);d V4=U0(T0,Y)+i3.xy;float t=j3==M9?V4.x:length(V4);t=clamp(t,.0,1.);float x=t*i3.z+i3.w;float y=uintBitsToFloat(p1.y);R=n2(KD,Mb,d(x,y),.0);}R.w*=o;
#if!defined(Q)&&defined(AB)
N R3;if(AB&&R.w!=.0&&(R3=W1((p1.x>>4)&0xfu))!=N5){i K1=H0(j0);R.xyz=Q4(R.xyz,K1,R3);}
#endif
#if defined(BC)&&(defined(Q)||defined(QC))
R=k3(R);
#endif
R.xyz*=R.w;}
#if!defined(Q)&&!defined(ZC)
e void X7(i R Q3){
#ifndef VC
if(R.w==.0)return;float I6=1.-R.w;if(I6!=.0)R+=H0(j0)*I6;
#endif
x0(j0,R);}
#endif
#if defined(I)&&!defined(QC)
e void N9(p4 o1 Q3){
#ifdef VC
x0(h0,o1);
#else
if(o1!=0u)c1(h0,o1);
#endif
}
#endif
#ifdef Q
#define O5 o2
#define P5 l3
#else
#define O5 L1
#define P5 Y1
#endif
#ifdef ID
O5(JB){
#ifdef HB
A(O,g);
#else
A(O,E);
#endif
A(B0,N);c Y7;
#ifdef HB
if(HB&&Nb(O)){Y7=v4(O d1);}else if(HB&&Ob(O)){Y7=Z7(O d1);}else
#endif
{Y7=min(min(G0(O.x),abs(G0(O.y))),G0(1.));}i R=C0(.0);
#ifdef I
p4 o1=S7;
#endif
uint a8=Le(Y7);uint Pb=(Qb(B0)<<Q5)|a8;uint p2=W4(q4,Pb);N B1=W1(p2>>Q5);B1=U7(B1);if(B1==B0){if(!R5(O)){a8+=p2-max(Pb,p2);a8-=O9;X4(q4,a8);}}else{c p0=T7(p2&c8);V7(B1,p0,R
#ifdef I
,o1
#endif
S2 M1);}R.xyz=E2(R.xyz,R.w,Y.xy,n.z3,n.A3);
#ifdef Q
C1=R;
#else
X7(R M1);
#endif
#ifdef I
N9(o1 M1);
#endif
P5}
#endif
#if defined(EB)||defined(FB)
O5(JB){
#ifdef FB
A(C2,d);
#else
A(i1,c);
#endif
A(B0,N);uint p2=T2(q4);N B1=W1(p2>>Q5);B1=U7(B1);uint P9;
#ifndef FB
if(B1==B0){P9=p2;}else
#endif
{P9=(Qb(B0)<<Q5)+O9;}c o;
#ifdef FB
o=clamp(n2(BD,Q9,C2,.0).x,G0(.0),G0(1.));
#else
o=i1;
#endif
int Qe=int(round(o*I9));U2(q4,P9+uint(Qe));i R=C0(.0);
#ifdef I
p4 o1=S7;
#endif
#ifndef FB
if(B1!=B0)
#endif
{c R9=T7(p2&c8);V7(B1,R9,R
#ifdef I
,o1
#endif
S2 M1);}R.xyz=E2(R.xyz,R.w,Y.xy,n.z3,n.A3);
#ifdef Q
C1=R;
#else
X7(R M1);
#endif
#ifdef I
N9(o1 M1);
#endif
P5}
#endif
#ifdef HE
O5(JB){A(X1,d);
#ifdef JD
A(R4,c);
#endif
#ifdef BB
A(L0,g);
#endif
A(H1,c);
#ifdef I
A(w3,N);
#endif
#ifdef AB
A(A1,N);
#endif
i w4=d8(IC,S5,X1);c T5=1.;
#ifdef JD
T5=min(R4,T5);
#endif
#ifdef BB
if(BB){c U4=f3(Y4(L0));T5=clamp(U4,G0(.0),T5);}
#endif
uint p2=T2(q4);N B1=W1(p2>>Q5);B1=U7(B1);c R9=T7(p2&c8);i R;
#ifdef I
p4 o1=S7;
#endif
V7(B1,R9,R
#ifdef I
,o1
#endif
S2 M1);
#ifdef I
if(I&&w3!=0u){p4 M0=Fb(o1)?o1:H9(h0);Jb(w3,M0,T5);}
#endif
#if!defined(Q)&&defined(AB)
if(AB&&A1!=N5){i K1=H0(j0)*(1.-R.w)+R;w4.xyz=Q4(C6(w4),K1,A1)*w4.w;}
#endif
w4*=T5*H1;
#if defined(BC)
w4=k3(w4);
#endif
R=R*(1.-w4.w)+w4;R.xyz=E2(R.xyz,R.w,Y.xy,n.z3,n.A3);
#ifdef Q
C1=R;
#else
X7(R M1);
#endif
#ifdef I
N9(o1 M1);
#endif
U2(q4,O9);P5}
#endif
#ifdef IE
O5(JB){
#ifndef Q
#ifdef LD
if(LD){x0(j0,unpackUnorm4x8(n.Re));}
#endif
#ifdef MD
if(MD){x0(j0,q1(IC,J));}
#endif
#ifdef CF
i j=H0(j0);x0(j0,j.zyxw);
#endif
#endif
U2(q4,n.Se);
#ifdef I
if(I){c1(h0,0u);}
#endif
#ifdef Q
discard;
#endif
P5}
#endif
#ifdef QC
#ifdef ZC
o2(JB)
#else
O5(JB)
#endif
{uint p2=T2(q4);c p0=T7(p2&c8);N B1=W1(p2>>Q5);B1=U7(B1);i R;V7(B1,p0,R S2 M1);
#ifdef ZC
float I6=1.-R.w;if(I6!=.0)R+=H0(j0)*I6;C1=R;l3
#else
R.xyz=E2(R.xyz,R.w,Y.xy,n.z3,n.A3);
#ifdef Q
C1=R;
#else
X7(R M1);
#endif
P5
#endif
}
#endif
#endif

