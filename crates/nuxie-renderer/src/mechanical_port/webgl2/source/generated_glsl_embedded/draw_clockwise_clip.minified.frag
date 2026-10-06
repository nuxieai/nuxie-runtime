#ifdef FB
S1
#ifndef W
B0(K2,n0);
#endif
o1(c3,m0);
#ifndef W
tb(o6,C4);
#endif
o1(U6,V0);T1 U1(IB){q(l1,C);d X0=-l1.x;
#ifdef DB
q(m1,d);d A0=m1;
#else
q(S,G2);d A0=S.x;
#endif
E2;C T0;d X5,G3;
#if defined(DB)&&defined(EC)
if(EC){G3=A0;}else
#endif
{T0=unpackHalf2x16(h1(m0));X5=T0.y;d f5=X5==X0?T0.x:H0(.0);G3=f5+A0;}
#ifdef AD
d F4=l1.y;if(AD&&F4!=.0){d H4=.0;
#if defined(DB)&&defined(EC)
if(EC){T0=unpackHalf2x16(h1(m0));X5=T0.y;}
#endif
if(X5!=X0){H4=X5==F4?T0.x:.0;j1(V0,packHalf2x16(H2(H4,Jg)));}else{H4=unpackHalf2x16(h1(V0)).x;a2(V0);}G3=min(G3,H4);}else
#endif
{a2(V0);}j1(m0,packHalf2x16(H2(G3,X0)));
#ifndef W
D2(n0);
#endif
F2;h2;}
#endif
