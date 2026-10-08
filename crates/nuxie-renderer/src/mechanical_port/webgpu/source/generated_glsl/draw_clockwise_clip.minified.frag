#ifdef FRAGMENT
V1
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
C0(U2,n0);
#endif
q1(i3,m0);
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
cc(w6,G4);
#endif
q1(d7,Z0);W1 Y1(IB){q(j2,D);d z1=-j2.x;
#ifdef DRAW_INTERIOR_TRIANGLES
q(o1,d);d B0=o1;
#else
q(S,Q2);d B0=S.x;
#endif
O2;D X0;d d6,O3;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(BORROWED_COVERAGE_PASS){O3=B0;}else
#endif
{X0=unpackHalf2x16(l1(m0));d6=X0.y;d m5=d6==z1?X0.x:J0(.0);O3=m5+B0;}
#ifdef ENABLE_NESTED_CLIPPING
d a6=j2.y;if(ENABLE_NESTED_CLIPPING&&a6!=.0){d K4=.0;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(BORROWED_COVERAGE_PASS){X0=unpackHalf2x16(l1(m0));d6=X0.y;}
#endif
if(d6!=z1){K4=d6==a6?X0.x:.0;m1(Z0,packHalf2x16(R2(K4,qh)));}else{K4=unpackHalf2x16(l1(Z0)).x;h2(Z0);}O3=min(O3,K4);}else
#endif
{h2(Z0);}m1(m0,packHalf2x16(R2(O3,z1)));
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
N2(n0);
#endif
P2;p2;}
#endif
