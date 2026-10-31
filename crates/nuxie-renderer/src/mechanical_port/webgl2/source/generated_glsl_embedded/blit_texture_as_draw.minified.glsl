n2
#ifdef JD
I0 W(0,c,o2);
#endif
P1
#ifdef BB
A4 B4 h5 i5 f1(d0) g1 l1(QF,d0,A,C,r){c I2;I2.x=(C&1)==0?-1.:1.;I2.y=(C&2)==0?-1.:1.;
#ifdef JD
V(o2,c);o2.x=I2.x*.5+.5;o2.y=I2.y*-.5+.5;e0(o2);
#endif
e I=e(I2,0,1);B1(I);}
#endif
#ifdef EB
Y3
#ifdef YD
Wg(I5,C4,JC);
#else
Y2(I5,C4,JC);
#endif
Z3
#ifdef JD
J5 D4(Xg) K5
#endif
r2(i,RF){i i9;
#ifdef JD
q(o2,c);i9=j5(JC,Xg,o2,.0);
#elif defined(YD)
i9=(j9(JC,0,f0(floor(Y.xy)))+j9(JC,1,f0(floor(Y.xy)))+j9(JC,2,f0(floor(Y.xy)))+j9(JC,3,f0(floor(Y.xy))))*0.25;
#else
i9=q1(JC,f0(floor(Y.xy)));
#endif
v2(i9);}
#endif
