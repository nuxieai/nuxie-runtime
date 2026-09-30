#ifdef FB
#ifdef OB
F3 c3(d5,X3,HC);
#ifdef AB
n5(YD);
#endif
G3 e5 Y3(X5)f5
#endif
d3(i,IB){
#ifdef OB
r(H5,c);r(H1,i);
#ifdef AB
r(A1,L);
#endif
#else
r(V1,f);
#ifdef JB
r(C2,Q);
#endif
#ifdef GB
r(F2,c);
#endif
#ifdef AB
r(g2,d);
#endif
#endif
#ifdef OB
i j=B7(HC,X5,H5,l.Dd)*H1;
#else
d o=
#ifdef GB
clamp(i2(FD,R9,F2,.0).x,I0(.0),I0(1.));
#else
1.;
#endif
i j=M7(V1,
#ifdef JB
C2,
#endif
o W2);
#endif
#if defined(AB)&&!defined(O)
#ifdef OB
j.xyz=H6(j);L U3=A1;
#else
L U3=d6(g2);
#endif
i L1=z6(YD);j.xyz=U4(j.xyz,L1,U3);j.xyz*=j.w;
#endif
#ifdef AC
if(AC){j=l3(j);}
#endif
j.xyz=J2(j.xyz,j.w,c0.xy,l.C3,l.D3);L2(j);}
#endif
