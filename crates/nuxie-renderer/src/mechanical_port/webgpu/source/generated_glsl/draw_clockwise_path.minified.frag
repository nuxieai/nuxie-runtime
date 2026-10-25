#ifdef FRAGMENT
F1
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
y0(x2,n0);
#endif
r1(k3,m0);
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
gc(E6,h7);
#endif
r1(m7,Y0);G1
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
H2(HB)
#else
I1(HB)
#endif
{q(P0,e);
#ifdef ENABLE_MODULATED_IMAGE
q(V0,M);
#endif
#ifdef DRAW_INTERIOR_TRIANGLES
q(p1,d);
#else
q(T,R2);
#endif
q(G0,d);
#ifdef ENABLE_CLIPPING
q(k2,E);
#endif
#ifdef ENABLE_CLIP_RECT
q(W0,e);
#endif
#ifdef ENABLE_ADVANCED_BLEND
q(Q0,d);
#endif
d B0=
#ifdef DRAW_INTERIOR_TRIANGLES
p1;
#else
Lc(T);
#endif
i p0;d V1;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(!BORROWED_COVERAGE_PASS)
#endif
{p0=x8(
#ifdef ENABLE_MODULATED_IMAGE
V0,
#endif
#ifdef ENABLE_ADVANCED_BLEND
W2(Q0),
#endif
P0 m3);V1=1.;
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){d Qc=C3(X4(W0));V1=min(Qc,V1);}
#endif
}P2;
#if defined(DRAW_INTERIOR_TRIANGLES)&&defined(BORROWED_COVERAGE_PASS)
if(BORROWED_COVERAGE_PASS){m1(Y0,packHalf2x16(S2(B0,G0)));
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
O2(n0);
#endif
}else
#endif
{E p5=unpackHalf2x16(k1(Y0));d ka=p5.y;d q5=ka==G0?p5.x:J0(.0);d Kf=
#ifndef DRAW_INTERIOR_TRIANGLES
r6(T)?max(q5,B0):
#endif
q5+B0;
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING&&k2.x!=.0){E X0=unpackHalf2x16(k1(m0));d j6=X0.y;d Rc=j6==k2.x?X0.x:J0(.0);V1=min(Rc,V1);}
#endif
V1=max(V1,.0);d q2=ib(q5,.0,V1);d T1=ib(Kf,.0,V1);
#ifdef ENABLE_DITHER
d i6;if(ENABLE_DITHER){i6=lb(c0.xy,j.G3,j.H3);}
#endif
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
i a1=K0(n0);
#ifdef ENABLE_ADVANCED_BLEND
if(ENABLE_ADVANCED_BLEND&&Q0!=x6(W3)){if(T1!=.0){if(q2==.0){p0.xyz=P4(p0.xyz,a1,W2(Q0));
#ifndef DRAW_INTERIOR_TRIANGLES
if(T1<V1){v C8=p0.xyz;
#ifdef ENABLE_DITHER
if(ENABLE_DITHER){C8+=i6*j.Ke;}
#endif
x0(h7,H0(C8,0.0));}
#endif
}else{p0.xyz=K0(h7).xyz;O2(h7);}}p0.xyz*=p0.w;}
#endif
#endif
p0*=M9(q2,T1,p0.w);
#ifdef ENABLE_DITHER
p0.xyz=J2(p0.xyz,p0.w,i6);
#endif
#ifndef DRAW_INTERIOR_TRIANGLES
#ifdef ENABLE_ADVANCED_BLEND
#define Lf (!ENABLE_ADVANCED_BLEND||Q0==x6(W3))&&p0.w>=1.
#else
#define Lf p0.w>=1.
#endif
Xe(Lf,Y0,packHalf2x16(S2(Kf,G0)));
#else
i2(Y0);
#endif
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
We(p0.x+p0.y+p0.z+p0.w==.0,n0,a1*(1.-p0.w)+p0);
#endif
}i2(m0);Q2;
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
Q1=p0;F3
#else
O1;
#endif
}
#endif
