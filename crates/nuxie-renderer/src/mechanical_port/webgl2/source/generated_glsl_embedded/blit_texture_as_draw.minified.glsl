m2
#ifdef CD
H0 W(0,d,Y1);
#endif
g2
#ifdef DB
T3 U3 B4 C4 g1(e0)h1 z1(FF,e0,F,B,A){d n2;n2.x=(B&1)==0?-1.:1.;n2.y=(B&2)==0?-1.:1.;
#ifdef CD
U(Y1,d);Y1.x=n2.x*.5+.5;Y1.y=n2.y*-.5+.5;c0(Y1);
#endif
g V=g(n2,0,1);A1(V);}
#endif
#ifdef GB
D3
#ifdef PD
kf(d5,V3,IC);
#else
Z2(d5,V3,IC);
#endif
E3
#ifdef CD
e5 W3(lf)f5
#endif
a3(i,ME){i l8;
#ifdef CD
r(Y1,d);l8=U6(IC,lf,Y1,.0);
#elif defined(PD)
l8=(m8(IC,0,Y(floor(a0.xy)))+m8(IC,1,Y(floor(a0.xy)))+m8(IC,2,Y(floor(a0.xy)))+m8(IC,3,Y(floor(a0.xy))))*0.25;
#else
l8=q1(IC,Y(floor(a0.xy)));
#endif
I2(l8);}
#endif
