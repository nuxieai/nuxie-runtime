v2
#ifdef ID
F0 W(0,c,l2);
#endif
k2
#ifdef BB
q4 r4 Y4 Z4 d1(f0) e1 w1(NF,f0,B,F,r){c E2;E2.x=(F&1)==0?-1.:1.;E2.y=(F&2)==0?-1.:1.;
#ifdef ID
V(l2,c);l2.x=E2.x*.5+.5;l2.y=E2.y*-.5+.5;Z(l2);
#endif
f I=f(E2,0,1);x1(I);}
#endif
#ifdef EB
V3
#ifdef VD
Dg(A5,v4,IC);
#else
p3(A5,v4,IC);
#endif
W3
#ifdef ID
B5 w4(Eg) C5
#endif
V2(i,RE){i Z8;
#ifdef ID
q(l2,c);Z8=D5(IC,Eg,l2,.0);
#elif defined(VD)
Z8=(a9(IC,0,g0(floor(d0.xy)))+a9(IC,1,g0(floor(d0.xy)))+a9(IC,2,g0(floor(d0.xy)))+a9(IC,3,g0(floor(d0.xy))))*0.25;
#else
Z8=q1(IC,g0(floor(d0.xy)));
#endif
K2(Z8);}
#endif
