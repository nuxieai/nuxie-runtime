l2
#ifdef HD
E0 W(0,c,e2);
#endif
d2
#ifdef BB
j4 k4 P4 Q4 c1(d0) d1 r1(MF,d0,D,G,r){c z2;z2.x=(G&1)==0?-1.:1.;z2.y=(G&2)==0?-1.:1.;
#ifdef HD
T(e2,c);e2.x=z2.x*.5+.5;e2.y=z2.y*-.5+.5;Z(e2);
#endif
e I=e(z2,0,1);v1(I);}
#endif
#ifdef EB
O3
#ifdef TD
Uf(r5,l4,JC);
#else
i3(r5,l4,JC);
#endif
P3
#ifdef HD
v5 m4(Vf) w5
#endif
j3(i,QE){i B8;
#ifdef HD
q(e2,c);B8=f7(JC,Vf,e2,.0);
#elif defined(TD)
B8=(C8(JC,0,e0(floor(f0.xy)))+C8(JC,1,e0(floor(f0.xy)))+C8(JC,2,e0(floor(f0.xy)))+C8(JC,3,e0(floor(f0.xy))))*0.25;
#else
B8=p1(JC,e0(floor(f0.xy)));
#endif
Q2(B8);}
#endif
