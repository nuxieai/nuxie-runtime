#ifdef FRAGMENT
I1
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
x0(S2,j0);
#endif
j1(T2,h0);
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
Ta(g6,k4);
#endif
j1(J6,P0);J1 L1(JB){r(V1,E);c k1=-V1.x;
#ifdef DRAW_INTERIOR_TRIANGLES
r(i1,c);c v0=i1;
#else
r(O,z2);c v0=O.x;
#endif
x2;E N0;c L5,v3;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(BORROWED_COVERAGE_PASS){v3=v0;}else
#endif
{N0=unpackHalf2x16(Y0(h0));L5=N0.y;c R4=L5==k1?N0.x:G0(.0);v3=R4+v0;}
#ifdef ENABLE_NESTED_CLIPPING
c I5=V1.y;if(ENABLE_NESTED_CLIPPING&&I5!=.0){c o4=.0;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(BORROWED_COVERAGE_PASS){N0=unpackHalf2x16(Y0(h0));L5=N0.y;}
#endif
if(L5!=k1){o4=L5==I5?N0.x:.0;c1(P0,packHalf2x16(B2(o4,Hf)));}else{o4=unpackHalf2x16(Y0(P0)).x;e2(P0);}v3=min(v3,o4);}else
#endif
{e2(P0);}c1(h0,packHalf2x16(B2(v3,k1)));
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
w2(j0);
#endif
y2;Z1;}
#endif
