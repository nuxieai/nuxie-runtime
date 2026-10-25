m2
#ifdef KD
F0 W(0,c,n2);
#endif
M1
#ifdef BB
v4 w4 a5 c5 e1(e0) f1 o1(QF,e0,A,C,r){c F2;F2.x=(C&1)==0?-1.:1.;F2.y=(C&2)==0?-1.:1.;
#ifdef KD
V(n2,c);n2.x=F2.x*.5+.5;n2.y=F2.y*-.5+.5;a0(n2);
#endif
e I=e(F2,0,1);A1(I);}
#endif
#ifdef EB
X3
#ifdef YD
Ig(C5,x4,JC);
#else
q3(C5,x4,JC);
#endif
Y3
#ifdef KD
D5 y4(Jg) E5
#endif
L2(i,WE){i e9;
#ifdef KD
q(n2,c);e9=F5(JC,Jg,n2,.0);
#elif defined(YD)
e9=(f9(JC,0,g0(floor(c0.xy)))+f9(JC,1,g0(floor(c0.xy)))+f9(JC,2,g0(floor(c0.xy)))+f9(JC,3,g0(floor(c0.xy))))*0.25;
#else
e9=v1(JC,g0(floor(c0.xy)));
#endif
M2(e9);}
#endif
