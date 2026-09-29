#ifdef GB
#ifdef PB
E3 Z2(d5,W3,JC);
#ifdef AB
k7(VD);
#endif
F3 e5 X3(W5)f5
#endif
a3(i,JB){
#ifdef PB
r(H5,d);r(I1,c);
#ifdef AB
r(B1,N);
#endif
#else
r(f1,g);
#ifdef KB
r(A2,R);
#endif
#ifdef FB
r(D2,d);
#endif
#ifdef AB
r(f2,c);
#endif
#endif
#ifdef PB
i j=B7(JC,W5,H5,m.ud)*I1;
#else
c n=
#ifdef FB
clamp(o2(CD,Q9,D2,.0).x,G0(.0),G0(1.));
#else
1.;
#endif
i j=M7(f1,
#ifdef KB
A2,
#endif
n U2);
#endif
#if defined(AB)&&!defined(Q)
#ifdef PB
j.xyz=G6(j);N T3=B1;
#else
N T3=c6(f2);
#endif
i L1=T8(VD);j.xyz=U4(j.xyz,L1,T3);j.xyz*=j.w;
#endif
#ifdef CC
if(CC){j=m3(j);}
#endif
j.xyz=F2(j.xyz,j.w,a0.xy,m.B3,m.C3);I2(j);}
#endif
