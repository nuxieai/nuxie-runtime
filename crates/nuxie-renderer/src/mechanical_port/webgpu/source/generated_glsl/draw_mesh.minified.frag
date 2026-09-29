#ifdef FRAGMENT
#if(defined(FIXED_FUNCTION_COLOR_OUTPUT)&&!defined(ENABLE_CLIPPING))||defined(RENDER_MODE_CLOCKWISE_ATOMIC)
#undef Ab
#else
#define Ab
#endif
J1
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
x0(S2,j0);
#endif
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
j1(T2,h0);
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
x0(h6,l4);
#endif
j1(L6,P0);
#else
x0(T2,h0);
#endif
K1
#ifdef DRAW_IMAGE_MESH
E3 Z2(e5,W3,JC);F3 f5 X3(W5)g5 Q3 R3
#endif
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
#ifdef DRAW_IMAGE_MESH
p2(JB)
#else
p2(JB)
#endif
#else
#ifdef DRAW_IMAGE_MESH
M1(JB)
#else
M1(JB)
#endif
#endif
{
#ifdef FEATHER_ATLAS_BLIT
r(f1,g);
#if defined(ENABLE_MODULATED_IMAGE)
r(A2,R);
#endif
r(D2,d);
#endif
#ifdef ENABLE_CLIPPING
r(K3,c);
#endif
#ifdef ENABLE_CLIP_RECT
r(M0,g);
#endif
#if defined(FEATHER_ATLAS_BLIT)&&defined(ENABLE_ADVANCED_BLEND)
r(f2,c);
#endif
#ifdef DRAW_IMAGE_MESH
r(H5,d);r(I1,c);
#ifdef ENABLE_ADVANCED_BLEND
r(B1,N);
#endif
#endif
#ifdef FEATHER_ATLAS_BLIT
i j=N7(f1,
#ifdef ENABLE_MODULATED_IMAGE
A2,
#endif
1. U2);c n=clamp(o2(CD,R9,D2,.0).x,G0(.0),G0(1.));
#endif
#ifdef DRAW_IMAGE_MESH
i j=B7(JC,W5,H5,m.ud);c n=1.;
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){c Z4=max(h3(d5(M0)),G0(.0));n=min(Z4,n);}
#endif
#ifdef Ab
x2;
#endif
#if defined(ENABLE_CLIPPING)
if(ENABLE_CLIPPING&&K3!=.0){c v3;
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
E N0=unpackHalf2x16(Y0(h0));c G6=N0.y;v3=max(G6==K3?N0.x:G0(.0),G0(.0));
#else
v3=I0(h0).x;
#endif
v3=max(v3,G0(.0));n=min(n,v3);}
#endif
#ifdef DRAW_IMAGE_MESH
n*=I1;
#endif
#if!defined(FIXED_FUNCTION_COLOR_OUTPUT)
i L1=I0(j0);
#ifdef ENABLE_ADVANCED_BLEND
if(ENABLE_ADVANCED_BLEND){
#ifdef FEATHER_ATLAS_BLIT
N T3=c6(f2);
#endif
#ifdef DRAW_IMAGE_MESH
j.xyz=H6(j);N T3=B1;
#endif
if(T3!=R5){j.xyz=V4(j.xyz,L1,T3);}j.w*=n;j.xyz*=j.w;}else
#endif
{j*=n;}
#ifdef NEEDS_GAMMA_CORRECTION
if(NEEDS_GAMMA_CORRECTION){j=m3(j);}
#endif
j.xyz=F2(j.xyz,j.w,a0.xy,m.B3,m.C3);
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
j=L1*(1.-j.w)+j;
#endif
y0(j0,j);
#endif
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
e2(h0);e2(P0);
#else
y0(h0,C0(.0));
#endif
#ifdef Ab
y2;
#endif
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
j=(j*n);j.xyz=F2(j.xyz,j.w,a0.xy,m.B3,m.C3);D1=j;n3
#else
Z1;
#endif
}
#endif
