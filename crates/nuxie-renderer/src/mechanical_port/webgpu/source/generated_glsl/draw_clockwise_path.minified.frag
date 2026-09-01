#ifdef FRAGMENT
I1
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
w0(S2,j0);
#endif
j1(T2,h0);
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
Sa(f6,C6);
#endif
j1(I6,P0);J1
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
o2(JB)
#else
L1(JB)
#endif
{r(f1,g);
#ifdef ENABLE_MODULATED_IMAGE
r(A2,R);
#endif
#ifdef DRAW_INTERIOR_TRIANGLES
r(i1,c);
#else
r(O,z2);
#endif
r(B0,c);
#ifdef ENABLE_CLIPPING
r(U1,E);
#endif
#ifdef ENABLE_CLIP_RECT
r(M0,g);
#endif
#ifdef ENABLE_ADVANCED_BLEND
r(e2,c);
#endif
c r0=
#ifdef DRAW_INTERIOR_TRIANGLES
i1;
#else
pb(O);
#endif
i v0;c F1;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(!BORROWED_COVERAGE_PASS)
#endif
{v0=L7(f1,
#ifdef ENABLE_MODULATED_IMAGE
A2,
#endif
1. U2);F1=1.;
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){c ub=h3(Z4(M0));F1=min(ub,F1);}
#endif
}w2;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(BORROWED_COVERAGE_PASS){c1(P0,packHalf2x16(B2(r0,B0)));
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
v2(j0);
#endif
}else
#endif
{E P4=unpackHalf2x16(Y0(P0));c i9=P4.y;c Q4=i9==B0?P4.x:G0(.0);c pe=
#ifndef DRAW_INTERIOR_TRIANGLES
T5(O)?max(Q4,r0):
#endif
Q4+r0;
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING&&U1.x!=.0){E N0=unpackHalf2x16(Y0(h0));c K5=N0.y;c vb=K5==U1.x?N0.x:G0(.0);F1=min(vb,F1);}
#endif
F1=max(F1,.0);c Z1=da(Q4,.0,F1);c E1=da(pe,.0,F1);
#ifdef ENABLE_DITHER
c J5;if(ENABLE_DITHER){J5=ga(Z.xy,m.B3,m.C3);}
#endif
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
i K1=I0(j0);
#ifdef ENABLE_ADVANCED_BLEND
if(ENABLE_ADVANCED_BLEND){if(e2!=Y5(P5)&&E1!=.0){if(Z1==.0){v0.xyz=S4(v0.xyz,K1,Z5(e2));
#ifndef DRAW_INTERIOR_TRIANGLES
if(E1<F1){A O7=v0.xyz;
#ifdef ENABLE_DITHER
if(ENABLE_DITHER){O7+=J5*m.td;}
#endif
x0(C6,C0(O7,0.0));}
#endif
}else{v0.xyz=I0(C6).xyz;v2(C6);}}v0.xyz*=v0.w;}
#endif
#endif
v0*=J8(Z1,E1,v0.w);
#ifdef ENABLE_DITHER
v0.xyz=F2(v0.xyz,v0.w,J5);
#endif
#ifndef DRAW_INTERIOR_TRIANGLES
#ifdef ENABLE_ADVANCED_BLEND
#define qe (!ENABLE_ADVANCED_BLEND||e2==Y5(P5))&&v0.w>=1.
#else
#define qe v0.w>=1.
#endif
Gd(qe,P0,packHalf2x16(B2(pe,B0)));
#else
d2(P0);
#endif
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
Fd(v0.w==.0,j0,K1*(1.-v0.w)+v0);
#endif
}d2(h0);x2;
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
C1=v0;n3
#else
Y1;
#endif
}
#endif
