#ifdef EB
#if(defined(Q)&&!defined(N))||defined(TB)
#undef bd
#else
#define bd
#endif
H1
#ifndef Q
y0(B2,p0);
#endif
#ifndef TB
w1(p3,n0);
#ifndef Q
y0(D6,S4);
#endif
w1(l7,c1);
#else
y0(p3,n0);
#endif
I1
#ifdef OB
Y3 Y2(I5,C4,SB);Z3 J5 D4(x4) K5 n4 p4
#endif
#ifdef Q
#ifdef OB
K2(HB)
#else
K2(HB)
#endif
#else
#ifdef OB
D1(HB)
#else
D1(HB)
#endif
#endif
{
#ifdef FB
q(N0,e);
#if defined(GB)
q(O0,M);
#endif
q(W2,c);
#endif
#ifdef N
q(j4,d);
#endif
#ifdef AB
q(Y0,e);
#endif
#if defined(FB)&&defined(H)
q(S0,d);
#endif
#ifdef OB
q(h6,c);q(a2,i);
#ifdef H
q(Q1,O);
#endif
#endif
#ifdef FB
i k=C8(
#ifdef GB
O0,
#endif
#ifdef H
Z2(S0),
#endif
N0 r3);d n=clamp(q2(ID,Xa,W2,.0).x,A0(.0),A0(1.));
#endif
#ifdef OB
i k=i8(SB,x4,h6,j.We);d n=1.;
#endif
#ifdef AB
if(AB){d F5=max(F3(y4(Y0)),A0(.0));n=min(F5,n);}
#endif
#ifdef bd
R2;
#endif
#if defined(N)
if(N&&j4!=.0){d S3;
#ifndef TB
F Z0=unpackHalf2x16(m1(n0));d h7=Z0.y;S3=max(h7==j4?Z0.x:A0(.0),A0(.0));
#else
S3=L0(n0).x;
#endif
S3=max(S3,A0(.0));n=min(n,S3);}
#endif
#ifdef OB
k*=a2;
#endif
#if!defined(Q)
i X0=L0(p0);
#ifdef H
#ifdef FB
O c2=Z2(S0);
#endif
#ifdef OB
O c2=Q1;
#endif
if(H&&c2!=X3){
#ifdef OB
k.xyz=m6(k);
#endif
k.xyz=X4(k.xyz,X0,c2)*k.w;}
#endif
k*=n;k.xyz=M2(k.xyz,k.w,Y.xy,j.K3,j.L3);
#ifndef TB
k=X0*(1.-k.w)+k;
#endif
x0(p0,k);
#endif
#ifndef TB
k2(n0);k2(c1);
#else
x0(n0,z0(.0));
#endif
#ifdef bd
S2;
#endif
#ifdef Q
k=(k*n);k.xyz=M2(k.xyz,k.w,Y.xy,j.K3,j.L3);S1=k;J3
#else
K1;
#endif
}
#endif
