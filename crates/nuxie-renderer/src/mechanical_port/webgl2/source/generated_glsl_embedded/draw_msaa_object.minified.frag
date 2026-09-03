#ifdef GB
#ifdef PB
E3 Z2(c5,W3,JC);
#ifdef AB
j7(UD);
#endif
F3 d5 X3(V5)e5
#endif
a3(i,JB){
#ifdef PB
r(G5,d);r(H1,c);
#ifdef AB
r(A1,N);
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
i j=A7(JC,V5,G5,m.td)*H1;
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
j.xyz=F6(j);N T3=A1;
#else
N T3=a6(f2);
#endif
i K1=T8(UD);j.xyz=T4(j.xyz,K1,T3);j.xyz*=j.w;
#endif
#ifdef CC
if(CC){j=m3(j);}
#endif
j.xyz=F2(j.xyz,j.w,Z.xy,m.B3,m.C3);I2(j);}
#endif
