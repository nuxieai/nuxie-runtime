#ifdef FRAGMENT
J1
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
y0(G2,k0);
#endif
i1(V2,h0);
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
Za(i6,m4);
#endif
i1(L6,Q0);K1 M1(IB){r(W1,E);d j1=-W1.x;
#ifdef DRAW_INTERIOR_TRIANGLES
r(h1,d);d w0=h1;
#else
r(M,B2);d w0=M.x;
#endif
z2;E O0;d L5,w3;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(BORROWED_COVERAGE_PASS){w3=w0;}else
#endif
{O0=unpackHalf2x16(Y0(h0));L5=O0.y;d S4=L5==j1?O0.x:I0(.0);w3=S4+w0;}
#ifdef ENABLE_NESTED_CLIPPING
d J5=W1.y;if(ENABLE_NESTED_CLIPPING&&J5!=.0){d q4=.0;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(BORROWED_COVERAGE_PASS){O0=unpackHalf2x16(Y0(h0));L5=O0.y;}
#endif
if(L5!=j1){q4=L5==J5?O0.x:.0;c1(Q0,packHalf2x16(D2(q4,cg)));}else{q4=unpackHalf2x16(Y0(Q0)).x;f2(Q0);}w3=min(w3,q4);}else
#endif
{f2(Q0);}c1(h0,packHalf2x16(D2(w3,j1)));
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
y2(k0);
#endif
A2;a2;}
#endif
