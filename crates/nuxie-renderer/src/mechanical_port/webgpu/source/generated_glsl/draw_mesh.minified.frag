#ifdef FRAGMENT
#if(defined(FIXED_FUNCTION_COLOR_OUTPUT)&&!defined(ENABLE_CLIPPING))||defined(RENDER_MODE_CLOCKWISE_ATOMIC)
#undef cc
#else
#define cc
#endif
Q1
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
A0(L2,o0);
#endif
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
o1(d3,m0);
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
A0(n6,B4);
#endif
o1(T6,V0);
#else
A0(d3,m0);
#endif
R1
#ifdef DRAW_IMAGE_MESH
O3 i3(r5,l4,IC);P3 v5 m4(f6) w5 f4 g4
#endif
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
#ifdef DRAW_IMAGE_MESH
A2(IB)
#else
A2(IB)
#endif
#else
#ifdef DRAW_IMAGE_MESH
T1(IB)
#else
T1(IB)
#endif
#endif
{
#ifdef FEATHER_ATLAS_BLIT
q(a1,e);
#if defined(ENABLE_MODULATED_IMAGE)
q(F1,P);
#endif
q(K2,c);
#endif
#ifdef ENABLE_CLIPPING
q(Z3,d);
#endif
#ifdef ENABLE_CLIP_RECT
q(R0,e);
#endif
#if defined(FEATHER_ATLAS_BLIT)&&defined(ENABLE_ADVANCED_BLEND)
q(Q0,d);
#endif
#ifdef DRAW_IMAGE_MESH
q(T5,c);q(P1,i);
#ifdef ENABLE_ADVANCED_BLEND
q(H1,R);
#endif
#endif
#ifdef FEATHER_ATLAS_BLIT
i l=Y7(
#ifdef ENABLE_MODULATED_IMAGE
F1,
#endif
#ifdef ENABLE_ADVANCED_BLEND
k3(Q0),
#endif
a1 e3);d o=clamp(o2(GD,ma,K2,.0).x,M0(.0),M0(1.));
#endif
#ifdef DRAW_IMAGE_MESH
i l=K7(IC,f6,T5,j.Vd);d o=1.;
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){d l5=max(v3(q5(R0)),M0(.0));o=min(l5,o);}
#endif
#ifdef cc
F2;
#endif
#if defined(ENABLE_CLIPPING)
if(ENABLE_CLIPPING&&Z3!=.0){d G3;
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
C T0=unpackHalf2x16(h1(m0));d O6=T0.y;G3=max(O6==Z3?T0.x:M0(.0),M0(.0));
#else
G3=N0(m0).x;
#endif
G3=max(G3,M0(.0));o=min(o,G3);}
#endif
#ifdef DRAW_IMAGE_MESH
l*=P1;
#endif
#if!defined(FIXED_FUNCTION_COLOR_OUTPUT)
i S1=N0(o0);
#ifdef ENABLE_ADVANCED_BLEND
#ifdef FEATHER_ATLAS_BLIT
R y3=k3(Q0);
#endif
#ifdef DRAW_IMAGE_MESH
R y3=H1;
#endif
if(ENABLE_ADVANCED_BLEND&&y3!=L4){
#ifdef DRAW_IMAGE_MESH
l.xyz=P6(l);
#endif
l.xyz=h5(l.xyz,S1,y3)*l.w;}
#endif
l*=o;
#ifdef NEEDS_GAMMA_CORRECTION
if(NEEDS_GAMMA_CORRECTION){l=z3(l);}
#endif
l.xyz=O2(l.xyz,l.w,f0.xy,j.M3,j.N3);
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
l=S1*(1.-l.w)+l;
#endif
B0(o0,l);
#endif
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
k2(m0);k2(V0);
#else
B0(m0,G0(.0));
#endif
#ifdef cc
G2;
#endif
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
l=(l*o);l.xyz=O2(l.xyz,l.w,f0.xy,j.M3,j.N3);J1=l;A3
#else
g2;
#endif
}
#endif
