#ifdef FRAGMENT
F1
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
y0(z2,o0);
#endif
q1(l3,m0);
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
lc(B6,M4);
#endif
q1(j7,X0);G1 I1(HB){q(l2,E);d B1=-l2.x;
#ifdef DRAW_INTERIOR_TRIANGLES
q(o1,d);d B0=o1;
#else
q(S,T2);d B0=S.x;
#endif
R2;E W0;d h6,S3;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(BORROWED_COVERAGE_PASS){S3=B0;}else
#endif
{W0=unpackHalf2x16(j1(m0));h6=W0.y;d r5=h6==B1?W0.x:C0(.0);S3=r5+B0;}
#ifdef ENABLE_NESTED_CLIPPING
d f6=l2.y;if(ENABLE_NESTED_CLIPPING&&f6!=.0){d Q4=.0;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(BORROWED_COVERAGE_PASS){W0=unpackHalf2x16(j1(m0));h6=W0.y;}
#endif
if(h6!=B1){Q4=h6==f6?W0.x:.0;l1(X0,packHalf2x16(U2(Q4,Ch)));}else{Q4=unpackHalf2x16(j1(X0)).x;j2(X0);}S3=min(S3,Q4);}else
#endif
{j2(X0);}l1(m0,packHalf2x16(U2(S3,B1)));
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
Q2(o0);
#endif
S2;O1;}
#endif
