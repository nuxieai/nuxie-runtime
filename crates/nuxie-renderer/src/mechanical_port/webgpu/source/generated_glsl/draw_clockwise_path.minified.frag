#ifdef FRAGMENT
H1
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
y0(B2,p0);
#endif
w1(p3,n0);
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
oc(D6,g7);
#endif
w1(l7,c1);I1
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
K2(HB)
#else
D1(HB)
#endif
{q(N0,e);
#ifdef ENABLE_MODULATED_IMAGE
q(O0,M);
#endif
#ifdef DRAW_INTERIOR_TRIANGLES
q(r1,d);
#else
q(S,T2);
#endif
q(J0,d);
#ifdef ENABLE_CLIPPING
q(l2,F);
#endif
#ifdef ENABLE_CLIP_RECT
q(Y0,e);
#endif
#ifdef ENABLE_ADVANCED_BLEND
q(S0,d);
#endif
d E0=
#ifdef DRAW_INTERIOR_TRIANGLES
r1;
#else
Tc(S);
#endif
i q0;d X1;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(!BORROWED_COVERAGE_PASS)
#endif
{q0=C8(
#ifdef ENABLE_MODULATED_IMAGE
O0,
#endif
#ifdef ENABLE_ADVANCED_BLEND
Z2(S0),
#endif
N0 r3);X1=1.;
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){d Yc=F3(y4(Y0));X1=min(Yc,X1);}
#endif
}R2;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(BORROWED_COVERAGE_PASS){o1(c1,packHalf2x16(U2(E0,J0)));
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
Q2(p0);
#endif
}else
#endif
{F A5=unpackHalf2x16(m1(c1));d oa=A5.y;d B5=oa==J0?A5.x:A0(.0);d Yf=
#ifndef DRAW_INTERIOR_TRIANGLES
r6(S)?max(B5,E0):
#endif
B5+E0;
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING&&l2.x!=.0){F Z0=unpackHalf2x16(m1(n0));d k6=Z0.y;d Zc=k6==l2.x?Z0.x:A0(.0);X1=min(Zc,X1);}
#endif
X1=max(X1,.0);d x2=nb(B5,.0,X1);d U1=nb(Yf,.0,X1);
#ifdef ENABLE_DITHER
d j6;if(ENABLE_DITHER){j6=qb(Y.xy,j.K3,j.L3);}
#endif
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
i X0=L0(p0);
#ifdef ENABLE_ADVANCED_BLEND
if(ENABLE_ADVANCED_BLEND&&S0!=x6(X3)){if(U1!=.0){if(x2==.0){q0.xyz=X4(q0.xyz,X0,Z2(S0));
#ifndef DRAW_INTERIOR_TRIANGLES
if(U1<X1){v H8=q0.xyz;
#ifdef ENABLE_DITHER
if(ENABLE_DITHER){H8+=j6*j.Xe;}
#endif
x0(g7,z0(H8,0.0));}
#endif
}else{q0.xyz=L0(g7).xyz;Q2(g7);}}q0.xyz*=q0.w;}
#endif
#endif
q0*=P9(x2,U1,q0.w);
#ifdef ENABLE_DITHER
q0.xyz=M2(q0.xyz,q0.w,j6);
#endif
#ifndef DRAW_INTERIOR_TRIANGLES
#ifdef ENABLE_ADVANCED_BLEND
#define Zf (!ENABLE_ADVANCED_BLEND||S0==x6(X3))&&q0.w>=1.
#else
#define Zf q0.w>=1.
#endif
lf(Zf,c1,packHalf2x16(U2(Yf,J0)));
#else
k2(c1);
#endif
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
kf(q0.x+q0.y+q0.z+q0.w==.0,p0,X0*(1.-q0.w)+q0);
#endif
}k2(n0);S2;
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
S1=q0;J3
#else
K1;
#endif
}
#endif
