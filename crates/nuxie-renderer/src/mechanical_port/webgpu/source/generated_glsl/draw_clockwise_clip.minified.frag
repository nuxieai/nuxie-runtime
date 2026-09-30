#ifdef FRAGMENT
M1
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
z0(H2,l0);
#endif
j1(Y2,i0);
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
Ya(f6,q4);
#endif
j1(K6,S0);N1 P1(HB){r(Y1,D);d l1=-Y1.x;
#ifdef DRAW_INTERIOR_TRIANGLES
r(i1,d);d y0=i1;
#else
r(O,C2);d y0=O.x;
#endif
A2;D Q0;d M5,z3;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(BORROWED_COVERAGE_PASS){z3=y0;}else
#endif
{Q0=unpackHalf2x16(a1(i0));M5=Q0.y;d X4=M5==l1?Q0.x:J0(.0);z3=X4+y0;}
#ifdef ENABLE_NESTED_CLIPPING
d K5=Y1.y;if(ENABLE_NESTED_CLIPPING&&K5!=.0){d x4=.0;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(BORROWED_COVERAGE_PASS){Q0=unpackHalf2x16(a1(i0));M5=Q0.y;}
#endif
if(M5!=l1){x4=M5==K5?Q0.x:.0;d1(S0,packHalf2x16(E2(x4,hg)));}else{x4=unpackHalf2x16(a1(S0)).x;h2(S0);}z3=min(z3,x4);}else
#endif
{h2(S0);}d1(i0,packHalf2x16(E2(z3,l1)));
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
z2(l0);
#endif
B2;d2;}
#endif
