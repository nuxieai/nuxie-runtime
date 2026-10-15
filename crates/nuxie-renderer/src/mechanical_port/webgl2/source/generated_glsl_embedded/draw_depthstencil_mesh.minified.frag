#ifdef EB
#ifdef NB
V3 q3(B5,v4,TB);
#if defined(H)&&!defined(KC)
O5(MD);
#endif
W3
#ifdef KC
E1 L6(x2,RC);F1
#endif
C5 w4(U4) D5
#endif
V2(i,IB){
#ifdef NB
q(d6,c);q(V1,i);
#ifdef H
q(L1,P);
#endif
#else
q(P0,f);
#ifdef GB
q(U0,M);
#endif
#ifdef FB
q(T2,c);
#endif
#ifdef H
q(Q0,d);
#endif
#endif
#ifdef NB
i k=h8(TB,U4,d6,j.Ge)*V1;
#else
d n=
#ifdef FB
clamp(n2(JD,Sa,T2,.0).x,J0(.0),J0(1.));
#else
1.;
#endif
i k=x8(
#ifdef GB
U0,
#endif
#ifdef H
W2(Q0),
#endif
P0 m3);
#endif
#if defined(H)&&!defined(U)
#ifdef NB
k.xyz=i6(k);P W1=L1;
#else
P W1=W2(Q0);
#endif
#ifdef KC
i a1=I6(RC);
#else
i a1=M5(MD);
#endif
k.xyz=N4(k.xyz,a1,W1)*k.w;
#endif
#ifndef NB
k*=n;
#endif
k.xyz=J2(k.xyz,k.w,d0.xy,j.F3,j.G3);
#ifdef KC
if(k.w!=.0){J6(RC,Oa(k,a1));}
#endif
L2(k);}
#endif
