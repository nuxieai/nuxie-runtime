#ifdef GB
J1
#ifndef Q
x0(S2,j0);
#endif
j1(T2,h0);
#ifndef Q
Va(h6,l4);
#endif
j1(L6,P0);K1 M1(JB){r(V1,E);c k1=-V1.x;
#ifdef EB
r(i1,c);c v0=i1;
#else
r(O,z2);c v0=O.x;
#endif
x2;E N0;c M5,v3;
#if defined(EB)&&defined(FC)
if(FC){v3=v0;}else
#endif
{N0=unpackHalf2x16(Y0(h0));M5=N0.y;c T4=M5==k1?N0.x:G0(.0);v3=T4+v0;}
#ifdef ZC
c J5=V1.y;if(ZC&&J5!=.0){c p4=.0;
#if defined(EB)&&defined(FC)
if(FC){N0=unpackHalf2x16(Y0(h0));M5=N0.y;}
#endif
if(M5!=k1){p4=M5==J5?N0.x:.0;c1(P0,packHalf2x16(B2(p4,Lf)));}else{p4=unpackHalf2x16(Y0(P0)).x;e2(P0);}v3=min(v3,p4);}else
#endif
{e2(P0);}c1(h0,packHalf2x16(B2(v3,k1)));
#ifndef Q
w2(j0);
#endif
y2;Z1;}
#endif
