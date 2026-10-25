#ifdef FRAGMENT
F1
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
y0(x2,n0);
#endif
r1(k3,m0);
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
gc(E6,K4);
#endif
r1(m7,Y0);G1 I1(HB){q(k2,E);d B1=-k2.x;
#ifdef DRAW_INTERIOR_TRIANGLES
q(p1,d);d B0=p1;
#else
q(T,R2);d B0=T.x;
#endif
P2;E X0;d j6,R3;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(BORROWED_COVERAGE_PASS){R3=B0;}else
#endif
{X0=unpackHalf2x16(k1(m0));j6=X0.y;d q5=j6==B1?X0.x:J0(.0);R3=q5+B0;}
#ifdef ENABLE_NESTED_CLIPPING
d h6=k2.y;if(ENABLE_NESTED_CLIPPING&&h6!=.0){d O4=.0;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(BORROWED_COVERAGE_PASS){X0=unpackHalf2x16(k1(m0));j6=X0.y;}
#endif
if(j6!=B1){O4=j6==h6?X0.x:.0;m1(Y0,packHalf2x16(S2(O4,xh)));}else{O4=unpackHalf2x16(k1(Y0)).x;i2(Y0);}R3=min(R3,O4);}else
#endif
{i2(Y0);}m1(m0,packHalf2x16(S2(R3,B1)));
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
O2(n0);
#endif
Q2;O1;}
#endif
