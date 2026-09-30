#ifdef FRAGMENT
J1
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
y0(F2,k0);
#endif
i1(U2,h0);
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
ab(g6,m4);
#endif
i1(K6,Q0);K1 M1(IB){r(W1,E);d j1=-W1.x;
#ifdef DRAW_INTERIOR_TRIANGLES
r(h1,d);d x0=h1;
#else
r(M,A2);d x0=M.x;
#endif
y2;E O0;d J5,w3;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(BORROWED_COVERAGE_PASS){w3=x0;}else
#endif
{O0=unpackHalf2x16(Y0(h0));J5=O0.y;d S4=J5==j1?O0.x:I0(.0);w3=S4+x0;}
#ifdef ENABLE_NESTED_CLIPPING
d H5=W1.y;if(ENABLE_NESTED_CLIPPING&&H5!=.0){d q4=.0;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(BORROWED_COVERAGE_PASS){O0=unpackHalf2x16(Y0(h0));J5=O0.y;}
#endif
if(J5!=j1){q4=J5==H5?O0.x:.0;c1(Q0,packHalf2x16(C2(q4,dg)));}else{q4=unpackHalf2x16(Y0(Q0)).x;f2(Q0);}w3=min(w3,q4);}else
#endif
{f2(Q0);}c1(h0,packHalf2x16(C2(w3,j1)));
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
x2(k0);
#endif
z2;a2;}
#endif
