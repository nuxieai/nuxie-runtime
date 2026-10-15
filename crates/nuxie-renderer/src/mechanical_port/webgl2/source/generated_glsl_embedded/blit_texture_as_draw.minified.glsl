v2
#ifdef KD
F0 W(0,c,l2);
#endif
k2
#ifdef BB
q4 r4 Y4 Z4 e1(f0) f1 y1(PF,f0,B,F,r){c F2;F2.x=(F&1)==0?-1.:1.;F2.y=(F&2)==0?-1.:1.;
#ifdef KD
V(l2,c);l2.x=F2.x*.5+.5;l2.y=F2.y*-.5+.5;Z(l2);
#endif
f I=f(F2,0,1);z1(I);}
#endif
#ifdef EB
V3
#ifdef XD
Fg(B5,v4,IC);
#else
q3(B5,v4,IC);
#endif
W3
#ifdef KD
C5 w4(Gg) D5
#endif
V2(i,TE){i d9;
#ifdef KD
q(l2,c);d9=E5(IC,Gg,l2,.0);
#elif defined(XD)
d9=(e9(IC,0,g0(floor(d0.xy)))+e9(IC,1,g0(floor(d0.xy)))+e9(IC,2,g0(floor(d0.xy)))+e9(IC,3,g0(floor(d0.xy))))*0.25;
#else
d9=r1(IC,g0(floor(d0.xy)));
#endif
L2(d9);}
#endif
