#ifdef FRAGMENT
#if(defined(FIXED_FUNCTION_COLOR_OUTPUT)&&!defined(ENABLE_CLIPPING))||defined(RENDER_MODE_CLOCKWISE_ATOMIC)
#undef Oc
#else
#define Oc
#endif
U1
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
C0(T2,n0);
#endif
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
p1(j3,m0);
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
C0(A6,I4);
#endif
p1(h7,Y0);
#else
C0(j3,m0);
#endif
V1
#ifdef DRAW_IMAGE_MESH
V3 p3(A5,v4,TB);W3 B5 w4(U4) C5 k4 l4
#endif
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
#ifdef DRAW_IMAGE_MESH
G2(IB)
#else
G2(IB)
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
q(O0,f);
#if defined(ENABLE_MODULATED_IMAGE)
q(U0,M);
#endif
q(S2,c);
#endif
#ifdef ENABLE_CLIPPING
q(f4,d);
#endif
#ifdef ENABLE_CLIP_RECT
q(V0,f);
#endif
#if defined(FEATHER_ATLAS_BLIT)&&defined(ENABLE_ADVANCED_BLEND)
q(P0,d);
#endif
#ifdef DRAW_IMAGE_MESH
q(d6,c);q(T1,i);
#ifdef ENABLE_ADVANCED_BLEND
q(J1,P);
#endif
#endif
#ifdef FEATHER_ATLAS_BLIT
i l=r8(
#ifdef ENABLE_MODULATED_IMAGE
U0,
#endif
#ifdef ENABLE_ADVANCED_BLEND
W2(P0),
#endif
O0 l3);d n=clamp(n2(HD,Pa,S2,.0).x,J0(.0),J0(1.));
#endif
#ifdef DRAW_IMAGE_MESH
i l=e8(TB,U4,d6,j.Ee);d n=1.;
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){d x5=max(B3(V4(V0)),J0(.0));n=min(x5,n);}
#endif
#ifdef Oc
N2;
#endif
#if defined(ENABLE_CLIPPING)
if(ENABLE_CLIPPING&&f4!=.0){d P3;
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
D W0=unpackHalf2x16(j1(m0));d d7=W0.y;P3=max(d7==f4?W0.x:J0(.0),J0(.0));
#else
P3=Q0(m0).x;
#endif
P3=max(P3,J0(.0));n=min(n,P3);}
#endif
#ifdef DRAW_IMAGE_MESH
l*=T1;
#endif
#if!defined(FIXED_FUNCTION_COLOR_OUTPUT)
i z1=Q0(n0);
#ifdef ENABLE_ADVANCED_BLEND
#ifdef FEATHER_ATLAS_BLIT
P W1=W2(P0);
#endif
#ifdef DRAW_IMAGE_MESH
P W1=J1;
#endif
if(ENABLE_ADVANCED_BLEND&&W1!=U3){
#ifdef DRAW_IMAGE_MESH
l.xyz=i6(l);
#endif
l.xyz=N4(l.xyz,z1,W1)*l.w;}
#endif
l*=n;l.xyz=I2(l.xyz,l.w,d0.xy,j.F3,j.G3);
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
l=z1*(1.-l.w)+l;
#endif
y0(n0,l);
#endif
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
g2(m0);g2(Y0);
#else
y0(m0,H0(.0));
#endif
#ifdef Oc
O2;
#endif
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
l=(l*n);l.xyz=I2(l.xyz,l.w,d0.xy,j.F3,j.G3);L1=l;E3
#else
o2;
#endif
}
#endif
