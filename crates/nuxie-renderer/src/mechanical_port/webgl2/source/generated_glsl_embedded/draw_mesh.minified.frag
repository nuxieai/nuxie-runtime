#ifdef DB
#if(defined(Q)&&!defined(N))||defined(SB)
#undef Xc
#else
#define Xc
#endif
F1
#ifndef Q
y0(z2,o0);
#endif
#ifndef SB
q1(l3,m0);
#ifndef Q
y0(B6,M4);
#endif
q1(j7,X0);
#else
y0(l3,m0);
#endif
G1
#ifdef NB
Y3 r3(B5,z4,RB);Z3 C5 A4(v4) D5 l4 n4
#endif
#ifdef Q
#ifdef NB
I2(HB)
#else
I2(HB)
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
q(M0,e);
#if defined(GB)
q(N0,M);
#endif
q(W2,c);
#endif
#ifdef N
q(h4,d);
#endif
#ifdef AB
q(V0,e);
#endif
#if defined(FB)&&defined(H)
q(R0,d);
#endif
#ifdef NB
q(e6,c);q(Y1,i);
#ifdef H
q(N1,O);
#endif
#endif
#ifdef FB
i k=x8(
#ifdef GB
N0,
#endif
#ifdef H
Y2(R0),
#endif
M0 n3);d n=clamp(q2(ID,Ua,W2,.0).x,C0(.0),C0(1.));
#endif
#ifdef NB
i k=g8(RB,v4,e6,j.Re);d n=1.;
#endif
#ifdef AB
if(AB){d y5=max(D3(Y4(V0)),C0(.0));n=min(y5,n);}
#endif
#ifdef Xc
R2;
#endif
#if defined(N)
if(N&&h4!=.0){d S3;
#ifndef SB
E W0=unpackHalf2x16(j1(m0));d f7=W0.y;S3=max(f7==h4?W0.x:C0(.0),C0(.0));
#else
S3=K0(m0).x;
#endif
S3=max(S3,C0(.0));n=min(n,S3);}
#endif
#ifdef NB
k*=Y1;
#endif
#if!defined(Q)
i Z0=K0(o0);
#ifdef H
#ifdef FB
O Z1=Y2(R0);
#endif
#ifdef NB
O Z1=N1;
#endif
if(H&&Z1!=X3){
#ifdef NB
k.xyz=j6(k);
#endif
k.xyz=R4(k.xyz,Z0,Z1)*k.w;}
#endif
k*=n;k.xyz=K2(k.xyz,k.w,c0.xy,j.I3,j.J3);
#ifndef SB
k=Z0*(1.-k.w)+k;
#endif
x0(o0,k);
#endif
#ifndef SB
j2(m0);j2(X0);
#else
x0(m0,E0(.0));
#endif
#ifdef Xc
S2;
#endif
#ifdef Q
k=(k*n);k.xyz=K2(k.xyz,k.w,c0.xy,j.I3,j.J3);Q1=k;H3
#else
O1;
#endif
}
#endif
