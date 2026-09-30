q2
#ifdef GD
I0 W(0,c,c2);
#endif
i2
#ifdef CB
Y3 Z3 F4 G4 h1(h0) i1 B1(LF,h0,F,A,q){c r2;r2.x=(A&1)==0?-1.:1.;r2.y=(A&2)==0?-1.:1.;
#ifdef GD
V(c2,c);c2.x=r2.x*.5+.5;c2.y=r2.y*-.5+.5;c0(c2);
#endif
f X=f(r2,0,1);C1(X);}
#endif
#ifdef EB
I3
#ifdef TD
Df(h5,a4,IC);
#else
e3(h5,a4,IC);
#endif
J3
#ifdef GD
i5 c4(Ef) j5
#endif
f3(i,RE){i l8;
#ifdef GD
r(c2,c);l8=U6(IC,Ef,c2,.0);
#elif defined(TD)
l8=(m8(IC,0,Y(floor(d0.xy)))+m8(IC,1,Y(floor(d0.xy)))+m8(IC,2,Y(floor(d0.xy)))+m8(IC,3,Y(floor(d0.xy))))*0.25;
#else
l8=v1(IC,Y(floor(d0.xy)));
#endif
M2(l8);}
#endif
