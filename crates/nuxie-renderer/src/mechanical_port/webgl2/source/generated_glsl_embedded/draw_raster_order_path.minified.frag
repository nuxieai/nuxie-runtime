#ifdef EB
M1 z0(H2,l0);j1(Y2,i0);z0(f6,q4);j1(K6,F7);N1 P1(HB){r(X1,f);
#ifdef IB
r(D2,S);
#endif
#ifdef DB
r(i1,d);
#else
r(O,C2);
#endif
r(D0,d);
#ifdef K
r(Y1,D);
#endif
#ifdef AB
r(P0,f);
#endif
#ifdef T
r(f1,d);
#endif
#if!defined(DB)
A2;
#endif
D V4=unpackHalf2x16(a1(F7));d j9=V4.y;d r0=j9==D0?V4.x:J0(.0);
#ifdef DB
r0+=i1;h2(F7);
#else
r0=xi(r0,O e1);d1(F7,packHalf2x16(E2(r0,D0)));
#endif
d o;
#ifdef HE
if(HE){o=ga(r0,J0(.0),J0(1.));}else
#endif
{o=abs(r0);
#ifdef XC
if(XC&&D0<.0){o=1.-J0(abs(fract(o*.5)*2.+-1.));}
#endif
o=min(o,J0(1.));}
#ifdef K
if(K&&Y1.x<.0){d l1=-Y1.x;
#ifdef ZC
if(ZC){d K5=Y1.y;if(K5!=.0){D Q0=unpackHalf2x16(a1(i0));d F6=Q0.y;d x4;if(F6!=l1){x4=F6==K5?Q0.x:.0;
#ifndef DB
A0(q4,E0(x4,.0,.0,.0));
#endif
}else{x4=K0(q4).x;
#ifndef DB
z2(q4);
#endif
}o=min(o,x4);}}
#endif
d1(i0,packHalf2x16(E2(o,l1)));z2(l0);}else
#endif
{
#ifdef K
if(K){d l1=Y1.x;if(l1!=.0){D Q0=unpackHalf2x16(a1(i0));d F6=Q0.y;o=(F6==l1)?min(Q0.x,o):J0(.0);}}
#endif
#ifdef AB
if(AB){d d5=m3(h5(P0));o=clamp(d5,J0(.0),o);}
#endif
i k=K7(
#ifdef IB
D2,
#endif
#ifdef T
g3(f1),
#endif
X1 Z2);i O1;if(j9!=D0){O1=K0(l0);
#ifndef DB
A0(q4,O1);
#endif
}else{O1=K0(q4);
#ifndef DB
z2(q4);
#endif
}
#ifdef T
if(T&&f1!=Z5(C4)){k.xyz=Z4(k.xyz,O1,g3(f1))*k.w;}
#endif
k*=o;
#ifdef BC
if(BC){k=q3(k);}
#endif
d j3=k.w;k+=O1*(1.-j3);k.xyz=L2(k.xyz,j3,e0.xy,j.F3,j.G3);A0(l0,k);h2(i0);}
#if!defined(DB)
B2;
#endif
d2;}
#endif
