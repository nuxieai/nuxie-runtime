#ifdef EB
#ifdef OB
Y3 Y2(I5,C4,SB);
#if defined(H)&&!defined(Q)&&!defined(LB)
J4(LD);
#endif
Z3
#ifdef LB
H1 K4(B2,LC);I1
#endif
J5 D4(x4) K5
#endif
#ifdef LB
D1(HB)
#else
r2(i,HB)
#endif
{
#ifdef OB
q(h6,c);q(a2,i);
#ifdef H
q(Q1,O);
#endif
#else
q(N0,e);
#ifdef GB
q(O0,M);
#endif
#ifdef FB
q(W2,c);
#endif
#ifdef H
q(S0,d);
#endif
#endif
#ifdef OB
i k=i8(SB,x4,h6,j.We)*a2;
#else
d n=
#ifdef FB
clamp(q2(ID,Xa,W2,.0).x,A0(.0),A0(1.));
#else
1.;
#endif
i k=C8(
#ifdef GB
O0,
#endif
#ifdef H
Z2(S0),
#endif
N0 r3);
#endif
#if defined(H)&&!defined(Q)
#ifdef OB
k.xyz=m6(k);O c2=Q1;
#else
O c2=Z2(S0);
#endif
#ifdef LB
i X0=e4(LC);
#else
i X0=I4(LD);
#endif
k.xyz=X4(k.xyz,X0,c2)*k.w;
#elif defined(LB)
i X0=e4(LC);
#endif
#ifndef OB
k*=n;
#endif
k.xyz=M2(k.xyz,k.w,Y.xy,j.K3,j.L3);
#ifdef LB
if(any(notEqual(k,z0(.0)))){q5(LC,Ta(k,X0));}
#endif
#ifdef LB
K1
#else
v2(k);
#endif
}
#endif
