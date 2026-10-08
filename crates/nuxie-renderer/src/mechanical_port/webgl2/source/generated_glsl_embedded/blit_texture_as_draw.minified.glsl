w2
#ifdef ID
F0 X(0,c,m2);
#endif
l2
#ifdef BB
o4 p4 W4 X4 f1(f0) g1 x1(NF,f0,B,F,r){c E2;E2.x=(F&1)==0?-1.:1.;E2.y=(F&2)==0?-1.:1.;
#ifdef ID
V(m2,c);m2.x=E2.x*.5+.5;m2.y=E2.y*-.5+.5;Z(m2);
#endif
e I=e(E2,0,1);y1(I);}
#endif
#ifdef EB
U3
#ifdef VD
Cg(x5,q4,IC);
#else
p3(x5,q4,IC);
#endif
V3
#ifdef ID
y5 r4(Dg) z5
#endif
W2(i,RE){i U8;
#ifdef ID
q(m2,c);U8=A5(IC,Dg,m2,.0);
#elif defined(VD)
U8=(V8(IC,0,g0(floor(d0.xy)))+V8(IC,1,g0(floor(d0.xy)))+V8(IC,2,g0(floor(d0.xy)))+V8(IC,3,g0(floor(d0.xy))))*0.25;
#else
U8=r1(IC,g0(floor(d0.xy)));
#endif
K2(U8);}
#endif
