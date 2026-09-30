#ifdef FRAGMENT
M1
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
z0(G2,m0);
#endif
k1(X2,i0);
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
db(j6,p4);
#endif
k1(M6,S0);N1 P1(HB){r(Y1,E);d m1=-Y1.x;
#ifdef DRAW_INTERIOR_TRIANGLES
r(j1,d);d y0=j1;
#else
r(O,B2);d y0=O.x;
#endif
z2;E Q0;d O5,z3;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(BORROWED_COVERAGE_PASS){z3=y0;}else
#endif
{Q0=unpackHalf2x16(a1(i0));O5=Q0.y;d X4=O5==m1?Q0.x:J0(.0);z3=X4+y0;}
#ifdef ENABLE_NESTED_CLIPPING
d M5=Y1.y;if(ENABLE_NESTED_CLIPPING&&M5!=.0){d w4=.0;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(BORROWED_COVERAGE_PASS){Q0=unpackHalf2x16(a1(i0));O5=Q0.y;}
#endif
if(O5!=m1){w4=O5==M5?Q0.x:.0;d1(S0,packHalf2x16(D2(w4,kg)));}else{w4=unpackHalf2x16(a1(S0)).x;h2(S0);}z3=min(z3,w4);}else
#endif
{h2(S0);}d1(i0,packHalf2x16(D2(z3,m1)));
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
y2(m0);
#endif
A2;d2;}
#endif
