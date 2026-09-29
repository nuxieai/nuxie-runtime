#ifdef FRAGMENT
I1
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
x0(S2,j0);
#endif
j1(T2,h0);
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
Ta(h6,l4);
#endif
j1(K6,P0);J1 L1(JB){r(V1,E);c k1=-V1.x;
#ifdef DRAW_INTERIOR_TRIANGLES
r(i1,c);c v0=i1;
#else
r(O,z2);c v0=O.x;
#endif
x2;E N0;c M5,v3;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(BORROWED_COVERAGE_PASS){v3=v0;}else
#endif
{N0=unpackHalf2x16(Y0(h0));M5=N0.y;c S4=M5==k1?N0.x:G0(.0);v3=S4+v0;}
#ifdef ENABLE_NESTED_CLIPPING
c J5=V1.y;if(ENABLE_NESTED_CLIPPING&&J5!=.0){c p4=.0;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(BORROWED_COVERAGE_PASS){N0=unpackHalf2x16(Y0(h0));M5=N0.y;}
#endif
if(M5!=k1){p4=M5==J5?N0.x:.0;c1(P0,packHalf2x16(B2(p4,Lf)));}else{p4=unpackHalf2x16(Y0(P0)).x;e2(P0);}v3=min(v3,p4);}else
#endif
{e2(P0);}c1(h0,packHalf2x16(B2(v3,k1)));
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
w2(j0);
#endif
y2;Z1;}
#endif
