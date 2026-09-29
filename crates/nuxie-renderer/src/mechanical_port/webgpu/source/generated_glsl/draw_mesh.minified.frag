#ifdef FRAGMENT
#if(defined(FIXED_FUNCTION_COLOR_OUTPUT)&&!defined(ENABLE_CLIPPING))||defined(RENDER_MODE_CLOCKWISE_ATOMIC)
#undef zb
#else
#define zb
#endif
J1
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
x0(S2,j0);
#endif
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
j1(T2,g0);
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
x0(g6,l4);
#endif
j1(J6,P0);
#else
x0(T2,g0);
#endif
K1
#ifdef DRAW_IMAGE_MESH
D3 Z2(d5,V3,HC);E3 e5 W3(V5)f5 O3 P3
#endif
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
#ifdef DRAW_IMAGE_MESH
p2(IB)
#else
p2(IB)
#endif
#else
#ifdef DRAW_IMAGE_MESH
M1(IB)
#else
M1(IB)
#endif
#endif
{
#ifdef FEATHER_ATLAS_BLIT
r(f1,g);
#if defined(ENABLE_MODULATED_IMAGE)
r(A2,Q);
#endif
r(D2,d);
#endif
#ifdef ENABLE_CLIPPING
r(J3,c);
#endif
#ifdef ENABLE_CLIP_RECT
r(M0,g);
#endif
#if defined(FEATHER_ATLAS_BLIT)&&defined(ENABLE_ADVANCED_BLEND)
r(f2,c);
#endif
#ifdef DRAW_IMAGE_MESH
r(G5,d);r(I1,c);
#ifdef ENABLE_ADVANCED_BLEND
r(B1,K);
#endif
#endif
#ifdef FEATHER_ATLAS_BLIT
i j=M7(f1,
#ifdef ENABLE_MODULATED_IMAGE
A2,
#endif
1. U2);c n=clamp(o2(BD,O9,D2,.0).x,G0(.0),G0(1.));
#endif
#ifdef DRAW_IMAGE_MESH
i j=A7(HC,V5,G5,m.ud);c n=1.;
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){c Y4=max(h3(c5(M0)),G0(.0));n=min(Y4,n);}
#endif
#ifdef zb
x2;
#endif
#if defined(ENABLE_CLIPPING)
if(ENABLE_CLIPPING&&J3!=.0){c r3;
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
E N0=unpackHalf2x16(Y0(g0));c E6=N0.y;r3=max(E6==J3?N0.x:G0(.0),G0(.0));
#else
r3=I0(g0).x;
#endif
r3=max(r3,G0(.0));n=min(n,r3);}
#endif
#ifdef DRAW_IMAGE_MESH
n*=I1;
#endif
#if!defined(FIXED_FUNCTION_COLOR_OUTPUT)
i L1=I0(j0);
#ifdef ENABLE_ADVANCED_BLEND
if(ENABLE_ADVANCED_BLEND){
#ifdef FEATHER_ATLAS_BLIT
K S3=a6(f2);
#endif
#ifdef DRAW_IMAGE_MESH
j.xyz=F6(j);K S3=B1;
#endif
if(S3!=Q5){j.xyz=U4(j.xyz,L1,S3);}j.w*=n;j.xyz*=j.w;}else
#endif
{j*=n;}
#ifdef NEEDS_GAMMA_CORRECTION
if(NEEDS_GAMMA_CORRECTION){j=l3(j);}
#endif
j.xyz=F2(j.xyz,j.w,a0.xy,m.A3,m.B3);
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
j=L1*(1.-j.w)+j;
#endif
y0(j0,j);
#endif
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
e2(g0);e2(P0);
#else
y0(g0,C0(.0));
#endif
#ifdef zb
y2;
#endif
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
j=(j*n);j.xyz=F2(j.xyz,j.w,a0.xy,m.A3,m.B3);D1=j;m3
#else
Z1;
#endif
}
#endif
