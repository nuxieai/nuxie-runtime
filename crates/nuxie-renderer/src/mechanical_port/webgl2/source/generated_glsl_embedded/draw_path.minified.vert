#undef T2
#ifdef IB
#define T2 e
#else
#define T2 F
#endif
#ifdef BB
f1(d0)
#if defined(DB)||defined(FB)
K(0,l4,MB);
#else
K(0,e,YB);K(1,e,ZB);
#endif
g1
#endif
n2 I0 W(0,e,N0);
#ifdef FB
I0 W(1,c,W2);
#elif!defined(CB)
#ifdef DB
NB W(1,d,r1);
#else
I0 W(2,T2,S);
#endif
NB W(3,d,J0);
#endif
#ifdef N
#ifdef FB
NB W(4,d,j4);
#else
NB W(4,F,l2);
#endif
#endif
#if defined(AB)&&!defined(CB)
I0 W(5,e,Y0);
#endif
#ifdef H
NB W(6,d,S0);
#endif
#ifdef TB
m3 W(7,R0,D3);W(8,c,V4);
#endif
#ifdef GB
I0 W(9,M,O0);
#endif
P1
#ifdef BB
l1(UB,d0,A,C,r){
#if defined(DB)||defined(FB)
L(C,A,MB,M);
#else
L(C,A,YB,e);L(C,A,ZB,e);
#endif
V(N0,e);
#if defined(GB)
V(O0,M);
#endif
#ifdef FB
V(W2,c);
#elif!defined(CB)
#ifdef DB
V(r1,d);
#else
V(S,T2);
#endif
V(J0,d);
#endif
#ifdef N
#ifdef FB
V(j4,d);
#else
V(l2,F);
#endif
#endif
#if defined(AB)&&!defined(CB)
V(Y0,e);
#endif
#ifdef H
V(S0,d);
#endif
#ifdef TB
V(D3,R0);V(V4,c);
#endif
bool Pf=false;uint g0;c i0;
#ifdef CB
O I6;
#endif
#ifdef FB
i0=jd(MB,g0,
#ifdef CB
I6,
#endif
W2 T3);
#elif defined(DB)
i0=kd(MB,g0
#ifdef CB
,I6
#else
,r1
#endif
T3);
#else
e T;Pf=!va(YB,ZB,r,g0,i0
#ifndef CB
,T
#else
,I6
#endif
T3);
#ifndef CB
#ifdef IB
S=T;
#else
S.xy=J8(T.xy);
#endif
#endif
#endif
R0 x1=o0(JB,g0*4u+1u).zw;
#if!defined(FB)&&!defined(CB)
J0=n9(g0,j.z6);if((x1.x&Qa)!=0u) J0=-J0;
#endif
uint A2=x1.x&0xfu;
#ifdef N
if(N){uint hk=(A2==R8?x1.y:x1.x)>>16;d C1=n9(hk,j.z6);if(A2==R8) C1=-C1;
#ifdef FB
j4=C1;
#else
l2.x=C1;
#endif
}
#endif
#ifdef H
if(H){S0=float((x1.x>>4)&0xfu);}
#endif
c m0=i0;
#ifdef VD
if(j.Ha!=0u){m0.y=float(j.Ia)-m0.y;}
#endif
#ifdef AB
if(AB){X N3=v1(o0(KB,g0*p2+2u));e a4=o0(KB,g0*p2+3u);
#ifndef CB
Y0=L8(N3,a4.xy,m0);
#else
tb(N3,a4.xy,m0 H4);
#endif
}
#endif
if(A2==Ra){N0=e(unpackUnorm4x8(x1.y));}
#if defined(N)&&!defined(FB)
else if(N&&A2==R8){d i6=n9(x1.x>>16,j.z6);l2.y=i6;}
#endif
else{X Pb=v1(o0(KB,g0*p2));e U7=o0(KB,g0*p2+1u);float w4=uintBitsToFloat(x1.y);N0=Ja(m0,Pb,U7.xy,U7.zw,A2,w4,1.0);}
#if defined(GB)
if(GB&&(x1.x&le)!=0u){X Qb=v1(o0(KB,g0*p2+4u));e V7=o0(KB,g0*p2+5u);c z3=F0(Qb,m0)+V7.xy;float W7=1.+V7.z;if((x1.x&me)!=0u){uint G3=(x1.x&oe)>>ne;W7=-(1.+float(G3));}O0=M(z3.x,z3.y,W7);}else{O0=M(0.0,0.0,0.0);}
#endif
e I;if(!Pf){I=U3(i0);
#ifdef OC
I.y=-I.y;
#endif
#ifdef CB
I.z=o9(I6,0xffu);
#elif defined(TB)
R A5=o0(JB,g0*4u+3u);D3=A5.xy;V4=i0+uintBitsToFloat(A5.zw);
#endif
}else{I=e(j.n3,j.n3,j.n3,j.n3);}e0(N0);
#if defined(GB)
e0(O0);
#endif
#ifdef FB
e0(W2);
#elif!defined(CB)
#ifdef DB
e0(r1);
#else
e0(S);
#endif
e0(J0);
#endif
#ifdef N
#ifdef FB
e0(j4);
#else
e0(l2);
#endif
#endif
#if defined(AB)&&!defined(CB)
e0(Y0);
#endif
#ifdef H
e0(S0);
#endif
#ifdef TB
e0(D3);e0(V4);
#endif
B1(I);}
#endif
#ifdef EB
n4 p4 f i C8(
#ifdef GB
M D8,
#endif
#ifdef H
O c2,
#endif
e ma n7){
#ifdef H
bool J2=H&&c2!=X3;
#else
const bool J2=false;
#endif
i k;if(ma.w>=.0){k=y4(ma);}else{c Za=ab(ma,j.T8,j.U8);k=q2(YC,V8,Za,.0);if(!J2){d w4=ze(ma);k.xyz*=k.w;k.w*=w4;}}
#if defined(GB)
if(GB&&D8.z<0.0){return j5(SB,x4,D8.xy,A0(.0));}if(GB&&D8.z>0.0){d Rb=D8.z-1.;i T1=j5(SB,x4,D8.xy,Rb);if(J2) T1=z0(m6(T1),T1.w);k*=T1;}
#endif
return k;}
#if!defined(DB)&&!defined(FB)
f d Qf(T2 T g4){
#ifdef IB
if(IB&&sd(T)) return e5(T p1);else
#endif
return min(T.x,T.y);}f d Rf(T2 T g4){
#if defined(IB)
if(IB&&td(T)) return Y8(T p1);else
#endif
return T.x;}f d Tc(T2 T g4){if(r6(T)) return Qf(T p1);else return Rf(T p1);}f d ik(d B5,T2 T g4){if(r6(T)){d E0=Qf(T p1);return max(E0,B5);}else{d E0=Rf(T p1);return B5+E0;}}
#endif
#endif
