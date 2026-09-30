#ifdef FRAGMENT
Q1
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
A0(L2,o0);
#endif
o1(d3,m0);
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
sb(n6,N6);
#endif
o1(T6,V0);R1
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
A2(IB)
#else
T1(IB)
#endif
{q(a1,e);
#ifdef ENABLE_MODULATED_IMAGE
q(F1,P);
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
q(R0,e);
#endif
#ifdef ENABLE_ADVANCED_BLEND
q(Q0,d);
#endif
d z0=
#ifdef DRAW_INTERIOR_TRIANGLES
m1;
#else
Ub(S);
#endif
i n0;d M1;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(!BORROWED_COVERAGE_PASS)
#endif
{n0=Y7(
#ifdef ENABLE_MODULATED_IMAGE
F1,
#endif
#ifdef ENABLE_ADVANCED_BLEND
k3(Q0),
#endif
a1 e3);M1=1.;
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){d Zb=v3(q5(R0));M1=min(Zb,M1);}
#endif
}F2;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(BORROWED_COVERAGE_PASS){j1(V0,packHalf2x16(I2(z0,F0)));
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
E2(o0);
#endif
}else
#endif
{C d5=unpackHalf2x16(h1(V0));d C9=d5.y;d f5=C9==F0?d5.x:M0(.0);d Xe=
#ifndef DRAW_INTERIOR_TRIANGLES
e6(S)?max(f5,z0):
#endif
f5+z0;
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING&&l1.x!=.0){C T0=unpackHalf2x16(h1(m0));d V5=T0.y;d ac=V5==l1.x?T0.x:M0(.0);M1=min(ac,M1);}
#endif
M1=max(M1,.0);d h2=Aa(f5,.0,M1);d L1=Aa(Xe,.0,M1);
#ifdef ENABLE_DITHER
d U5;if(ENABLE_DITHER){U5=Da(f0.xy,j.M3,j.N3);}
#endif
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
i S1=N0(o0);
#ifdef ENABLE_ADVANCED_BLEND
if(ENABLE_ADVANCED_BLEND&&Q0!=i6(L4)){if(L1!=.0){if(h2==.0){n0.xyz=h5(n0.xyz,S1,k3(Q0));
#ifndef DRAW_INTERIOR_TRIANGLES
if(L1<M1){v d8=n0.xyz;
#ifdef ENABLE_DITHER
if(ENABLE_DITHER){d8+=U5*j.Wd;}
#endif
B0(N6,G0(d8,0.0));}
#endif
}else{n0.xyz=N0(N6).xyz;E2(N6);}}n0.xyz*=n0.w;}
#endif
#endif
n0*=a9(h2,L1,n0.w);
#ifdef ENABLE_DITHER
n0.xyz=O2(n0.xyz,n0.w,U5);
#endif
#ifndef DRAW_INTERIOR_TRIANGLES
#ifdef ENABLE_ADVANCED_BLEND
#define Ye (!ENABLE_ADVANCED_BLEND||Q0==i6(L4))&&n0.w>=1.
#else
#define Ye n0.w>=1.
#endif
je(Ye,V0,packHalf2x16(I2(Xe,F0)));
#else
k2(V0);
#endif
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
ie(n0.x+n0.y+n0.z+n0.w==.0,o0,S1*(1.-n0.w)+n0);
#endif
}k2(m0);G2;
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
J1=n0;A3
#else
g2;
#endif
}
#endif
