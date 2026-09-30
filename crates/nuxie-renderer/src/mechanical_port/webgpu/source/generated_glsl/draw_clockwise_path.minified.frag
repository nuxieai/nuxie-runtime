#ifdef FRAGMENT
L1
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
z0(G2,l0);
#endif
k1(V2,i0);
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
bb(i6,F6);
#endif
k1(L6,R0);M1
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
v2(HB)
#else
O1(HB)
#endif
{r(X1,f);
#ifdef ENABLE_MODULATED_IMAGE
r(C2,R);
#endif
#ifdef DRAW_INTERIOR_TRIANGLES
r(j1,d);
#else
r(M,B2);
#endif
r(D0,d);
#ifdef ENABLE_CLIPPING
r(Y1,E);
#endif
#ifdef ENABLE_CLIP_RECT
r(N0,f);
#endif
#ifdef ENABLE_ADVANCED_BLEND
r(g1,d);
#endif
d y0=
#ifdef DRAW_INTERIOR_TRIANGLES
j1;
#else
wb(M);
#endif
i q0;d H1;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(!BORROWED_COVERAGE_PASS)
#endif
{q0=N7(
#ifdef ENABLE_MODULATED_IMAGE
C2,
#endif
#ifdef ENABLE_ADVANCED_BLEND
e3(g1),
#endif
X1 W2);H1=1.;
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){d Bb=k3(f5(N0));H1=min(Bb,H1);}
#endif
}z2;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(BORROWED_COVERAGE_PASS){d1(R0,packHalf2x16(D2(y0,D0)));
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
y2(l0);
#endif
}else
#endif
{E U4=unpackHalf2x16(Z0(R0));d m9=U4.y;d V4=m9==D0?U4.x:J0(.0);d He=
#ifndef DRAW_INTERIOR_TRIANGLES
X5(M)?max(V4,y0):
#endif
V4+y0;
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING&&Y1.x!=.0){E P0=unpackHalf2x16(Z0(i0));d N5=P0.y;d Cb=N5==Y1.x?P0.x:J0(.0);H1=min(Cb,H1);}
#endif
H1=max(H1,.0);d e2=ja(V4,.0,H1);d G1=ja(He,.0,H1);
#ifdef ENABLE_DITHER
d M5;if(ENABLE_DITHER){M5=ma(d0.xy,j.F3,j.G3);}
#endif
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
i N1=K0(l0);
#ifdef ENABLE_ADVANCED_BLEND
if(ENABLE_ADVANCED_BLEND&&g1!=d6(A4)){if(G1!=.0){if(e2==.0){q0.xyz=X4(q0.xyz,N1,e3(g1));
#ifndef DRAW_INTERIOR_TRIANGLES
if(G1<H1){A S7=q0.xyz;
#ifdef ENABLE_DITHER
if(ENABLE_DITHER){S7+=M5*j.Gd;}
#endif
A0(F6,E0(S7,0.0));}
#endif
}else{q0.xyz=K0(F6).xyz;y2(F6);}}q0.xyz*=q0.w;}
#endif
#endif
q0*=N8(e2,G1,q0.w);
#ifdef ENABLE_DITHER
q0.xyz=J2(q0.xyz,q0.w,M5);
#endif
#ifndef DRAW_INTERIOR_TRIANGLES
#ifdef ENABLE_ADVANCED_BLEND
#define Ie (!ENABLE_ADVANCED_BLEND||g1==d6(A4))&&q0.w>=1.
#else
#define Ie q0.w>=1.
#endif
Wd(Ie,R0,packHalf2x16(D2(He,D0)));
#else
h2(R0);
#endif
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
Vd(q0.w==.0,l0,N1*(1.-q0.w)+q0);
#endif
}h2(i0);A2;
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
E1=q0;p3
#else
d2;
#endif
}
#endif
