#ifdef EB
E1
#ifndef U
z0(x2,n0);
#endif
q1(k3,m0);
#ifndef U
fc(A6,I4);
#endif
q1(l7,Y0);F1 X1(IB){q(i2,D);d A1=-i2.x;
#ifdef DB
q(o1,d);d B0=o1;
#else
q(S,Q2);d B0=S.x;
#endif
O2;D W0;d g6,P3;
#if defined(DB)&&defined(EC)
if(EC){P3=B0;}else
#endif
{W0=unpackHalf2x16(k1(m0));g6=W0.y;d p5=g6==A1?W0.x:J0(.0);P3=p5+B0;}
#ifdef ED
d e6=i2.y;if(ED&&e6!=.0){d M4=.0;
#if defined(DB)&&defined(EC)
if(EC){W0=unpackHalf2x16(k1(m0));g6=W0.y;}
#endif
if(g6!=A1){M4=g6==e6?W0.x:.0;m1(Y0,packHalf2x16(R2(M4,uh)));}else{M4=unpackHalf2x16(k1(Y0)).x;g2(Y0);}P3=min(P3,M4);}else
#endif
{g2(Y0);}m1(m0,packHalf2x16(R2(P3,A1)));
#ifndef U
N2(n0);
#endif
P2;o2;}
#endif
