#ifdef FRAGMENT
#if(defined(FIXED_FUNCTION_COLOR_OUTPUT)&&!defined(ENABLE_CLIPPING))||defined(RENDER_MODE_CLOCKWISE_ATOMIC)
#undef xb
#else
#define xb
#endif
I1
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
w0(S2,j0);
#endif
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
j1(T2,h0);
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
w0(f6,k4);
#endif
j1(I6,P0);
#else
w0(T2,h0);
#endif
J1
#ifdef DRAW_IMAGE_MESH
E3 Z2(a5,W3,JC);F3 c5 X3(U5)d5 Q3 R3
#endif
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
#ifdef DRAW_IMAGE_MESH
o2(JB)
#else
o2(JB)
#endif
#else
#ifdef DRAW_IMAGE_MESH
L1(JB)
#else
L1(JB)
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
r(e2,c);
#endif
#ifdef DRAW_IMAGE_MESH
r(F5,d);r(H1,c);
#ifdef ENABLE_ADVANCED_BLEND
r(A1,N);
#endif
#endif
#ifdef FEATHER_ATLAS_BLIT
i j=L7(f1,
#ifdef ENABLE_MODULATED_IMAGE
A2,
#endif
1. U2);c n=clamp(n2(CD,Q9,D2,.0).x,G0(.0),G0(1.));
#endif
#ifdef DRAW_IMAGE_MESH
i j=z7(JC,U5,F5,m.sd);c n=1.;
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){c W4=max(h3(Z4(M0)),G0(.0));n=min(W4,n);}
#endif
#ifdef xb
w2;
#endif
#if defined(ENABLE_CLIPPING)
if(ENABLE_CLIPPING&&K3!=.0){c v3;
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
E N0=unpackHalf2x16(Y0(h0));c D6=N0.y;v3=max(D6==K3?N0.x:G0(.0),G0(.0));
#else
v3=I0(h0).x;
#endif
v3=max(v3,G0(.0));n=min(n,v3);}
#endif
#ifdef DRAW_IMAGE_MESH
n*=H1;
#endif
#if!defined(FIXED_FUNCTION_COLOR_OUTPUT)
i K1=I0(j0);
#ifdef ENABLE_ADVANCED_BLEND
if(ENABLE_ADVANCED_BLEND){
#ifdef FEATHER_ATLAS_BLIT
N T3=Z5(e2);
#endif
#ifdef DRAW_IMAGE_MESH
j.xyz=E6(j);N T3=A1;
#endif
if(T3!=P5){j.xyz=S4(j.xyz,K1,T3);}j.w*=n;j.xyz*=j.w;}else
#endif
{j*=n;}
#ifdef NEEDS_GAMMA_CORRECTION
if(NEEDS_GAMMA_CORRECTION){j=m3(j);}
#endif
j.xyz=F2(j.xyz,j.w,Z.xy,m.B3,m.C3);
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
j=K1*(1.-j.w)+j;
#endif
x0(j0,j);
#endif
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
d2(h0);d2(P0);
#else
x0(h0,C0(.0));
#endif
#ifdef xb
x2;
#endif
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
j=(j*n);j.xyz=F2(j.xyz,j.w,Z.xy,m.B3,m.C3);C1=j;n3
#else
Y1;
#endif
}
#endif
