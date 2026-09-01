#ifdef FRAGMENT
I1
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
w0(S2,j0);
#endif
j1(T2,h0);
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
Sa(f6,k4);
#endif
j1(I6,P0);J1 L1(JB){r(U1,E);c k1=-U1.x;
#ifdef DRAW_INTERIOR_TRIANGLES
r(i1,c);c r0=i1;
#else
r(O,z2);c r0=O.x;
#endif
w2;E N0;c K5,v3;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(BORROWED_COVERAGE_PASS){v3=r0;}else
#endif
{N0=unpackHalf2x16(Y0(h0));K5=N0.y;c Q4=K5==k1?N0.x:G0(.0);v3=Q4+r0;}
#ifdef ENABLE_NESTED_CLIPPING
c H5=U1.y;if(ENABLE_NESTED_CLIPPING&&H5!=.0){c o4=.0;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(BORROWED_COVERAGE_PASS){N0=unpackHalf2x16(Y0(h0));K5=N0.y;}
#endif
if(K5!=k1){o4=K5==H5?N0.x:.0;c1(P0,packHalf2x16(B2(o4,Gf)));}else{o4=unpackHalf2x16(Y0(P0)).x;d2(P0);}v3=min(v3,o4);}else
#endif
{d2(P0);}c1(h0,packHalf2x16(B2(v3,k1)));
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
v2(j0);
#endif
x2;Y1;}
#endif
