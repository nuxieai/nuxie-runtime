n2
#ifdef JD
H0 W(0,c,o2);
#endif
M1
#ifdef BB
x4 y4 c5 d5 d1(e0) e1 n1(PF,e0,A,C,r){c G2;G2.x=(C&1)==0?-1.:1.;G2.y=(C&2)==0?-1.:1.;
#ifdef JD
V(o2,c);o2.x=G2.x*.5+.5;o2.y=G2.y*-.5+.5;a0(o2);
#endif
e I=e(G2,0,1);A1(I);}
#endif
#ifdef DB
Y3
#ifdef XD
Ng(B5,z4,JC);
#else
r3(B5,z4,JC);
#endif
Z3
#ifdef JD
C5 A4(Og) D5
#endif
M2(i,VE){i d9;
#ifdef JD
q(o2,c);d9=e5(JC,Og,o2,.0);
#elif defined(XD)
d9=(e9(JC,0,f0(floor(c0.xy)))+e9(JC,1,f0(floor(c0.xy)))+e9(JC,2,f0(floor(c0.xy)))+e9(JC,3,f0(floor(c0.xy))))*0.25;
#else
d9=v1(JC,f0(floor(c0.xy)));
#endif
N2(d9);}
#endif
