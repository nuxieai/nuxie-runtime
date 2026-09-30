#ifdef EB
#if(defined(Q)&&!defined(I))||defined(QB)
#undef Gb
#else
#define Gb
#endif
M1
#ifndef Q
z0(G2,m0);
#endif
#ifndef QB
k1(X2,i0);
#ifndef Q
z0(j6,p4);
#endif
k1(M6,S0);
#else
z0(X2,i0);
#endif
N1
#ifdef KB
I3 e3(h5,a4,GC);J3 i5 c4(Z5)j5 U3 V3
#endif
#ifdef Q
#ifdef KB
v2(HB)
#else
v2(HB)
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
r(C2,R);
#endif
r(F2,c);
#endif
#ifdef I
r(O3,d);
#endif
#ifdef AB
r(O0,f);
#endif
#if defined(FB)&&defined(S)
r(g1,d);
#endif
#ifdef KB
r(L5,c);r(K1,i);
#ifdef S
r(D1,N);
#endif
#endif
#ifdef FB
i k=O7(
#ifdef IB
C2,
#endif
#ifdef S
g3(g1),
#endif
X1 Y2);d o=clamp(j2(ED,U9,F2,.0).x,J0(.0),J0(1.));
#endif
#ifdef KB
i k=D7(GC,Z5,L5,j.Kd);d o=1.;
#endif
#ifdef AB
if(AB){d c5=max(m3(g5(O0)),J0(.0));o=min(c5,o);}
#endif
#ifdef Gb
z2;
#endif
#if defined(I)
if(I&&O3!=.0){d z3;
#ifndef QB
E Q0=unpackHalf2x16(a1(i0));d H6=Q0.y;z3=max(H6==O3?Q0.x:J0(.0),J0(.0));
#else
z3=K0(i0).x;
#endif
z3=max(z3,J0(.0));o=min(o,z3);}
#endif
#ifdef KB
k*=K1;
#endif
#if!defined(Q)
i O1=K0(m0);
#ifdef S
#ifdef FB
N p3=g3(g1);
#endif
#ifdef KB
N p3=D1;
#endif
if(S&&p3!=B4){
#ifdef KB
k.xyz=I6(k);
#endif
k.xyz=Z4(k.xyz,O1,p3)*k.w;}
#endif
k*=o;
#ifdef AC
if(AC){k=q3(k);}
#endif
k.xyz=K2(k.xyz,k.w,d0.xy,j.F3,j.G3);
#ifndef QB
k=O1*(1.-k.w)+k;
#endif
A0(m0,k);
#endif
#ifndef QB
h2(i0);h2(S0);
#else
A0(i0,E0(.0));
#endif
#ifdef Gb
A2;
#endif
#ifdef Q
k=(k*o);k.xyz=K2(k.xyz,k.w,d0.xy,j.F3,j.G3);F1=k;r3
#else
d2;
#endif
}
#endif
