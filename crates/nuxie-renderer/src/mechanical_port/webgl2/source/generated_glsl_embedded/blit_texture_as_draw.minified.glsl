q2
#ifdef FD
I0 W(0,c,c2);
#endif
i2
#ifdef CB
X3 Y3 E4 F4 h1(g0)i1 A1(KF,g0,F,B,v){c r2;r2.x=(B&1)==0?-1.:1.;r2.y=(B&2)==0?-1.:1.;
#ifdef FD
U(c2,c);c2.x=r2.x*.5+.5;c2.y=r2.y*-.5+.5;c0(c2);
#endif
f X=f(r2,0,1);B1(X);}
#endif
#ifdef EB
I3
#ifdef SD
Df(g5,Z3,HC);
#else
c3(g5,Z3,HC);
#endif
J3
#ifdef FD
h5 a4(Ef)i5
#endif
d3(i,QE){i o8;
#ifdef FD
r(c2,c);o8=W6(HC,Ef,c2,.0);
#elif defined(SD)
o8=(p8(HC,0,Z(floor(d0.xy)))+p8(HC,1,Z(floor(d0.xy)))+p8(HC,2,Z(floor(d0.xy)))+p8(HC,3,Z(floor(d0.xy))))*0.25;
#else
o8=r1(HC,Z(floor(d0.xy)));
#endif
L2(o8);}
#endif
