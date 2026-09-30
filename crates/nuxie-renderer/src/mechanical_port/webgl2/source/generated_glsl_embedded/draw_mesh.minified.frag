#ifdef EB
#if(defined(Q)&&!defined(K))||defined(QB)
#undef Bb
#else
#define Bb
#endif
M1
#ifndef Q
z0(H2,l0);
#endif
#ifndef QB
j1(Y2,i0);
#ifndef Q
z0(f6,q4);
#endif
j1(K6,S0);
#else
z0(Y2,i0);
#endif
N1
#ifdef KB
H3 e3(i5,a4,HC);I3 j5 c4(W5) k5 T3 U3
#endif
#ifdef Q
#ifdef KB
w2(HB)
#else
w2(HB)
#endif
#else
#ifdef KB
P1(HB)
#else
P1(HB)
#endif
#endif
{
#ifdef FB
r(X1,f);
#if defined(IB)
r(D2,S);
#endif
r(G2,c);
#endif
#ifdef K
r(N3,d);
#endif
#ifdef AB
r(P0,f);
#endif
#if defined(FB)&&defined(T)
r(f1,d);
#endif
#ifdef KB
r(J5,c);r(K1,i);
#ifdef T
r(C1,N);
#endif
#endif
#ifdef FB
i k=K7(
#ifdef IB
D2,
#endif
#ifdef T
g3(f1),
#endif
X1 Z2);d o=clamp(j2(FD,S9,G2,.0).x,J0(.0),J0(1.));
#endif
#ifdef KB
i k=z7(HC,W5,J5,j.Dd);d o=1.;
#endif
#ifdef AB
if(AB){d d5=max(m3(h5(P0)),J0(.0));o=min(d5,o);}
#endif
#ifdef Bb
A2;
#endif
#if defined(K)
if(K&&N3!=.0){d z3;
#ifndef QB
D Q0=unpackHalf2x16(a1(i0));d F6=Q0.y;z3=max(F6==N3?Q0.x:J0(.0),J0(.0));
#else
z3=K0(i0).x;
#endif
z3=max(z3,J0(.0));o=min(o,z3);}
#endif
#ifdef KB
k*=K1;
#endif
#if!defined(Q)
i O1=K0(l0);
#ifdef T
#ifdef FB
N p3=g3(f1);
#endif
#ifdef KB
N p3=C1;
#endif
if(T&&p3!=C4){
#ifdef KB
k.xyz=G6(k);
#endif
k.xyz=Z4(k.xyz,O1,p3)*k.w;}
#endif
k*=o;
#ifdef BC
if(BC){k=q3(k);}
#endif
k.xyz=L2(k.xyz,k.w,e0.xy,j.F3,j.G3);
#ifndef QB
k=O1*(1.-k.w)+k;
#endif
A0(l0,k);
#endif
#ifndef QB
h2(i0);h2(S0);
#else
A0(i0,E0(.0));
#endif
#ifdef Bb
B2;
#endif
#ifdef Q
k=(k*o);k.xyz=L2(k.xyz,k.w,e0.xy,j.F3,j.G3);E1=k;r3
#else
d2;
#endif
}
#endif
