#ifdef FRAGMENT
#if(defined(FIXED_FUNCTION_COLOR_OUTPUT)&&!defined(ENABLE_CLIPPING))||defined(RENDER_MODE_CLOCKWISE_ATOMIC)
#undef Xc
#else
#define Xc
#endif
F1
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
y0(z2,o0);
#endif
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
q1(l3,m0);
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
y0(B6,M4);
#endif
q1(j7,X0);
#else
y0(l3,m0);
#endif
G1
#ifdef DRAW_IMAGE_MESH
Y3 r3(B5,z4,RB);Z3 C5 A4(v4) D5 l4 n4
#endif
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
#ifdef DRAW_IMAGE_MESH
I2(HB)
#else
I2(HB)
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
q(M0,e);
#if defined(ENABLE_MODULATED_IMAGE)
q(N0,M);
#endif
q(W2,c);
#endif
#ifdef ENABLE_CLIPPING
q(h4,d);
#endif
#ifdef ENABLE_CLIP_RECT
q(V0,e);
#endif
#if defined(FEATHER_ATLAS_BLIT)&&defined(ENABLE_ADVANCED_BLEND)
q(R0,d);
#endif
#ifdef DRAW_IMAGE_MESH
q(e6,c);q(Y1,i);
#ifdef ENABLE_ADVANCED_BLEND
q(N1,O);
#endif
#endif
#ifdef FEATHER_ATLAS_BLIT
i k=x8(
#ifdef ENABLE_MODULATED_IMAGE
N0,
#endif
#ifdef ENABLE_ADVANCED_BLEND
Y2(R0),
#endif
M0 n3);d n=clamp(q2(ID,Ua,W2,.0).x,C0(.0),C0(1.));
#endif
#ifdef DRAW_IMAGE_MESH
i k=g8(RB,v4,e6,j.Re);d n=1.;
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){d y5=max(D3(Y4(V0)),C0(.0));n=min(y5,n);}
#endif
#ifdef Xc
R2;
#endif
#if defined(ENABLE_CLIPPING)
if(ENABLE_CLIPPING&&h4!=.0){d S3;
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
E W0=unpackHalf2x16(j1(m0));d f7=W0.y;S3=max(f7==h4?W0.x:C0(.0),C0(.0));
#else
S3=K0(m0).x;
#endif
S3=max(S3,C0(.0));n=min(n,S3);}
#endif
#ifdef DRAW_IMAGE_MESH
k*=Y1;
#endif
#if!defined(FIXED_FUNCTION_COLOR_OUTPUT)
i Z0=K0(o0);
#ifdef ENABLE_ADVANCED_BLEND
#ifdef FEATHER_ATLAS_BLIT
O Z1=Y2(R0);
#endif
#ifdef DRAW_IMAGE_MESH
O Z1=N1;
#endif
if(ENABLE_ADVANCED_BLEND&&Z1!=X3){
#ifdef DRAW_IMAGE_MESH
k.xyz=j6(k);
#endif
k.xyz=R4(k.xyz,Z0,Z1)*k.w;}
#endif
k*=n;k.xyz=K2(k.xyz,k.w,c0.xy,j.I3,j.J3);
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
k=Z0*(1.-k.w)+k;
#endif
x0(o0,k);
#endif
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
j2(m0);j2(X0);
#else
x0(m0,E0(.0));
#endif
#ifdef Xc
S2;
#endif
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
k=(k*n);k.xyz=K2(k.xyz,k.w,c0.xy,j.I3,j.J3);Q1=k;H3
#else
O1;
#endif
}
#endif
