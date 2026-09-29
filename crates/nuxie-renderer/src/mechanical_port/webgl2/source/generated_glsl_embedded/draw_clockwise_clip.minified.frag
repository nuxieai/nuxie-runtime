#ifdef GB
J1
#ifndef N
x0(S2,j0);
#endif
j1(T2,g0);
#ifndef N
Va(g6,l4);
#endif
j1(J6,P0);K1 M1(IB){r(V1,E);c k1=-V1.x;
#ifdef EB
r(i1,c);c v0=i1;
#else
r(L,z2);c v0=L.x;
#endif
x2;E N0;c L5,r3;
#if defined(EB)&&defined(DC)
if(DC){r3=v0;}else
#endif
{N0=unpackHalf2x16(Y0(g0));L5=N0.y;c S4=L5==k1?N0.x:G0(.0);r3=S4+v0;}
#ifdef YC
c I5=V1.y;if(YC&&I5!=.0){c p4=.0;
#if defined(EB)&&defined(DC)
if(DC){N0=unpackHalf2x16(Y0(g0));L5=N0.y;}
#endif
if(L5!=k1){p4=L5==I5?N0.x:.0;c1(P0,packHalf2x16(B2(p4,Lf)));}else{p4=unpackHalf2x16(Y0(P0)).x;e2(P0);}r3=min(r3,p4);}else
#endif
{e2(P0);}c1(g0,packHalf2x16(B2(r3,k1)));
#ifndef N
w2(j0);
#endif
y2;Z1;}
#endif
