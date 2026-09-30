#ifdef EB
L1
#ifndef O
z0(G2,l0);
#endif
k1(V2,i0);
#ifndef O
bb(i6,o4);
#endif
k1(L6,R0);M1 O1(HB){r(Y1,E);d l1=-Y1.x;
#ifdef DB
r(j1,d);d y0=j1;
#else
r(M,B2);d y0=M.x;
#endif
z2;E P0;d N5,z3;
#if defined(DB)&&defined(CC)
if(CC){z3=y0;}else
#endif
{P0=unpackHalf2x16(Z0(i0));N5=P0.y;d V4=N5==l1?P0.x:J0(.0);z3=V4+y0;}
#ifdef YC
d L5=Y1.y;if(YC&&L5!=.0){d v4=.0;
#if defined(DB)&&defined(CC)
if(CC){P0=unpackHalf2x16(Z0(i0));N5=P0.y;}
#endif
if(N5!=l1){v4=N5==L5?P0.x:.0;d1(R0,packHalf2x16(D2(v4,eg)));}else{v4=unpackHalf2x16(Z0(R0)).x;h2(R0);}z3=min(z3,v4);}else
#endif
{h2(R0);}d1(i0,packHalf2x16(D2(z3,l1)));
#ifndef O
y2(l0);
#endif
A2;d2;}
#endif
