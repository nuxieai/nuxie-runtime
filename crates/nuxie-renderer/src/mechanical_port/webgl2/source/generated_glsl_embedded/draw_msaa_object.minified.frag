#ifdef GB
#ifdef PB
E3 Z2(a5,W3,JC);
#ifdef AB
i7(UD);
#endif
F3 c5 X3(U5)d5
#endif
a3(i,JB){
#ifdef PB
r(F5,d);r(H1,c);
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
r(e2,c);
#endif
#endif
#ifdef PB
i j=z7(JC,U5,F5,m.sd)*H1;
#else
c n=
#ifdef FB
clamp(n2(CD,Q9,D2,.0).x,G0(.0),G0(1.));
#else
1.;
#endif
i j=L7(f1,
#ifdef KB
A2,
#endif
n U2);
#endif
#if defined(AB)&&!defined(Q)
#ifdef PB
j.xyz=E6(j);N T3=A1;
#else
N T3=Z5(e2);
#endif
i K1=S8(UD);j.xyz=S4(j.xyz,K1,T3);j.xyz*=j.w;
#endif
#ifdef CC
if(CC){j=m3(j);}
#endif
j.xyz=F2(j.xyz,j.w,Z.xy,m.B3,m.C3);I2(j);}
#endif
