#ifdef FRAGMENT
#if(defined(FIXED_FUNCTION_COLOR_OUTPUT)&&!defined(ENABLE_CLIPPING))||defined(RENDER_MODE_CLOCKWISE_ATOMIC)
#undef Pc
#else
#define Pc
#endif
E1
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
z0(x2,n0);
#endif
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
q1(k3,m0);
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
z0(A6,I4);
#endif
q1(l7,Y0);
#else
z0(k3,m0);
#endif
F1
#ifdef DRAW_IMAGE_MESH
V3 q3(B5,v4,TB);W3 C5 w4(U4) D5 k4 l4
#endif
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
#ifdef DRAW_IMAGE_MESH
H2(IB)
#else
H2(IB)
#endif
#else
#ifdef DRAW_IMAGE_MESH
X1(IB)
#else
X1(IB)
#endif
#endif
{
#ifdef FEATHER_ATLAS_BLIT
q(P0,f);
#if defined(ENABLE_MODULATED_IMAGE)
q(U0,M);
#endif
q(T2,c);
#endif
#ifdef ENABLE_CLIPPING
q(f4,d);
#endif
#ifdef ENABLE_CLIP_RECT
q(V0,f);
#endif
#if defined(FEATHER_ATLAS_BLIT)&&defined(ENABLE_ADVANCED_BLEND)
q(Q0,d);
#endif
#ifdef DRAW_IMAGE_MESH
q(d6,c);q(V1,i);
#ifdef ENABLE_ADVANCED_BLEND
q(L1,P);
#endif
#endif
#ifdef FEATHER_ATLAS_BLIT
i k=x8(
#ifdef ENABLE_MODULATED_IMAGE
U0,
#endif
#ifdef ENABLE_ADVANCED_BLEND
W2(Q0),
#endif
P0 m3);d n=clamp(n2(JD,Sa,T2,.0).x,J0(.0),J0(1.));
#endif
#ifdef DRAW_IMAGE_MESH
i k=h8(TB,U4,d6,j.Ge);d n=1.;
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){d y5=max(B3(V4(V0)),J0(.0));n=min(y5,n);}
#endif
#ifdef Pc
O2;
#endif
#if defined(ENABLE_CLIPPING)
if(ENABLE_CLIPPING&&f4!=.0){d P3;
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
D W0=unpackHalf2x16(k1(m0));d h7=W0.y;P3=max(h7==f4?W0.x:J0(.0),J0(.0));
#else
P3=O0(m0).x;
#endif
P3=max(P3,J0(.0));n=min(n,P3);}
#endif
#ifdef DRAW_IMAGE_MESH
k*=V1;
#endif
#if!defined(FIXED_FUNCTION_COLOR_OUTPUT)
i a1=O0(n0);
#ifdef ENABLE_ADVANCED_BLEND
#ifdef FEATHER_ATLAS_BLIT
P W1=W2(Q0);
#endif
#ifdef DRAW_IMAGE_MESH
P W1=L1;
#endif
if(ENABLE_ADVANCED_BLEND&&W1!=U3){
#ifdef DRAW_IMAGE_MESH
k.xyz=i6(k);
#endif
k.xyz=N4(k.xyz,a1,W1)*k.w;}
#endif
k*=n;k.xyz=J2(k.xyz,k.w,d0.xy,j.F3,j.G3);
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
k=a1*(1.-k.w)+k;
#endif
x0(n0,k);
#endif
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
g2(m0);g2(Y0);
#else
x0(m0,H0(.0));
#endif
#ifdef Pc
P2;
#endif
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
k=(k*n);k.xyz=J2(k.xyz,k.w,d0.xy,j.F3,j.G3);N1=k;E3
#else
o2;
#endif
}
#endif
