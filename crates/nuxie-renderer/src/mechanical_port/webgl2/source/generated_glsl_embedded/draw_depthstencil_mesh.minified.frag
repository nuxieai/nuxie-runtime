#ifdef DB
#ifdef NB
Y3 q3(D5,z4,RB);
#if defined(H)&&!defined(R)&&!defined(QB)
R5(MD);
#endif
Z3
#ifdef QB
F1 S5(x2,RC);G1
#endif
E5 A4(v4) F5
#endif
#ifdef QB
I1(HB)
#else
L2(i,HB)
#endif
{
#ifdef NB
q(g6,c);q(Y1,i);
#ifdef H
q(N1,P);
#endif
#else
q(N0,e);
#ifdef GB
q(O0,M);
#endif
#ifdef FB
q(U2,c);
#endif
#ifdef H
q(S0,d);
#endif
#endif
#ifdef NB
i k=j8(RB,v4,g6,j.Re)*Y1;
#else
d n=
#ifdef FB
clamp(p2(JD,Ta,U2,.0).x,C0(.0),C0(1.));
#else
1.;
#endif
i k=z8(
#ifdef GB
O0,
#endif
#ifdef H
W2(S0),
#endif
N0 m3);
#endif
#if defined(H)&&!defined(R)
#ifdef NB
k.xyz=l6(k);P Z1=N1;
#else
P Z1=W2(S0);
#endif
#ifdef QB
i a1=N5(RC);
#else
i a1=O5(MD);
#endif
k.xyz=R4(k.xyz,a1,Z1)*k.w;
#endif
#ifndef NB
k*=n;
#endif
k.xyz=J2(k.xyz,k.w,c0.xy,j.H3,j.I3);
#ifdef QB
if(k.w!=.0){P5(RC,Pa(k,a1));}
#endif
#ifdef QB
O1
#else
M2(k);
#endif
}
#endif
