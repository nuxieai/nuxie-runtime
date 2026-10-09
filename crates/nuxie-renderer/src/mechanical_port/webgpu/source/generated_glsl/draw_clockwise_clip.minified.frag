#ifdef FRAGMENT
U1
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
C0(T2,n0);
#endif
p1(j3,m0);
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
ec(A6,I4);
#endif
p1(h7,Y0);V1 X1(IB){q(i2,D);d y1=-i2.x;
#ifdef DRAW_INTERIOR_TRIANGLES
q(n1,d);d A0=n1;
#else
q(S,P2);d A0=S.x;
#endif
N2;D W0;d g6,P3;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(BORROWED_COVERAGE_PASS){P3=A0;}else
#endif
{W0=unpackHalf2x16(j1(m0));g6=W0.y;d o5=g6==y1?W0.x:J0(.0);P3=o5+A0;}
#ifdef ENABLE_NESTED_CLIPPING
d e6=i2.y;if(ENABLE_NESTED_CLIPPING&&e6!=.0){d M4=.0;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(BORROWED_COVERAGE_PASS){W0=unpackHalf2x16(j1(m0));g6=W0.y;}
#endif
if(g6!=y1){M4=g6==e6?W0.x:.0;l1(Y0,packHalf2x16(Q2(M4,rh)));}else{M4=unpackHalf2x16(j1(Y0)).x;g2(Y0);}P3=min(P3,M4);}else
#endif
{g2(Y0);}l1(m0,packHalf2x16(Q2(P3,y1)));
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
M2(n0);
#endif
O2;o2;}
#endif
