#ifdef FRAGMENT
#if(defined(FIXED_FUNCTION_COLOR_OUTPUT)&&!defined(ENABLE_CLIPPING))||defined(RENDER_MODE_CLOCKWISE_ATOMIC)
#undef bd
#else
#define bd
#endif
H1
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
y0(B2,p0);
#endif
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
w1(p3,n0);
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
y0(D6,S4);
#endif
w1(l7,c1);
#else
y0(p3,n0);
#endif
I1
#ifdef DRAW_IMAGE_MESH
Y3 Y2(I5,C4,SB);Z3 J5 D4(x4) K5 n4 p4
#endif
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
#ifdef DRAW_IMAGE_MESH
K2(HB)
#else
K2(HB)
#endif
#else
#ifdef DRAW_IMAGE_MESH
D1(HB)
#else
D1(HB)
#endif
#endif
{
#ifdef FEATHER_ATLAS_BLIT
q(N0,e);
#if defined(ENABLE_MODULATED_IMAGE)
q(O0,M);
#endif
q(W2,c);
#endif
#ifdef ENABLE_CLIPPING
q(j4,d);
#endif
#ifdef ENABLE_CLIP_RECT
q(Y0,e);
#endif
#if defined(FEATHER_ATLAS_BLIT)&&defined(ENABLE_ADVANCED_BLEND)
q(S0,d);
#endif
#ifdef DRAW_IMAGE_MESH
q(h6,c);q(a2,i);
#ifdef ENABLE_ADVANCED_BLEND
q(Q1,O);
#endif
#endif
#ifdef FEATHER_ATLAS_BLIT
i k=C8(
#ifdef ENABLE_MODULATED_IMAGE
O0,
#endif
#ifdef ENABLE_ADVANCED_BLEND
Z2(S0),
#endif
N0 r3);d n=clamp(q2(ID,Xa,W2,.0).x,A0(.0),A0(1.));
#endif
#ifdef DRAW_IMAGE_MESH
i k=i8(SB,x4,h6,j.We);d n=1.;
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){d F5=max(F3(y4(Y0)),A0(.0));n=min(F5,n);}
#endif
#ifdef bd
R2;
#endif
#if defined(ENABLE_CLIPPING)
if(ENABLE_CLIPPING&&j4!=.0){d S3;
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
F Z0=unpackHalf2x16(m1(n0));d h7=Z0.y;S3=max(h7==j4?Z0.x:A0(.0),A0(.0));
#else
S3=L0(n0).x;
#endif
S3=max(S3,A0(.0));n=min(n,S3);}
#endif
#ifdef DRAW_IMAGE_MESH
k*=a2;
#endif
#if!defined(FIXED_FUNCTION_COLOR_OUTPUT)
i X0=L0(p0);
#ifdef ENABLE_ADVANCED_BLEND
#ifdef FEATHER_ATLAS_BLIT
O c2=Z2(S0);
#endif
#ifdef DRAW_IMAGE_MESH
O c2=Q1;
#endif
if(ENABLE_ADVANCED_BLEND&&c2!=X3){
#ifdef DRAW_IMAGE_MESH
k.xyz=m6(k);
#endif
k.xyz=X4(k.xyz,X0,c2)*k.w;}
#endif
k*=n;k.xyz=M2(k.xyz,k.w,Y.xy,j.K3,j.L3);
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
k=X0*(1.-k.w)+k;
#endif
x0(p0,k);
#endif
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
k2(n0);k2(c1);
#else
x0(n0,z0(.0));
#endif
#ifdef bd
S2;
#endif
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
k=(k*n);k.xyz=M2(k.xyz,k.w,Y.xy,j.K3,j.L3);S1=k;J3
#else
K1;
#endif
}
#endif
