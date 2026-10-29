#ifdef FRAGMENT
F1
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
y0(z2,o0);
#endif
q1(l3,m0);
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
lc(B6,e7);
#endif
q1(j7,X0);G1
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
I2(HB)
#else
I1(HB)
#endif
{q(M0,e);
#ifdef ENABLE_MODULATED_IMAGE
q(N0,M);
#endif
#ifdef DRAW_INTERIOR_TRIANGLES
q(o1,d);
#else
q(S,T2);
#endif
q(I0,d);
#ifdef ENABLE_CLIPPING
q(l2,E);
#endif
#ifdef ENABLE_CLIP_RECT
q(V0,e);
#endif
#ifdef ENABLE_ADVANCED_BLEND
q(R0,d);
#endif
d B0=
#ifdef DRAW_INTERIOR_TRIANGLES
o1;
#else
Pc(S);
#endif
i q0;d V1;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(!BORROWED_COVERAGE_PASS)
#endif
{q0=x8(
#ifdef ENABLE_MODULATED_IMAGE
N0,
#endif
#ifdef ENABLE_ADVANCED_BLEND
Y2(R0),
#endif
M0 n3);V1=1.;
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){d Uc=D3(Y4(V0));V1=min(Uc,V1);}
#endif
}R2;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(BORROWED_COVERAGE_PASS){l1(X0,packHalf2x16(U2(B0,I0)));
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
Q2(o0);
#endif
}else
#endif
{E q5=unpackHalf2x16(j1(X0));d la=q5.y;d r5=la==I0?q5.x:C0(.0);d Qf=
#ifndef DRAW_INTERIOR_TRIANGLES
o6(S)?max(r5,B0):
#endif
r5+B0;
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING&&l2.x!=.0){E W0=unpackHalf2x16(j1(m0));d h6=W0.y;d Vc=h6==l2.x?W0.x:C0(.0);V1=min(Vc,V1);}
#endif
V1=max(V1,.0);d v2=kb(r5,.0,V1);d T1=kb(Qf,.0,V1);
#ifdef ENABLE_DITHER
d g6;if(ENABLE_DITHER){g6=nb(c0.xy,j.I3,j.J3);}
#endif
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
i Z0=K0(o0);
#ifdef ENABLE_ADVANCED_BLEND
if(ENABLE_ADVANCED_BLEND&&R0!=r6(X3)){if(T1!=.0){if(v2==.0){q0.xyz=R4(q0.xyz,Z0,Y2(R0));
#ifndef DRAW_INTERIOR_TRIANGLES
if(T1<V1){v C8=q0.xyz;
#ifdef ENABLE_DITHER
if(ENABLE_DITHER){C8+=g6*j.Se;}
#endif
x0(e7,E0(C8,0.0));}
#endif
}else{q0.xyz=K0(e7).xyz;Q2(e7);}}q0.xyz*=q0.w;}
#endif
#endif
q0*=L9(v2,T1,q0.w);
#ifdef ENABLE_DITHER
q0.xyz=K2(q0.xyz,q0.w,g6);
#endif
#ifndef DRAW_INTERIOR_TRIANGLES
#ifdef ENABLE_ADVANCED_BLEND
#define Rf (!ENABLE_ADVANCED_BLEND||R0==r6(X3))&&q0.w>=1.
#else
#define Rf q0.w>=1.
#endif
ff(Rf,X0,packHalf2x16(U2(Qf,I0)));
#else
j2(X0);
#endif
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
ef(q0.x+q0.y+q0.z+q0.w==.0,o0,Z0*(1.-q0.w)+q0);
#endif
}j2(m0);S2;
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
Q1=q0;H3
#else
O1;
#endif
}
#endif
