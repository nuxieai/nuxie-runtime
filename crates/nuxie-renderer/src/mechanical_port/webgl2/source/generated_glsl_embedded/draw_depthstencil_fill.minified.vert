#ifdef BB
c1(d0) d1
#endif
l2 E0 W(0,e,a1);
#ifdef A
KB W(4,C,l1);
#endif
#ifdef O
KB W(6,d,Q0);
#endif
#ifdef GB
E0 W(9,P,F1);
#endif
d2
#ifdef BB
r1(RB,d0,D,p3,E6){T(a1,e);
#ifdef A
T(l1,C);
#endif
#ifdef O
T(Q0,d);
#endif
#ifdef GB
T(F1,P);
#endif
bool z9=(p3&xg)!=0;bool Ki=(p3&wg)!=0;int Ke=p3&((1<<Ja)-1);int Le=ld(z9);int Li=Ke>>Le;int Me=Ke&((1<<Le)-1);int Ne=z9?int(sg):int(kd);bool Pb=!z9&&Me==vg;int T3=Pb?0:Me;int E5=min(T3,Ne-1);int U3=Li*Ne+E5;N D2=p1(TB,q4(U3));uint i0=D2.w;uint w6=max(i0&Ma,1u);N F5=p0(AD,w6-1u);c R8=uintBitsToFloat(F5.xy);uint a0=F5.z&0xffffu;uint S8=F5.w;Y S0=n1(uintBitsToFloat(p0(LB,a0*4u)));N V3=p0(LB,a0*4u+1u);c m2=uintBitsToFloat(V3.xy);uint B7=i0&R2;if(B7!=0u&&!z9&&!Pb){T3=T3-1;}if(T3!=E5){int T8=U3+T3-E5;N C7=p1(TB,q4(T8));if((C7.w&(R2|0xffffu))!=(i0&(R2|0xffffu))){D2=p1(TB,q4(int(S8)));}else{D2=C7;}i0=(D2.w&~R2)|B7;}c V8=Pb?R8:uintBitsToFloat(D2.xy);c k0=K0(S0,V8)+m2;O0 L0=k5(XC,a0);uint n2=L0.x&0xfu;
#ifdef A
if(A){uint Qb=(n2==n5?L0.y:L0.x)>>16;d X0=k6(Qb,j.T4);if(n2==n5) X0=-X0;l1.x=X0;}
#endif
#ifdef O
if(O){Q0=float((L0.x>>4)&0xfu);}
#endif
c l0=k0;
#ifdef QD
if(j.W9!=0u){l0.y=float(j.X9)-l0.y;}
#endif
#ifdef AB
if(AB){Y C3=n1(p0(JB,a0*f2+2u));e Q3=p0(JB,a0*f2+3u);Ga(C3,Q3.xy,l0 Y4);}
#endif
if(n2==fa){a1=e(unpackUnorm4x8(L0.y));}
#ifdef A
else if(A&&n2==n5){d E4=k6(L0.x>>16,j.T4);l1.y=E4;}
#endif
else{Y Rb=n1(p0(JB,a0*f2));e W7=p0(JB,a0*f2+1u);a1=Y9(l0,Rb,W7.xy,float(n2),W7.zw,uintBitsToFloat(L0.y));a1.w=-a1.w;}if(Ki){a1=e(.0,.0,.0,.0);}
#ifdef GB
if(GB&&(L0.x&vd)!=0u){Y Sb=n1(p0(JB,a0*f2+4u));e X7=p0(JB,a0*f2+5u);c o3=K0(Sb,l0)+X7.xy;F1=P(o3.x,o3.y,1.+X7.z);}else{F1=P(0.0,0.0,0.0);}
#endif
e I=I3(k0);
#ifdef NC
I.y=-I.y;
#endif
N W3=p0(LB,a0*4u+2u);I.z=H8(O1(W3.x),0xffu);Z(a1);
#ifdef A
Z(l1);
#endif
#ifdef O
Z(Q0);
#endif
#ifdef GB
Z(F1);
#endif
v1(I);}
#endif
