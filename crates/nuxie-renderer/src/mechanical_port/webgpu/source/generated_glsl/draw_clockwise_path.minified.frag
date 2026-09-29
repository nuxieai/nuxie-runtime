#ifdef FRAGMENT
J1
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
x0(S2,j0);
#endif
j1(T2,g0);
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
Va(g6,D6);
#endif
j1(J6,P0);K1
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
p2(IB)
#else
M1(IB)
#endif
{r(f1,g);
#ifdef ENABLE_MODULATED_IMAGE
r(A2,Q);
#endif
#ifdef DRAW_INTERIOR_TRIANGLES
r(i1,c);
#else
r(L,z2);
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
rb(L);
#endif
i w0;c G1;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(!BORROWED_COVERAGE_PASS)
#endif
{w0=M7(f1,
#ifdef ENABLE_MODULATED_IMAGE
A2,
#endif
1. U2);G1=1.;
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){c wb=h3(c5(M0));G1=min(wb,G1);}
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
U5(L)?max(S4,v0):
#endif
S4+v0;
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING&&V1.x!=.0){E N0=unpackHalf2x16(Y0(g0));c L5=N0.y;c xb=L5==V1.x?N0.x:G0(.0);G1=min(xb,G1);}
#endif
G1=max(G1,.0);c a2=ca(S4,.0,G1);c F1=ca(ue,.0,G1);
#ifdef ENABLE_DITHER
c K5;if(ENABLE_DITHER){K5=fa(a0.xy,m.A3,m.B3);}
#endif
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
i L1=I0(j0);
#ifdef ENABLE_ADVANCED_BLEND
if(ENABLE_ADVANCED_BLEND){if(f2!=Z5(Q5)&&F1!=.0){if(a2==.0){w0.xyz=U4(w0.xyz,L1,a6(f2));
#ifndef DRAW_INTERIOR_TRIANGLES
if(F1<G1){v P7=w0.xyz;
#ifdef ENABLE_DITHER
if(ENABLE_DITHER){P7+=K5*m.vd;}
#endif
y0(D6,C0(P7,0.0));}
#endif
}else{w0.xyz=I0(D6).xyz;w2(D6);}}w0.xyz*=w0.w;}
#endif
#endif
w0*=K8(a2,F1,w0.w);
#ifdef ENABLE_DITHER
w0.xyz=F2(w0.xyz,w0.w,K5);
#endif
#ifndef DRAW_INTERIOR_TRIANGLES
#ifdef ENABLE_ADVANCED_BLEND
#define ve (!ENABLE_ADVANCED_BLEND||f2==Z5(Q5))&&w0.w>=1.
#else
#define ve w0.w>=1.
#endif
Ld(ve,P0,packHalf2x16(B2(ue,B0)));
#else
e2(P0);
#endif
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
Kd(w0.w==.0,j0,L1*(1.-w0.w)+w0);
#endif
}e2(g0);y2;
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
D1=w0;m3
#else
Z1;
#endif
}
#endif
