m2
#ifdef KD
H0 W(0,c,n2);
#endif
M1
#ifdef BB
x4 y4 c5 d5 e1(e0) f1 o1(QF,e0,A,C,r){c F2;F2.x=(C&1)==0?-1.:1.;F2.y=(C&2)==0?-1.:1.;
#ifdef KD
V(n2,c);n2.x=F2.x*.5+.5;n2.y=F2.y*-.5+.5;a0(n2);
#endif
e I=e(F2,0,1);A1(I);}
#endif
#ifdef DB
Y3
#ifdef YD
Og(D5,z4,JC);
#else
q3(D5,z4,JC);
#endif
Z3
#ifdef KD
E5 A4(Pg) F5
#endif
L2(i,WE){i f9;
#ifdef KD
q(n2,c);f9=e5(JC,Pg,n2,.0);
#elif defined(YD)
f9=(g9(JC,0,g0(floor(c0.xy)))+g9(JC,1,g0(floor(c0.xy)))+g9(JC,2,g0(floor(c0.xy)))+g9(JC,3,g0(floor(c0.xy))))*0.25;
#else
f9=v1(JC,g0(floor(c0.xy)));
#endif
M2(f9);}
#endif
