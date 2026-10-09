#ifdef EB
#ifdef NB
V3 p3(A5,v4,TB);
#ifdef H
N5(KD);
#endif
W3 B5 w4(U4) C5
#endif
V2(i,IB){
#ifdef NB
q(d6,c);q(T1,i);
#ifdef H
q(J1,P);
#endif
#else
q(O0,f);
#ifdef GB
q(U0,M);
#endif
#ifdef FB
q(S2,c);
#endif
#ifdef H
q(P0,d);
#endif
#endif
#ifdef NB
i l=e8(TB,U4,d6,j.Ee)*T1;
#else
d n=
#ifdef FB
clamp(n2(HD,Pa,S2,.0).x,J0(.0),J0(1.));
#else
1.;
#endif
i l=r8(
#ifdef GB
U0,
#endif
#ifdef H
W2(P0),
#endif
O0 l3);
#endif
#if defined(H)&&!defined(U)
#ifdef NB
l.xyz=i6(l);P W1=J1;
#else
P W1=W2(P0);
#endif
i z1=L5(KD);l.xyz=N4(l.xyz,z1,W1)*l.w;
#endif
#ifndef NB
l*=n;
#endif
l.xyz=I2(l.xyz,l.w,d0.xy,j.F3,j.G3);K2(l);}
#endif
