#ifdef EB
#ifdef KB
I3 e3(h5,a4,HC);
#ifdef T
p5(YD);
#endif
J3 i5 c4(W5) j5
#endif
f3(i,HB){
#ifdef KB
r(J5,c);r(K1,i);
#ifdef T
r(D1,N);
#endif
#else
r(X1,f);
#ifdef IB
r(C2,S);
#endif
#ifdef FB
r(F2,c);
#endif
#ifdef T
r(g1,d);
#endif
#endif
#ifdef KB
i k=A7(HC,W5,J5,j.Ed)*K1;
#else
d o=
#ifdef FB
clamp(j2(FD,S9,F2,.0).x,J0(.0),J0(1.));
#else
1.;
#endif
i k=L7(
#ifdef IB
C2,
#endif
#ifdef T
g3(g1),
#endif
X1 Y2);
#endif
#if defined(T)&&!defined(Q)
#ifdef KB
k.xyz=F6(k);N p3=D1;
#else
N p3=g3(g1);
#endif
i O1=x6(YD);k.xyz=Y4(k.xyz,O1,p3)*k.w;
#endif
#ifndef KB
k*=o;
#endif
#ifdef BC
if(BC){k=q3(k);}
#endif
k.xyz=K2(k.xyz,k.w,d0.xy,j.F3,j.G3);M2(k);}
#endif
