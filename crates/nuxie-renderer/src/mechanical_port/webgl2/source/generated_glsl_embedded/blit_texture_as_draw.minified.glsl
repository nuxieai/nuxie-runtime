m2
#ifdef DD
H0 X(0,d,Y1);
#endif
g2
#ifdef DB
U3 V3 C4 D4 g1(e0)h1 z1(GF,e0,F,B,v){d n2;n2.x=(B&1)==0?-1.:1.;n2.y=(B&2)==0?-1.:1.;
#ifdef DD
V(Y1,d);Y1.x=n2.x*.5+.5;Y1.y=n2.y*-.5+.5;c0(Y1);
#endif
g W=g(n2,0,1);A1(W);}
#endif
#ifdef GB
E3
#ifdef QD
kf(d5,W3,KC);
#else
Z2(d5,W3,KC);
#endif
F3
#ifdef DD
e5 X3(lf)f5
#endif
a3(i,NE){i l8;
#ifdef DD
r(Y1,d);l8=U6(KC,lf,Y1,.0);
#elif defined(QD)
l8=(m8(KC,0,Y(floor(a0.xy)))+m8(KC,1,Y(floor(a0.xy)))+m8(KC,2,Y(floor(a0.xy)))+m8(KC,3,Y(floor(a0.xy))))*0.25;
#else
l8=q1(KC,Y(floor(a0.xy)));
#endif
I2(l8);}
#endif
