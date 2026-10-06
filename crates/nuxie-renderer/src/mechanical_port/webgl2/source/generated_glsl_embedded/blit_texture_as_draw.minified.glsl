l2
#ifdef GD
E0 V(0,c,f2);
#endif
e2
#ifdef BB
k4 l4 Q4 R4 c1(d0) d1 w1(LF,d0,D,G,r){c y2;y2.x=(G&1)==0?-1.:1.;y2.y=(G&2)==0?-1.:1.;
#ifdef GD
T(f2,c);f2.x=y2.x*.5+.5;f2.y=y2.y*-.5+.5;Z(f2);
#endif
e I=e(y2,0,1);x1(I);}
#endif
#ifdef FB
O3
#ifdef SD
Zf(w5,m4,IC);
#else
i3(w5,m4,IC);
#endif
P3
#ifdef GD
x5 n4(ag) y5
#endif
j3(i,PE){i B8;
#ifdef GD
q(f2,c);B8=i6(IC,ag,f2,.0);
#elif defined(SD)
B8=(C8(IC,0,e0(floor(f0.xy)))+C8(IC,1,e0(floor(f0.xy)))+C8(IC,2,e0(floor(f0.xy)))+C8(IC,3,e0(floor(f0.xy))))*0.25;
#else
B8=p1(IC,e0(floor(f0.xy)));
#endif
P2(B8);}
#endif
