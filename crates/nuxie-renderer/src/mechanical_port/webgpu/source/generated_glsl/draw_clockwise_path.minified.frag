#ifdef FRAGMENT
M1
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
z0(H2,l0);
#endif
j1(Y2,i0);
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
Ya(f6,E6);
#endif
j1(K6,S0);N1
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
w2(HB)
#else
P1(HB)
#endif
{r(X1,f);
#ifdef ENABLE_MODULATED_IMAGE
r(D2,S);
#endif
#ifdef DRAW_INTERIOR_TRIANGLES
r(i1,d);
#else
r(O,C2);
#endif
r(D0,d);
#ifdef ENABLE_CLIPPING
r(Y1,D);
#endif
#ifdef ENABLE_CLIP_RECT
r(P0,f);
#endif
#ifdef ENABLE_ADVANCED_BLEND
r(f1,d);
#endif
d y0=
#ifdef DRAW_INTERIOR_TRIANGLES
i1;
#else
tb(O);
#endif
i j0;d I1;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(!BORROWED_COVERAGE_PASS)
#endif
{j0=K7(
#ifdef ENABLE_MODULATED_IMAGE
D2,
#endif
#ifdef ENABLE_ADVANCED_BLEND
g3(f1),
#endif
X1 Z2);I1=1.;
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){d yb=m3(h5(P0));I1=min(yb,I1);}
#endif
}A2;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(BORROWED_COVERAGE_PASS){d1(S0,packHalf2x16(E2(y0,D0)));
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
z2(l0);
#endif
}else
#endif
{D V4=unpackHalf2x16(a1(S0));d j9=V4.y;d X4=j9==D0?V4.x:J0(.0);d Ge=
#ifndef DRAW_INTERIOR_TRIANGLES
V5(O)?max(X4,y0):
#endif
X4+y0;
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING&&Y1.x!=.0){D Q0=unpackHalf2x16(a1(i0));d M5=Q0.y;d zb=M5==Y1.x?Q0.x:J0(.0);I1=min(zb,I1);}
#endif
I1=max(I1,.0);d e2=ga(X4,.0,I1);d H1=ga(Ge,.0,I1);
#ifdef ENABLE_DITHER
d L5;if(ENABLE_DITHER){L5=ja(e0.xy,j.F3,j.G3);}
#endif
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
i O1=K0(l0);
#ifdef ENABLE_ADVANCED_BLEND
if(ENABLE_ADVANCED_BLEND&&f1!=Z5(C4)){if(H1!=.0){if(e2==.0){j0.xyz=Z4(j0.xyz,O1,g3(f1));
#ifndef DRAW_INTERIOR_TRIANGLES
if(H1<I1){v O7=j0.xyz;
#ifdef ENABLE_DITHER
if(ENABLE_DITHER){O7+=L5*j.Ed;}
#endif
A0(E6,E0(O7,0.0));}
#endif
}else{j0.xyz=K0(E6).xyz;z2(E6);}}j0.xyz*=j0.w;}
#endif
#endif
j0*=K8(e2,H1,j0.w);
#ifdef ENABLE_DITHER
j0.xyz=L2(j0.xyz,j0.w,L5);
#endif
#ifndef DRAW_INTERIOR_TRIANGLES
#ifdef ENABLE_ADVANCED_BLEND
#define He (!ENABLE_ADVANCED_BLEND||f1==Z5(C4))&&j0.w>=1.
#else
#define He j0.w>=1.
#endif
Ud(He,S0,packHalf2x16(E2(Ge,D0)));
#else
h2(S0);
#endif
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
Td(j0.x+j0.y+j0.z+j0.w==.0,l0,O1*(1.-j0.w)+j0);
#endif
}h2(i0);B2;
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
E1=j0;r3
#else
d2;
#endif
}
#endif
