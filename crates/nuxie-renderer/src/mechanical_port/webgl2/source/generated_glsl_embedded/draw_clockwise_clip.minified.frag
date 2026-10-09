#ifdef EB
U1
#ifndef U
C0(T2,n0);
#endif
p1(j3,m0);
#ifndef U
ec(A6,I4);
#endif
p1(h7,Y0);V1 X1(IB){q(i2,D);d y1=-i2.x;
#ifdef DB
q(n1,d);d A0=n1;
#else
q(S,P2);d A0=S.x;
#endif
N2;D W0;d g6,P3;
#if defined(DB)&&defined(EC)
if(EC){P3=A0;}else
#endif
{W0=unpackHalf2x16(j1(m0));g6=W0.y;d o5=g6==y1?W0.x:J0(.0);P3=o5+A0;}
#ifdef CD
d e6=i2.y;if(CD&&e6!=.0){d M4=.0;
#if defined(DB)&&defined(EC)
if(EC){W0=unpackHalf2x16(j1(m0));g6=W0.y;}
#endif
if(g6!=y1){M4=g6==e6?W0.x:.0;l1(Y0,packHalf2x16(Q2(M4,rh)));}else{M4=unpackHalf2x16(j1(Y0)).x;g2(Y0);}P3=min(P3,M4);}else
#endif
{g2(Y0);}l1(m0,packHalf2x16(Q2(P3,y1)));
#ifndef U
M2(n0);
#endif
O2;o2;}
#endif
