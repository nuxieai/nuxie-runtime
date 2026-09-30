#ifdef FRAGMENT
R1
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
B0(L2,n0);
#endif
o1(d3,m0);
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
tb(p6,P6);
#endif
o1(V6,W0);S1
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
A2(IB)
#else
T1(IB)
#endif
{q(a1,f);
#ifdef ENABLE_MODULATED_IMAGE
q(r1,P);
#endif
#ifdef DRAW_INTERIOR_TRIANGLES
q(m1,d);
#else
q(S,H2);
#endif
q(F0,d);
#ifdef ENABLE_CLIPPING
q(l1,C);
#endif
#ifdef ENABLE_CLIP_RECT
q(S0,f);
#endif
#ifdef ENABLE_ADVANCED_BLEND
q(Q0,d);
#endif
d A0=
#ifdef DRAW_INTERIOR_TRIANGLES
m1;
#else
Vb(S);
#endif
i o0;d N1;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(!BORROWED_COVERAGE_PASS)
#endif
{o0=Z7(
#ifdef ENABLE_MODULATED_IMAGE
r1,
#endif
#ifdef ENABLE_ADVANCED_BLEND
k3(Q0),
#endif
a1 e3);N1=1.;
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){d ac=v3(w5(S0));N1=min(ac,N1);}
#endif
}F2;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(BORROWED_COVERAGE_PASS){j1(W0,packHalf2x16(I2(A0,F0)));
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
E2(n0);
#endif
}else
#endif
{C e5=unpackHalf2x16(h1(W0));d E9=e5.y;d g5=E9==F0?e5.x:I0(.0);d af=
#ifndef DRAW_INTERIOR_TRIANGLES
g6(S)?max(g5,A0):
#endif
g5+A0;
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING&&l1.x!=.0){C U0=unpackHalf2x16(h1(m0));d Y5=U0.y;d bc=Y5==l1.x?U0.x:I0(.0);N1=min(bc,N1);}
#endif
N1=max(N1,.0);d i2=Ba(g5,.0,N1);d M1=Ba(af,.0,N1);
#ifdef ENABLE_DITHER
d X5;if(ENABLE_DITHER){X5=Ea(f0.xy,j.M3,j.N3);}
#endif
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
i I1=N0(n0);
#ifdef ENABLE_ADVANCED_BLEND
if(ENABLE_ADVANCED_BLEND&&Q0!=k6(M4)){if(M1!=.0){if(i2==.0){o0.xyz=i5(o0.xyz,I1,k3(Q0));
#ifndef DRAW_INTERIOR_TRIANGLES
if(M1<N1){v f8=o0.xyz;
#ifdef ENABLE_DITHER
if(ENABLE_DITHER){f8+=X5*j.Xd;}
#endif
y0(P6,G0(f8,0.0));}
#endif
}else{o0.xyz=N0(P6).xyz;E2(P6);}}o0.xyz*=o0.w;}
#endif
#endif
o0*=d9(i2,M1,o0.w);
#ifdef ENABLE_DITHER
o0.xyz=O2(o0.xyz,o0.w,X5);
#endif
#ifndef DRAW_INTERIOR_TRIANGLES
#ifdef ENABLE_ADVANCED_BLEND
#define bf (!ENABLE_ADVANCED_BLEND||Q0==k6(M4))&&o0.w>=1.
#else
#define bf o0.w>=1.
#endif
ke(bf,W0,packHalf2x16(I2(af,F0)));
#else
Z1(W0);
#endif
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
je(o0.x+o0.y+o0.z+o0.w==.0,n0,I1*(1.-o0.w)+o0);
#endif
}Z1(m0);G2;
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
K1=o0;A3
#else
h2;
#endif
}
#endif
