#ifdef FRAGMENT
E1
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
z0(x2,n0);
#endif
q1(k3,m0);
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
fc(A6,I4);
#endif
q1(l7,Y0);F1 X1(IB){q(i2,D);d A1=-i2.x;
#ifdef DRAW_INTERIOR_TRIANGLES
q(o1,d);d B0=o1;
#else
q(S,Q2);d B0=S.x;
#endif
O2;D W0;d g6,P3;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(BORROWED_COVERAGE_PASS){P3=B0;}else
#endif
{W0=unpackHalf2x16(k1(m0));g6=W0.y;d p5=g6==A1?W0.x:J0(.0);P3=p5+B0;}
#ifdef ENABLE_NESTED_CLIPPING
d e6=i2.y;if(ENABLE_NESTED_CLIPPING&&e6!=.0){d M4=.0;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(BORROWED_COVERAGE_PASS){W0=unpackHalf2x16(k1(m0));g6=W0.y;}
#endif
if(g6!=A1){M4=g6==e6?W0.x:.0;m1(Y0,packHalf2x16(R2(M4,uh)));}else{M4=unpackHalf2x16(k1(Y0)).x;g2(Y0);}P3=min(P3,M4);}else
#endif
{g2(Y0);}m1(m0,packHalf2x16(R2(P3,A1)));
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
N2(n0);
#endif
P2;o2;}
#endif
