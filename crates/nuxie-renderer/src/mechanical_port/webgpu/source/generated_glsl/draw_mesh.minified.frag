#ifdef FRAGMENT
#if(defined(FIXED_FUNCTION_COLOR_OUTPUT)&&!defined(ENABLE_CLIPPING))||defined(RENDER_MODE_CLOCKWISE_ATOMIC)
#undef Cb
#else
#define Cb
#endif
J1
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
y0(U2,k0);
#endif
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
i1(V2,h0);
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
y0(h6,m4);
#endif
i1(K6,Q0);
#else
y0(V2,h0);
#endif
K1
#ifdef DRAW_IMAGE_MESH
E3 c3(d5,W3,HC);F3 e5 X3(W5)f5 P3 Q3
#endif
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
#ifdef DRAW_IMAGE_MESH
r2(IB)
#else
r2(IB)
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
r(V1,f);
#if defined(ENABLE_MODULATED_IMAGE)
r(C2,Q);
#endif
r(F2,c);
#endif
#ifdef ENABLE_CLIPPING
r(K3,d);
#endif
#ifdef ENABLE_CLIP_RECT
r(M0,f);
#endif
#if defined(FEATHER_ATLAS_BLIT)&&defined(ENABLE_ADVANCED_BLEND)
r(g2,d);
#endif
#ifdef DRAW_IMAGE_MESH
r(G5,c);r(H1,i);
#ifdef ENABLE_ADVANCED_BLEND
r(A1,L);
#endif
#endif
#ifdef FEATHER_ATLAS_BLIT
i j=M7(V1,
#ifdef ENABLE_MODULATED_IMAGE
C2,
#endif
1. W2);d o=clamp(i2(FD,R9,F2,.0).x,I0(.0),I0(1.));
#endif
#ifdef DRAW_IMAGE_MESH
i j=A7(HC,W5,G5,n.Cd);d o=1.;
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){d Y4=max(i3(c5(M0)),I0(.0));o=min(Y4,o);}
#endif
#ifdef Cb
z2;
#endif
#if defined(ENABLE_CLIPPING)
if(ENABLE_CLIPPING&&K3!=.0){d v3;
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
E O0=unpackHalf2x16(Y0(h0));d F6=O0.y;v3=max(F6==K3?O0.x:I0(.0),I0(.0));
#else
v3=J0(h0).x;
#endif
v3=max(v3,I0(.0));o=min(o,v3);}
#endif
#ifdef DRAW_IMAGE_MESH
j*=H1;
#endif
#if!defined(FIXED_FUNCTION_COLOR_OUTPUT)
i L1=J0(k0);
#ifdef ENABLE_ADVANCED_BLEND
if(ENABLE_ADVANCED_BLEND){
#ifdef FEATHER_ATLAS_BLIT
L T3=c6(g2);
#endif
#ifdef DRAW_IMAGE_MESH
j.xyz=G6(j);L T3=A1;
#endif
if(T3!=R5){j.xyz=U4(j.xyz,L1,T3);}j.w*=o;j.xyz*=j.w;}else
#endif
{j*=o;}
#ifdef NEEDS_GAMMA_CORRECTION
if(NEEDS_GAMMA_CORRECTION){j=l3(j);}
#endif
j.xyz=I2(j.xyz,j.w,c0.xy,n.B3,n.C3);
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
j=L1*(1.-j.w)+j;
#endif
z0(k0,j);
#endif
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
f2(h0);f2(Q0);
#else
z0(h0,D0(.0));
#endif
#ifdef Cb
A2;
#endif
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
j=(j*o);j.xyz=I2(j.xyz,j.w,c0.xy,n.B3,n.C3);C1=j;m3
#else
a2;
#endif
}
#endif
