#ifdef EB
#ifdef NB
X3 q3(C5,x4,UB);
#if defined(H)&&!defined(R)&&!defined(QB)
R5(MD);
#endif
Y3
#ifdef QB
F1 S5(x2,RC);G1
#endif
D5 y4(W4) E5
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
q(P0,e);
#ifdef GB
q(V0,M);
#endif
#ifdef FB
q(U2,c);
#endif
#ifdef H
q(Q0,d);
#endif
#endif
#ifdef NB
i k=i8(UB,W4,g6,j.Je)*Y1;
#else
d n=
#ifdef FB
clamp(p2(JD,Sa,U2,.0).x,J0(.0),J0(1.));
#else
1.;
#endif
i k=y8(
#ifdef GB
V0,
#endif
#ifdef H
W2(Q0),
#endif
P0 m3);
#endif
#if defined(H)&&!defined(R)
#ifdef NB
k.xyz=l6(k);P Z1=N1;
#else
P Z1=W2(Q0);
#endif
#ifdef QB
i a1=N5(RC);
#else
i a1=O5(MD);
#endif
k.xyz=P4(k.xyz,a1,Z1)*k.w;
#endif
#ifndef NB
k*=n;
#endif
k.xyz=J2(k.xyz,k.w,c0.xy,j.G3,j.H3);
#ifdef QB
if(k.w!=.0){P5(RC,Oa(k,a1));}
#endif
#ifdef QB
O1
#else
M2(k);
#endif
}
#endif
