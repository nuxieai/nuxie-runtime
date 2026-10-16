#ifdef FRAGMENT
E1
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
z0(x2,n0);
#endif
q1(k3,m0);
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
fc(A6,g7);
#endif
q1(l7,Y0);F1
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
H2(IB)
#else
X1(IB)
#endif
{q(P0,f);
#ifdef ENABLE_MODULATED_IMAGE
q(U0,M);
#endif
#ifdef DRAW_INTERIOR_TRIANGLES
q(o1,d);
#else
q(S,Q2);
#endif
q(G0,d);
#ifdef ENABLE_CLIPPING
q(i2,D);
#endif
#ifdef ENABLE_CLIP_RECT
q(V0,f);
#endif
#ifdef ENABLE_ADVANCED_BLEND
q(Q0,d);
#endif
d B0=
#ifdef DRAW_INTERIOR_TRIANGLES
o1;
#else
Hc(S);
#endif
i o0;d S1;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(!BORROWED_COVERAGE_PASS)
#endif
{o0=x8(
#ifdef ENABLE_MODULATED_IMAGE
U0,
#endif
#ifdef ENABLE_ADVANCED_BLEND
W2(Q0),
#endif
P0 m3);S1=1.;
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){d Mc=C3(V4(V0));S1=min(Mc,S1);}
#endif
}O2;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(BORROWED_COVERAGE_PASS){m1(Y0,packHalf2x16(R2(B0,G0)));
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
N2(n0);
#endif
}else
#endif
{D o5=unpackHalf2x16(k1(Y0));d ka=o5.y;d p5=ka==G0?o5.x:J0(.0);d Hf=
#ifndef DRAW_INTERIOR_TRIANGLES
o6(S)?max(p5,B0):
#endif
p5+B0;
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING&&i2.x!=.0){D W0=unpackHalf2x16(k1(m0));d g6=W0.y;d Nc=g6==i2.x?W0.x:J0(.0);S1=min(Nc,S1);}
#endif
S1=max(S1,.0);d p2=ib(p5,.0,S1);d Q1=ib(Hf,.0,S1);
#ifdef ENABLE_DITHER
d f6;if(ENABLE_DITHER){f6=lb(d0.xy,j.G3,j.H3);}
#endif
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
i a1=O0(n0);
#ifdef ENABLE_ADVANCED_BLEND
if(ENABLE_ADVANCED_BLEND&&Q0!=r6(U3)){if(Q1!=.0){if(p2==.0){o0.xyz=N4(o0.xyz,a1,W2(Q0));
#ifndef DRAW_INTERIOR_TRIANGLES
if(Q1<S1){v C8=o0.xyz;
#ifdef ENABLE_DITHER
if(ENABLE_DITHER){C8+=f6*j.He;}
#endif
x0(g7,H0(C8,0.0));}
#endif
}else{o0.xyz=O0(g7).xyz;N2(g7);}}o0.xyz*=o0.w;}
#endif
#endif
o0*=M9(p2,Q1,o0.w);
#ifdef ENABLE_DITHER
o0.xyz=J2(o0.xyz,o0.w,f6);
#endif
#ifndef DRAW_INTERIOR_TRIANGLES
#ifdef ENABLE_ADVANCED_BLEND
#define If (!ENABLE_ADVANCED_BLEND||Q0==r6(U3))&&o0.w>=1.
#else
#define If o0.w>=1.
#endif
Ue(If,Y0,packHalf2x16(R2(Hf,G0)));
#else
g2(Y0);
#endif
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
Te(o0.x+o0.y+o0.z+o0.w==.0,n0,a1*(1.-o0.w)+o0);
#endif
}g2(m0);P2;
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
N1=o0;F3
#else
o2;
#endif
}
#endif
