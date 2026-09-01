l2
#ifdef DD
H0 X(0,d,X1);
#endif
f2
#ifdef DB
U3 V3 B4 C4 g1(e0)h1 y1(FF,e0,F,B,v){d m2;m2.x=(B&1)==0?-1.:1.;m2.y=(B&2)==0?-1.:1.;
#ifdef DD
V(X1,d);X1.x=m2.x*.5+.5;X1.y=m2.y*-.5+.5;a0(X1);
#endif
g W=g(m2,0,1);z1(W);}
#endif
#ifdef GB
E3
#ifdef PD
ef(a5,W3,KC);
#else
Z2(a5,W3,KC);
#endif
F3
#ifdef DD
c5 X3(ff)d5
#endif
a3(i,NE){i k8;
#ifdef DD
r(X1,d);k8=S6(KC,ff,X1,.0);
#elif defined(PD)
k8=(l8(KC,0,Y(floor(Z.xy)))+l8(KC,1,Y(floor(Z.xy)))+l8(KC,2,Y(floor(Z.xy)))+l8(KC,3,Y(floor(Z.xy))))*0.25;
#else
k8=q1(KC,Y(floor(Z.xy)));
#endif
I2(k8);}
#endif
