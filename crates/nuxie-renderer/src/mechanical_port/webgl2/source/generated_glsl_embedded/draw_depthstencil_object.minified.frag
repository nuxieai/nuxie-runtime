#ifdef FB
#ifdef OB
F3 a3(c5,X3,HC);
#ifdef AB
m5(YD);
#endif
G3 d5 Y3(V5)e5
#endif
c3(i,IB){
#ifdef OB
r(G5,c);r(H1,i);
#ifdef AB
r(A1,L);
#endif
#else
r(V1,f);
#ifdef JB
r(B2,Q);
#endif
#ifdef GB
r(E2,c);
#endif
#ifdef AB
r(g2,d);
#endif
#endif
#ifdef OB
i j=B7(HC,V5,G5,l.Ed)*H1;
#else
d o=
#ifdef GB
clamp(i2(FD,S9,E2,.0).x,I0(.0),I0(1.));
#else
1.;
#endif
i j=M7(V1,
#ifdef JB
B2,
#endif
o V2);
#endif
#if defined(AB)&&!defined(O)
#ifdef OB
j.xyz=G6(j);L U3=A1;
#else
L U3=a6(g2);
#endif
i L1=x6(YD);j.xyz=U4(j.xyz,L1,U3);j.xyz*=j.w;
#endif
#ifdef AC
if(AC){j=l3(j);}
#endif
j.xyz=I2(j.xyz,j.w,c0.xy,l.C3,l.D3);K2(j);}
#endif
