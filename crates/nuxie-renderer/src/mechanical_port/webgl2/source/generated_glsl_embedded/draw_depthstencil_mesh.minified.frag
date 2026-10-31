#ifdef DB
#ifdef NB
Y3 r3(B5,z4,RB);
#if defined(H)&&!defined(Q)&&!defined(QB)
P5(LD);
#endif
Z3
#ifdef QB
F1 Q5(z2,RC);G1
#endif
C5 A4(v4) D5
#endif
#ifdef QB
I1(HB)
#else
M2(i,HB)
#endif
{
#ifdef NB
q(e6,c);q(Y1,i);
#ifdef H
q(O1,O);
#endif
#else
q(M0,e);
#ifdef GB
q(N0,M);
#endif
#ifdef FB
q(W2,c);
#endif
#ifdef H
q(R0,d);
#endif
#endif
#ifdef NB
i k=h8(RB,v4,e6,j.Se)*Y1;
#else
d n=
#ifdef FB
clamp(q2(ID,Ua,W2,.0).x,C0(.0),C0(1.));
#else
1.;
#endif
i k=y8(
#ifdef GB
N0,
#endif
#ifdef H
Y2(R0),
#endif
M0 n3);
#endif
#if defined(H)&&!defined(Q)
#ifdef NB
k.xyz=j6(k);O Z1=O1;
#else
O Z1=Y2(R0);
#endif
#ifdef QB
i Z0=L5(RC);
#else
i Z0=M5(LD);
#endif
k.xyz=R4(k.xyz,Z0,Z1)*k.w;
#endif
#ifndef NB
k*=n;
#endif
k.xyz=K2(k.xyz,k.w,c0.xy,j.I3,j.J3);
#ifdef QB
if(k.w!=.0){N5(RC,Qa(k,Z0));}
#endif
#ifdef QB
P1
#else
N2(k);
#endif
}
#endif
