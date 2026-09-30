p2
#ifdef GD
H0 V(0,c,Z1);
#endif
h2
#ifdef DB
V3 W3 B4 C4 f1(f0)g1 y1(LF,f0,F,B,v){c q2;q2.x=(B&1)==0?-1.:1.;q2.y=(B&2)==0?-1.:1.;
#ifdef GD
T(Z1,c);Z1.x=q2.x*.5+.5;Z1.y=q2.y*-.5+.5;a0(Z1);
#endif
f W=f(q2,0,1);z1(W);}
#endif
#ifdef FB
F3
#ifdef TD
Bf(d5,X3,IC);
#else
c3(d5,X3,IC);
#endif
G3
#ifdef GD
e5 Y3(Cf)f5
#endif
d3(i,RE){i m8;
#ifdef GD
r(Z1,c);m8=V6(IC,Cf,Z1,.0);
#elif defined(TD)
m8=(n8(IC,0,Y(floor(c0.xy)))+n8(IC,1,Y(floor(c0.xy)))+n8(IC,2,Y(floor(c0.xy)))+n8(IC,3,Y(floor(c0.xy))))*0.25;
#else
m8=p1(IC,Y(floor(c0.xy)));
#endif
L2(m8);}
#endif
