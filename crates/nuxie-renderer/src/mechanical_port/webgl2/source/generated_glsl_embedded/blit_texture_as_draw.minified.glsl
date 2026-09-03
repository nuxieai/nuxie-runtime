m2
#ifdef DD
H0 X(0,d,Y1);
#endif
g2
#ifdef DB
U3 V3 B4 C4 g1(e0)h1 y1(FF,e0,F,B,v){d n2;n2.x=(B&1)==0?-1.:1.;n2.y=(B&2)==0?-1.:1.;
#ifdef DD
V(Y1,d);Y1.x=n2.x*.5+.5;Y1.y=n2.y*-.5+.5;a0(Y1);
#endif
g W=g(n2,0,1);z1(W);}
#endif
#ifdef GB
E3
#ifdef PD
ff(c5,W3,KC);
#else
Z2(c5,W3,KC);
#endif
F3
#ifdef DD
d5 X3(gf)e5
#endif
a3(i,NE){i l8;
#ifdef DD
r(Y1,d);l8=T6(KC,gf,Y1,.0);
#elif defined(PD)
l8=(m8(KC,0,Y(floor(Z.xy)))+m8(KC,1,Y(floor(Z.xy)))+m8(KC,2,Y(floor(Z.xy)))+m8(KC,3,Y(floor(Z.xy))))*0.25;
#else
l8=q1(KC,Y(floor(Z.xy)));
#endif
I2(l8);}
#endif
