#ifdef FRAGMENT
I1
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
x0(S2,j0);
#endif
j1(T2,h0);
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
Ta(h6,E6);
#endif
j1(K6,P0);J1
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
p2(JB)
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
r(V1,E);
#endif
#ifdef ENABLE_CLIP_RECT
r(M0,g);
#endif
#ifdef ENABLE_ADVANCED_BLEND
r(f2,c);
#endif
c v0=
#ifdef DRAW_INTERIOR_TRIANGLES
i1;
#else
rb(O);
#endif
i w0;c F1;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(!BORROWED_COVERAGE_PASS)
#endif
{w0=M7(f1,
#ifdef ENABLE_MODULATED_IMAGE
A2,
#endif
1. U2);F1=1.;
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){c wb=h3(c5(M0));F1=min(wb,F1);}
#endif
}x2;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(BORROWED_COVERAGE_PASS){c1(P0,packHalf2x16(B2(v0,B0)));
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
w2(j0);
#endif
}else
#endif
{E R4=unpackHalf2x16(Y0(P0));c i9=R4.y;c S4=i9==B0?R4.x:G0(.0);c ue=
#ifndef DRAW_INTERIOR_TRIANGLES
V5(O)?max(S4,v0):
#endif
S4+v0;
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING&&V1.x!=.0){E N0=unpackHalf2x16(Y0(h0));c M5=N0.y;c xb=M5==V1.x?N0.x:G0(.0);F1=min(xb,F1);}
#endif
F1=max(F1,.0);c a2=da(S4,.0,F1);c E1=da(ue,.0,F1);
#ifdef ENABLE_DITHER
c L5;if(ENABLE_DITHER){L5=ga(a0.xy,m.B3,m.C3);}
#endif
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
i K1=I0(j0);
#ifdef ENABLE_ADVANCED_BLEND
if(ENABLE_ADVANCED_BLEND){if(f2!=a6(R5)&&E1!=.0){if(a2==.0){w0.xyz=U4(w0.xyz,K1,c6(f2));
#ifndef DRAW_INTERIOR_TRIANGLES
if(E1<F1){A P7=w0.xyz;
#ifdef ENABLE_DITHER
if(ENABLE_DITHER){P7+=L5*m.vd;}
#endif
y0(E6,C0(P7,0.0));}
#endif
}else{w0.xyz=I0(E6).xyz;w2(E6);}}w0.xyz*=w0.w;}
#endif
#endif
w0*=K8(a2,E1,w0.w);
#ifdef ENABLE_DITHER
w0.xyz=F2(w0.xyz,w0.w,L5);
#endif
#ifndef DRAW_INTERIOR_TRIANGLES
#ifdef ENABLE_ADVANCED_BLEND
#define ve (!ENABLE_ADVANCED_BLEND||f2==a6(R5))&&w0.w>=1.
#else
#define ve w0.w>=1.
#endif
Ld(ve,P0,packHalf2x16(B2(ue,B0)));
#else
e2(P0);
#endif
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
Kd(w0.w==.0,j0,K1*(1.-w0.w)+w0);
#endif
}e2(h0);y2;
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
C1=w0;n3
#else
Z1;
#endif
}
#endif
