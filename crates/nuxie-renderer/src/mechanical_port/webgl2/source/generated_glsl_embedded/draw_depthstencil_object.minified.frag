#ifdef FB
#ifdef OB
E3 c3(d5,W3,HC);
#ifdef AB
k6(YD);
#endif
F3 e5 X3(W5)f5
#endif
d3(i,IB){
#ifdef OB
r(G5,c);r(H1,i);
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
i j=A7(HC,W5,G5,n.Cd)*H1;
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
j.xyz=G6(j);L T3=A1;
#else
L T3=c6(g2);
#endif
i L1=F7(YD);j.xyz=U4(j.xyz,L1,T3);j.xyz*=j.w;
#endif
#ifdef AC
if(AC){j=l3(j);}
#endif
j.xyz=I2(j.xyz,j.w,c0.xy,n.B3,n.C3);K2(j);}
#endif
