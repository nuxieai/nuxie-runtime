#ifdef DB
#if(defined(R)&&!defined(N))||defined(SB)
#undef Wc
#else
#define Wc
#endif
F1
#ifndef R
y0(x2,n0);
#endif
#ifndef SB
r1(k3,m0);
#ifndef R
y0(E6,M4);
#endif
r1(m7,Y0);
#else
y0(k3,m0);
#endif
G1
#ifdef NB
Y3 q3(D5,z4,RB);Z3 E5 A4(v4) F5 m4 n4
#endif
#ifdef R
#ifdef NB
H2(HB)
#else
H2(HB)
#endif
#else
#ifdef NB
I1(HB)
#else
I1(HB)
#endif
#endif
{
#ifdef FB
q(N0,e);
#if defined(GB)
q(O0,M);
#endif
q(U2,c);
#endif
#ifdef N
q(i4,d);
#endif
#ifdef AB
q(W0,e);
#endif
#if defined(FB)&&defined(H)
q(S0,d);
#endif
#ifdef NB
q(g6,c);q(Y1,i);
#ifdef H
q(N1,P);
#endif
#endif
#ifdef FB
i k=z8(
#ifdef GB
O0,
#endif
#ifdef H
W2(S0),
#endif
N0 m3);d n=clamp(p2(JD,Ta,U2,.0).x,C0(.0),C0(1.));
#endif
#ifdef NB
i k=j8(RB,v4,g6,j.Re);d n=1.;
#endif
#ifdef AB
if(AB){d A5=max(C3(Y4(W0)),C0(.0));n=min(A5,n);}
#endif
#ifdef Wc
P2;
#endif
#if defined(N)
if(N&&i4!=.0){d S3;
#ifndef SB
E X0=unpackHalf2x16(k1(m0));d i7=X0.y;S3=max(i7==i4?X0.x:C0(.0),C0(.0));
#else
S3=K0(m0).x;
#endif
S3=max(S3,C0(.0));n=min(n,S3);}
#endif
#ifdef NB
k*=Y1;
#endif
#if!defined(R)
i a1=K0(n0);
#ifdef H
#ifdef FB
P Z1=W2(S0);
#endif
#ifdef NB
P Z1=N1;
#endif
if(H&&Z1!=X3){
#ifdef NB
k.xyz=l6(k);
#endif
k.xyz=R4(k.xyz,a1,Z1)*k.w;}
#endif
k*=n;k.xyz=J2(k.xyz,k.w,c0.xy,j.H3,j.I3);
#ifndef SB
k=a1*(1.-k.w)+k;
#endif
x0(n0,k);
#endif
#ifndef SB
i2(m0);i2(Y0);
#else
x0(m0,E0(.0));
#endif
#ifdef Wc
Q2;
#endif
#ifdef R
k=(k*n);k.xyz=J2(k.xyz,k.w,c0.xy,j.H3,j.I3);Q1=k;G3
#else
O1;
#endif
}
#endif
