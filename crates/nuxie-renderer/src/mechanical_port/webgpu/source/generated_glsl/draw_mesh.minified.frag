#ifdef FRAGMENT
#if(defined(FIXED_FUNCTION_COLOR_OUTPUT)&&!defined(ENABLE_CLIPPING))||defined(RENDER_MODE_CLOCKWISE_ATOMIC)
#undef Tc
#else
#define Tc
#endif
F1
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
y0(x2,n0);
#endif
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
r1(k3,m0);
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
y0(E6,K4);
#endif
r1(m7,Y0);
#else
y0(k3,m0);
#endif
G1
#ifdef DRAW_IMAGE_MESH
X3 q3(C5,x4,UB);Y3 D5 y4(W4) E5 m4 n4
#endif
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
#ifdef DRAW_IMAGE_MESH
H2(HB)
#else
H2(HB)
#endif
#else
#ifdef DRAW_IMAGE_MESH
I1(HB)
#else
I1(HB)
#endif
#endif
{
#ifdef FEATHER_ATLAS_BLIT
q(P0,e);
#if defined(ENABLE_MODULATED_IMAGE)
q(V0,M);
#endif
q(U2,c);
#endif
#ifdef ENABLE_CLIPPING
q(h4,d);
#endif
#ifdef ENABLE_CLIP_RECT
q(W0,e);
#endif
#if defined(FEATHER_ATLAS_BLIT)&&defined(ENABLE_ADVANCED_BLEND)
q(Q0,d);
#endif
#ifdef DRAW_IMAGE_MESH
q(g6,c);q(Y1,i);
#ifdef ENABLE_ADVANCED_BLEND
q(N1,P);
#endif
#endif
#ifdef FEATHER_ATLAS_BLIT
i k=x8(
#ifdef ENABLE_MODULATED_IMAGE
V0,
#endif
#ifdef ENABLE_ADVANCED_BLEND
W2(Q0),
#endif
P0 m3);d n=clamp(p2(JD,Sa,U2,.0).x,J0(.0),J0(1.));
#endif
#ifdef DRAW_IMAGE_MESH
i k=h8(UB,W4,g6,j.Je);d n=1.;
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){d z5=max(C3(X4(W0)),J0(.0));n=min(z5,n);}
#endif
#ifdef Tc
P2;
#endif
#if defined(ENABLE_CLIPPING)
if(ENABLE_CLIPPING&&h4!=.0){d R3;
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
E X0=unpackHalf2x16(k1(m0));d i7=X0.y;R3=max(i7==h4?X0.x:J0(.0),J0(.0));
#else
R3=K0(m0).x;
#endif
R3=max(R3,J0(.0));n=min(n,R3);}
#endif
#ifdef DRAW_IMAGE_MESH
k*=Y1;
#endif
#if!defined(FIXED_FUNCTION_COLOR_OUTPUT)
i a1=K0(n0);
#ifdef ENABLE_ADVANCED_BLEND
#ifdef FEATHER_ATLAS_BLIT
P Z1=W2(Q0);
#endif
#ifdef DRAW_IMAGE_MESH
P Z1=N1;
#endif
if(ENABLE_ADVANCED_BLEND&&Z1!=W3){
#ifdef DRAW_IMAGE_MESH
k.xyz=l6(k);
#endif
k.xyz=P4(k.xyz,a1,Z1)*k.w;}
#endif
k*=n;k.xyz=J2(k.xyz,k.w,c0.xy,j.G3,j.H3);
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
k=a1*(1.-k.w)+k;
#endif
x0(n0,k);
#endif
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
i2(m0);i2(Y0);
#else
x0(m0,H0(.0));
#endif
#ifdef Tc
Q2;
#endif
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
k=(k*n);k.xyz=J2(k.xyz,k.w,c0.xy,j.G3,j.H3);Q1=k;F3
#else
O1;
#endif
}
#endif
