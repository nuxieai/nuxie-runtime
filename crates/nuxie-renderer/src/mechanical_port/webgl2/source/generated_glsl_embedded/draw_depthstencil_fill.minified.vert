#ifdef BB
c1(d0) d1
#endif
l2 E0 V(0,e,a1);
#ifdef A
KB V(4,C,l1);
#endif
#ifdef N
KB V(6,d,Q0);
#endif
#ifdef GB
E0 V(9,O,v1);
#endif
e2
#ifdef BB
w1(RB,d0,D,p3,F6){T(a1,e);
#ifdef A
T(l1,C);
#endif
#ifdef N
T(Q0,d);
#endif
#ifdef GB
T(v1,O);
#endif
bool B9=(p3&Cg)!=0;bool Wi=(p3&Bg)!=0;int Me=p3&((1<<Ka)-1);int Ne=md(B9);int Xi=Me>>Ne;int Oe=Me&((1<<Ne)-1);int Pe=B9?int(xg):int(ld);bool Qb=!B9&&Oe==Ag;int T3=Qb?0:Oe;int G5=min(T3,Pe-1);int U3=Xi*Pe+G5;M C2=p1(TB,r4(U3));uint i0=C2.w;uint x6=max(i0&Na,1u);M H5=p0(ZC,x6-1u);c S8=uintBitsToFloat(H5.xy);uint a0=H5.z&0xffffu;uint T8=H5.w;Y S0=n1(uintBitsToFloat(p0(LB,a0*4u)));M V3=p0(LB,a0*4u+1u);c m2=uintBitsToFloat(V3.xy);uint A7=i0&Q2;if(A7!=0u&&!B9&&!Qb){T3=T3-1;}if(T3!=G5){int U8=U3+T3-G5;M B7=p1(TB,r4(U8));if((B7.w&(Q2|0xffffu))!=(i0&(Q2|0xffffu))){C2=p1(TB,r4(int(T8)));}else{C2=B7;}i0=(C2.w&~Q2)|A7;}c W8=Qb?S8:uintBitsToFloat(C2.xy);c k0=M0(S0,W8)+m2;O0 G0=l5(WC,a0);uint n2=G0.x&0xfu;
#ifdef A
if(A){uint Rb=(n2==o5?G0.y:G0.x)>>16;d X0=l6(Rb,j.U4);if(n2==o5) X0=-X0;l1.x=X0;}
#endif
#ifdef N
if(N){Q0=float((G0.x>>4)&0xfu);}
#endif
c l0=k0;
#ifdef PD
if(j.X9!=0u){l0.y=float(j.Y9)-l0.y;}
#endif
#ifdef AB
if(AB){Y C3=n1(p0(JB,a0*g2+2u));e Q3=p0(JB,a0*g2+3u);Ha(C3,Q3.xy,l0 Z4);}
#endif
if(n2==ga){a1=e(unpackUnorm4x8(G0.y));}
#ifdef A
else if(A&&n2==o5){d F4=l6(G0.x>>16,j.U4);l1.y=F4;}
#endif
else{Y Sb=n1(p0(JB,a0*g2));e V7=p0(JB,a0*g2+1u);a1=Z9(l0,Sb,V7.xy,float(n2),V7.zw,uintBitsToFloat(G0.y));a1.w=-a1.w;}if(Wi){a1=e(.0,.0,.0,.0);}
#ifdef GB
if(GB&&(G0.x&wd)!=0u){Y Tb=n1(p0(JB,a0*g2+4u));e W7=p0(JB,a0*g2+5u);c o3=M0(Tb,l0)+W7.xy;v1=O(o3.x,o3.y,1.+W7.z);}else{v1=O(0.0,0.0,0.0);}
#endif
e I=I3(k0);
#ifdef MC
I.y=-I.y;
#endif
M W3=p0(LB,a0*4u+2u);I.z=H8(Q1(W3.x),0xffu);Z(a1);
#ifdef A
Z(l1);
#endif
#ifdef N
Z(Q0);
#endif
#ifdef GB
Z(v1);
#endif
x1(I);}
#endif
