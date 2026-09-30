#ifdef FRAGMENT
R1
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
B0(L2,n0);
#endif
o1(d3,m0);
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
tb(p6,C4);
#endif
o1(V6,W0);S1 T1(IB){q(l1,C);d X0=-l1.x;
#ifdef DRAW_INTERIOR_TRIANGLES
q(m1,d);d A0=m1;
#else
q(S,H2);d A0=S.x;
#endif
F2;C U0;d Y5,G3;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(BORROWED_COVERAGE_PASS){G3=A0;}else
#endif
{U0=unpackHalf2x16(h1(m0));Y5=U0.y;d g5=Y5==X0?U0.x:I0(.0);G3=g5+A0;}
#ifdef ENABLE_NESTED_CLIPPING
d F4=l1.y;if(ENABLE_NESTED_CLIPPING&&F4!=.0){d H4=.0;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(BORROWED_COVERAGE_PASS){U0=unpackHalf2x16(h1(m0));Y5=U0.y;}
#endif
if(Y5!=X0){H4=Y5==F4?U0.x:.0;j1(W0,packHalf2x16(I2(H4,Ig)));}else{H4=unpackHalf2x16(h1(W0)).x;Z1(W0);}G3=min(G3,H4);}else
#endif
{Z1(W0);}j1(m0,packHalf2x16(I2(G3,X0)));
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
E2(n0);
#endif
G2;h2;}
#endif
