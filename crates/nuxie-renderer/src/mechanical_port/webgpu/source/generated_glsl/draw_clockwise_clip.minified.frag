#ifdef FRAGMENT
H1
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
y0(B2,p0);
#endif
w1(p3,n0);
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
oc(D6,S4);
#endif
w1(l7,c1);I1 D1(HB){q(l2,F);d C1=-l2.x;
#ifdef DRAW_INTERIOR_TRIANGLES
q(r1,d);d E0=r1;
#else
q(S,T2);d E0=S.x;
#endif
R2;F Z0;d k6,S3;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(BORROWED_COVERAGE_PASS){S3=E0;}else
#endif
{Z0=unpackHalf2x16(m1(n0));k6=Z0.y;d B5=k6==C1?Z0.x:A0(.0);S3=B5+E0;}
#ifdef ENABLE_NESTED_CLIPPING
d i6=l2.y;if(ENABLE_NESTED_CLIPPING&&i6!=.0){d W4=.0;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(BORROWED_COVERAGE_PASS){Z0=unpackHalf2x16(m1(n0));k6=Z0.y;}
#endif
if(k6!=C1){W4=k6==i6?Z0.x:.0;o1(c1,packHalf2x16(U2(W4,Lh)));}else{W4=unpackHalf2x16(m1(c1)).x;k2(c1);}S3=min(S3,W4);}else
#endif
{k2(c1);}o1(n0,packHalf2x16(U2(S3,C1)));
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
Q2(p0);
#endif
S2;K1;}
#endif
