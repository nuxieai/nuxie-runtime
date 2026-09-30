#ifdef FRAGMENT
#if(defined(FIXED_FUNCTION_COLOR_OUTPUT)&&!defined(ENABLE_CLIPPING))||defined(RENDER_MODE_CLOCKWISE_ATOMIC)
#undef dc
#else
#define dc
#endif
R1
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
B0(L2,n0);
#endif
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
o1(d3,m0);
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
B0(p6,C4);
#endif
o1(V6,W0);
#else
B0(d3,m0);
#endif
S1
#ifdef DRAW_IMAGE_MESH
O3 i3(x5,m4,DC);P3 y5 n4(v5) z5 g4 h4
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
q(a1,f);
#if defined(ENABLE_MODULATED_IMAGE)
q(r1,P);
#endif
q(K2,c);
#endif
#ifdef ENABLE_CLIPPING
q(Z3,d);
#endif
#ifdef ENABLE_CLIP_RECT
q(S0,f);
#endif
#if defined(FEATHER_ATLAS_BLIT)&&defined(ENABLE_ADVANCED_BLEND)
q(Q0,d);
#endif
#ifdef DRAW_IMAGE_MESH
q(W5,c);q(Q1,i);
#ifdef ENABLE_ADVANCED_BLEND
q(H1,R);
#endif
#endif
#ifdef FEATHER_ATLAS_BLIT
i l=Z7(
#ifdef ENABLE_MODULATED_IMAGE
r1,
#endif
#ifdef ENABLE_ADVANCED_BLEND
k3(Q0),
#endif
a1 e3);d o=clamp(o2(GD,na,K2,.0).x,I0(.0),I0(1.));
#endif
#ifdef DRAW_IMAGE_MESH
i l=L7(DC,v5,W5,j.Wd);d o=1.;
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){d n5=max(v3(w5(S0)),I0(.0));o=min(n5,o);}
#endif
#ifdef dc
F2;
#endif
#if defined(ENABLE_CLIPPING)
if(ENABLE_CLIPPING&&Z3!=.0){d G3;
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
C U0=unpackHalf2x16(h1(m0));d Q6=U0.y;G3=max(Q6==Z3?U0.x:I0(.0),I0(.0));
#else
G3=N0(m0).x;
#endif
G3=max(G3,I0(.0));o=min(o,G3);}
#endif
#ifdef DRAW_IMAGE_MESH
l*=Q1;
#endif
#if!defined(FIXED_FUNCTION_COLOR_OUTPUT)
i I1=N0(n0);
#ifdef ENABLE_ADVANCED_BLEND
#ifdef FEATHER_ATLAS_BLIT
R y3=k3(Q0);
#endif
#ifdef DRAW_IMAGE_MESH
R y3=H1;
#endif
if(ENABLE_ADVANCED_BLEND&&y3!=M4){
#ifdef DRAW_IMAGE_MESH
l.xyz=R6(l);
#endif
l.xyz=i5(l.xyz,I1,y3)*l.w;}
#endif
l*=o;
#ifdef NEEDS_GAMMA_CORRECTION
if(NEEDS_GAMMA_CORRECTION){l=z3(l);}
#endif
l.xyz=O2(l.xyz,l.w,f0.xy,j.M3,j.N3);
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
l=I1*(1.-l.w)+l;
#endif
y0(n0,l);
#endif
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
Z1(m0);Z1(W0);
#else
y0(m0,G0(.0));
#endif
#ifdef dc
G2;
#endif
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
l=(l*o);l.xyz=O2(l.xyz,l.w,f0.xy,j.M3,j.N3);K1=l;A3
#else
h2;
#endif
}
#endif
