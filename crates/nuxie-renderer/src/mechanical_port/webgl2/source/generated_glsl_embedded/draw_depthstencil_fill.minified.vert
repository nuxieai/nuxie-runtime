#ifdef BB
c1(d0) d1
#endif
l2 E0 W(0,f,a1);
#ifdef A
KB W(4,C,l1);
#endif
#ifdef O
KB W(6,d,Q0);
#endif
#ifdef GB
E0 W(9,P,r1);
#endif
e2
#ifdef BB
v1(RB,d0,D,p3,G6){T(a1,f);
#ifdef A
T(l1,C);
#endif
#ifdef O
T(Q0,d);
#endif
#ifdef GB
T(r1,P);
#endif
bool B9=(p3&Bg)!=0;bool Ui=(p3&Ag)!=0;int Le=p3&((1<<Ka)-1);int Me=md(B9);int Vi=Le>>Me;int Ne=Le&((1<<Me)-1);int Oe=B9?int(wg):int(ld);bool Qb=!B9&&Ne==zg;int T3=Qb?0:Ne;int H5=min(T3,Oe-1);int U3=Vi*Oe+H5;N D2=p1(TB,r4(U3));uint i0=D2.w;uint y6=max(i0&Na,1u);N I5=p0(AD,y6-1u);c T8=uintBitsToFloat(I5.xy);uint a0=I5.z&0xffffu;uint U8=I5.w;Y T0=n1(uintBitsToFloat(p0(LB,a0*4u)));N V3=p0(LB,a0*4u+1u);c m2=uintBitsToFloat(V3.xy);uint C7=i0&R2;if(C7!=0u&&!B9&&!Qb){T3=T3-1;}if(T3!=H5){int V8=U3+T3-H5;N D7=p1(TB,r4(V8));if((D7.w&(R2|0xffffu))!=(i0&(R2|0xffffu))){D2=p1(TB,r4(int(U8)));}else{D2=D7;}i0=(D2.w&~R2)|C7;}c X8=Qb?T8:uintBitsToFloat(D2.xy);c k0=M0(T0,X8)+m2;O0 H0=m5(XC,a0);uint n2=H0.x&0xfu;
#ifdef A
if(A){uint Rb=(n2==p5?H0.y:H0.x)>>16;d X0=m6(Rb,j.U4);if(n2==p5) X0=-X0;l1.x=X0;}
#endif
#ifdef O
if(O){Q0=float((H0.x>>4)&0xfu);}
#endif
c l0=k0;
#ifdef QD
if(j.X9!=0u){l0.y=float(j.Y9)-l0.y;}
#endif
#ifdef AB
if(AB){Y C3=n1(p0(JB,a0*g2+2u));f Q3=p0(JB,a0*g2+3u);Ha(C3,Q3.xy,l0 Z4);}
#endif
if(n2==ga){a1=f(unpackUnorm4x8(H0.y));}
#ifdef A
else if(A&&n2==p5){d F4=m6(H0.x>>16,j.U4);l1.y=F4;}
#endif
else{Y Sb=n1(p0(JB,a0*g2));f X7=p0(JB,a0*g2+1u);a1=Z9(l0,Sb,X7.xy,float(n2),X7.zw,uintBitsToFloat(H0.y));a1.w=-a1.w;}if(Ui){a1=f(.0,.0,.0,.0);}
#ifdef GB
if(GB&&(H0.x&wd)!=0u){Y Tb=n1(p0(JB,a0*g2+4u));f Y7=p0(JB,a0*g2+5u);c o3=M0(Tb,l0)+Y7.xy;r1=P(o3.x,o3.y,1.+Y7.z);}else{r1=P(0.0,0.0,0.0);}
#endif
f I=I3(k0);
#ifdef NC
I.y=-I.y;
#endif
N W3=p0(LB,a0*4u+2u);I.z=J8(P1(W3.x),0xffu);Z(a1);
#ifdef A
Z(l1);
#endif
#ifdef O
Z(Q0);
#endif
#ifdef GB
Z(r1);
#endif
w1(I);}
#endif
