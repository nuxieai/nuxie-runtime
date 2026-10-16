#ifdef EB
#if(defined(U)&&!defined(N))||defined(QB)
#undef Pc
#else
#define Pc
#endif
E1
#ifndef U
z0(x2,n0);
#endif
#ifndef QB
q1(k3,m0);
#ifndef U
z0(A6,I4);
#endif
q1(l7,Y0);
#else
z0(k3,m0);
#endif
F1
#ifdef NB
V3 q3(B5,v4,TB);W3 C5 w4(U4) D5 k4 l4
#endif
#ifdef U
#ifdef NB
H2(IB)
#else
H2(IB)
#endif
#else
#ifdef NB
X1(IB)
#else
X1(IB)
#endif
#endif
{
#ifdef FB
q(P0,f);
#if defined(GB)
q(U0,M);
#endif
q(T2,c);
#endif
#ifdef N
q(f4,d);
#endif
#ifdef AB
q(V0,f);
#endif
#if defined(FB)&&defined(H)
q(Q0,d);
#endif
#ifdef NB
q(d6,c);q(V1,i);
#ifdef H
q(L1,P);
#endif
#endif
#ifdef FB
i k=x8(
#ifdef GB
U0,
#endif
#ifdef H
W2(Q0),
#endif
P0 m3);d n=clamp(n2(JD,Sa,T2,.0).x,J0(.0),J0(1.));
#endif
#ifdef NB
i k=h8(TB,U4,d6,j.Ge);d n=1.;
#endif
#ifdef AB
if(AB){d y5=max(C3(V4(V0)),J0(.0));n=min(y5,n);}
#endif
#ifdef Pc
O2;
#endif
#if defined(N)
if(N&&f4!=.0){d P3;
#ifndef QB
D W0=unpackHalf2x16(k1(m0));d h7=W0.y;P3=max(h7==f4?W0.x:J0(.0),J0(.0));
#else
P3=O0(m0).x;
#endif
P3=max(P3,J0(.0));n=min(n,P3);}
#endif
#ifdef NB
k*=V1;
#endif
#if!defined(U)
i a1=O0(n0);
#ifdef H
#ifdef FB
P W1=W2(Q0);
#endif
#ifdef NB
P W1=L1;
#endif
if(H&&W1!=U3){
#ifdef NB
k.xyz=i6(k);
#endif
k.xyz=N4(k.xyz,a1,W1)*k.w;}
#endif
k*=n;k.xyz=J2(k.xyz,k.w,d0.xy,j.G3,j.H3);
#ifndef QB
k=a1*(1.-k.w)+k;
#endif
x0(n0,k);
#endif
#ifndef QB
g2(m0);g2(Y0);
#else
x0(m0,H0(.0));
#endif
#ifdef Pc
P2;
#endif
#ifdef U
k=(k*n);k.xyz=J2(k.xyz,k.w,d0.xy,j.G3,j.H3);N1=k;F3
#else
o2;
#endif
}
#endif
