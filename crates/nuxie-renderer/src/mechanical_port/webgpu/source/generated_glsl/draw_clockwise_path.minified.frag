#ifdef FRAGMENT
J1
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
y0(G2,k0);
#endif
i1(V2,h0);
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
Za(i6,F6);
#endif
i1(L6,Q0);K1
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
r2(IB)
#else
M1(IB)
#endif
{r(V1,f);
#ifdef ENABLE_MODULATED_IMAGE
r(C2,Q);
#endif
#ifdef DRAW_INTERIOR_TRIANGLES
r(h1,d);
#else
r(M,B2);
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
d w0=
#ifdef DRAW_INTERIOR_TRIANGLES
h1;
#else
ub(M);
#endif
i x0;d F1;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(!BORROWED_COVERAGE_PASS)
#endif
{x0=M7(V1,
#ifdef ENABLE_MODULATED_IMAGE
C2,
#endif
1. W2);F1=1.;
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){d zb=i3(c5(M0));F1=min(zb,F1);}
#endif
}z2;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(BORROWED_COVERAGE_PASS){c1(Q0,packHalf2x16(D2(w0,C0)));
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
y2(k0);
#endif
}else
#endif
{E R4=unpackHalf2x16(Y0(Q0));d k9=R4.y;d S4=k9==C0?R4.x:I0(.0);d Ee=
#ifndef DRAW_INTERIOR_TRIANGLES
W5(M)?max(S4,w0):
#endif
S4+w0;
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING&&W1.x!=.0){E O0=unpackHalf2x16(Y0(h0));d L5=O0.y;d Ab=L5==W1.x?O0.x:I0(.0);F1=min(Ab,F1);}
#endif
F1=max(F1,.0);d c2=ha(S4,.0,F1);d E1=ha(Ee,.0,F1);
#ifdef ENABLE_DITHER
d K5;if(ENABLE_DITHER){K5=ka(c0.xy,n.C3,n.D3);}
#endif
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
i L1=J0(k0);
#ifdef ENABLE_ADVANCED_BLEND
if(ENABLE_ADVANCED_BLEND){if(g2!=c6(S5)&&E1!=.0){if(c2==.0){x0.xyz=U4(x0.xyz,L1,d6(g2));
#ifndef DRAW_INTERIOR_TRIANGLES
if(E1<F1){A Q7=x0.xyz;
#ifdef ENABLE_DITHER
if(ENABLE_DITHER){Q7+=K5*n.Dd;}
#endif
z0(F6,D0(Q7,0.0));}
#endif
}else{x0.xyz=J0(F6).xyz;y2(F6);}}x0.xyz*=x0.w;}
#endif
#endif
x0*=L8(c2,E1,x0.w);
#ifdef ENABLE_DITHER
x0.xyz=J2(x0.xyz,x0.w,K5);
#endif
#ifndef DRAW_INTERIOR_TRIANGLES
#ifdef ENABLE_ADVANCED_BLEND
#define Fe (!ENABLE_ADVANCED_BLEND||g2==c6(S5))&&x0.w>=1.
#else
#define Fe x0.w>=1.
#endif
Td(Fe,Q0,packHalf2x16(D2(Ee,C0)));
#else
f2(Q0);
#endif
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
Sd(x0.w==.0,k0,L1*(1.-x0.w)+x0);
#endif
}f2(h0);A2;
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
C1=x0;m3
#else
a2;
#endif
}
#endif
