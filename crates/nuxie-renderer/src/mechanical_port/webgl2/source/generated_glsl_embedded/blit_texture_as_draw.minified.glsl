n2
#ifdef JD
H0 W(0,c,o2);
#endif
N1
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
Og(B5,z4,JC);
#else
r3(B5,z4,JC);
#endif
Z3
#ifdef JD
C5 A4(Pg) D5
#endif
M2(i,VE){i e9;
#ifdef JD
q(o2,c);e9=e5(JC,Pg,o2,.0);
#elif defined(XD)
e9=(f9(JC,0,f0(floor(c0.xy)))+f9(JC,1,f0(floor(c0.xy)))+f9(JC,2,f0(floor(c0.xy)))+f9(JC,3,f0(floor(c0.xy))))*0.25;
#else
e9=v1(JC,f0(floor(c0.xy)));
#endif
N2(e9);}
#endif
