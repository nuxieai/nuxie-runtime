#ifdef GB
#ifdef OB
D3 Z2(d5,V3,HC);
#ifdef AB
j6(UD);
#endif
E3 e5 W3(V5)f5
#endif
a3(i,IB){
#ifdef OB
r(G5,d);r(I1,c);
#ifdef AB
r(B1,K);
#endif
#else
r(f1,g);
#ifdef JB
r(A2,Q);
#endif
#ifdef FB
r(D2,d);
#endif
#ifdef AB
r(f2,c);
#endif
#endif
#ifdef OB
i j=A7(HC,V5,G5,m.ud)*I1;
#else
c n=
#ifdef FB
clamp(o2(BD,O9,D2,.0).x,G0(.0),G0(1.));
#else
1.;
#endif
i j=M7(f1,
#ifdef JB
A2,
#endif
n U2);
#endif
#if defined(AB)&&!defined(N)
#ifdef OB
j.xyz=F6(j);K S3=B1;
#else
K S3=a6(f2);
#endif
i L1=F7(UD);j.xyz=U4(j.xyz,L1,S3);j.xyz*=j.w;
#endif
#ifdef AC
if(AC){j=l3(j);}
#endif
j.xyz=F2(j.xyz,j.w,a0.xy,m.A3,m.B3);I2(j);}
#endif
