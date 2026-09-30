#ifdef FRAGMENT
J1
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
y0(F2,k0);
#endif
i1(U2,h0);
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
ab(g6,E6);
#endif
i1(K6,Q0);K1
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
r2(IB)
#else
M1(IB)
#endif
{r(V1,f);
#ifdef ENABLE_MODULATED_IMAGE
r(B2,Q);
#endif
#ifdef DRAW_INTERIOR_TRIANGLES
r(h1,d);
#else
r(M,A2);
#endif
r(C0,d);
#ifdef ENABLE_CLIPPING
r(W1,E);
#endif
#ifdef ENABLE_CLIP_RECT
r(M0,f);
#endif
#ifdef ENABLE_ADVANCED_BLEND
r(g2,d);
#endif
d x0=
#ifdef DRAW_INTERIOR_TRIANGLES
h1;
#else
vb(M);
#endif
i q0;d F1;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(!BORROWED_COVERAGE_PASS)
#endif
{q0=M7(V1,
#ifdef ENABLE_MODULATED_IMAGE
B2,
#endif
1. V2);F1=1.;
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){d Ab=i3(a5(M0));F1=min(Ab,F1);}
#endif
}y2;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(BORROWED_COVERAGE_PASS){c1(Q0,packHalf2x16(C2(x0,C0)));
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
x2(k0);
#endif
}else
#endif
{E R4=unpackHalf2x16(Y0(Q0));d l9=R4.y;d S4=l9==C0?R4.x:I0(.0);d Ge=
#ifndef DRAW_INTERIOR_TRIANGLES
U5(M)?max(S4,x0):
#endif
S4+x0;
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING&&W1.x!=.0){E O0=unpackHalf2x16(Y0(h0));d J5=O0.y;d Bb=J5==W1.x?O0.x:I0(.0);F1=min(Bb,F1);}
#endif
F1=max(F1,.0);d c2=ia(S4,.0,F1);d E1=ia(Ge,.0,F1);
#ifdef ENABLE_DITHER
d I5;if(ENABLE_DITHER){I5=la(c0.xy,l.C3,l.D3);}
#endif
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
i L1=J0(k0);
#ifdef ENABLE_ADVANCED_BLEND
if(ENABLE_ADVANCED_BLEND){if(g2!=Z5(Q5)&&E1!=.0){if(c2==.0){q0.xyz=U4(q0.xyz,L1,a6(g2));
#ifndef DRAW_INTERIOR_TRIANGLES
if(E1<F1){A Q7=q0.xyz;
#ifdef ENABLE_DITHER
if(ENABLE_DITHER){Q7+=I5*l.Fd;}
#endif
z0(E6,D0(Q7,0.0));}
#endif
}else{q0.xyz=J0(E6).xyz;x2(E6);}}q0.xyz*=q0.w;}
#endif
#endif
q0*=L8(c2,E1,q0.w);
#ifdef ENABLE_DITHER
q0.xyz=I2(q0.xyz,q0.w,I5);
#endif
#ifndef DRAW_INTERIOR_TRIANGLES
#ifdef ENABLE_ADVANCED_BLEND
#define He (!ENABLE_ADVANCED_BLEND||g2==Z5(Q5))&&q0.w>=1.
#else
#define He q0.w>=1.
#endif
Vd(He,Q0,packHalf2x16(C2(Ge,C0)));
#else
f2(Q0);
#endif
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
Ud(q0.w==.0,k0,L1*(1.-q0.w)+q0);
#endif
}f2(h0);z2;
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
C1=q0;m3
#else
a2;
#endif
}
#endif
