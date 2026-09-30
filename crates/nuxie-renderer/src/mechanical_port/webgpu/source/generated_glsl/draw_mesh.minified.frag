#ifdef FRAGMENT
#if(defined(FIXED_FUNCTION_COLOR_OUTPUT)&&!defined(ENABLE_CLIPPING))||defined(RENDER_MODE_CLOCKWISE_ATOMIC)
#undef Bb
#else
#define Bb
#endif
M1
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
z0(H2,l0);
#endif
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
j1(Y2,i0);
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
z0(f6,q4);
#endif
j1(K6,S0);
#else
z0(Y2,i0);
#endif
N1
#ifdef DRAW_IMAGE_MESH
H3 e3(i5,a4,HC);I3 j5 c4(W5) k5 T3 U3
#endif
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
#ifdef DRAW_IMAGE_MESH
w2(HB)
#else
w2(HB)
#endif
#else
#ifdef DRAW_IMAGE_MESH
P1(HB)
#else
P1(HB)
#endif
#endif
{
#ifdef FEATHER_ATLAS_BLIT
r(X1,f);
#if defined(ENABLE_MODULATED_IMAGE)
r(D2,S);
#endif
r(G2,c);
#endif
#ifdef ENABLE_CLIPPING
r(N3,d);
#endif
#ifdef ENABLE_CLIP_RECT
r(P0,f);
#endif
#if defined(FEATHER_ATLAS_BLIT)&&defined(ENABLE_ADVANCED_BLEND)
r(f1,d);
#endif
#ifdef DRAW_IMAGE_MESH
r(J5,c);r(K1,i);
#ifdef ENABLE_ADVANCED_BLEND
r(C1,N);
#endif
#endif
#ifdef FEATHER_ATLAS_BLIT
i k=K7(
#ifdef ENABLE_MODULATED_IMAGE
D2,
#endif
#ifdef ENABLE_ADVANCED_BLEND
g3(f1),
#endif
X1 Z2);d o=clamp(j2(FD,S9,G2,.0).x,J0(.0),J0(1.));
#endif
#ifdef DRAW_IMAGE_MESH
i k=z7(HC,W5,J5,j.Dd);d o=1.;
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){d d5=max(m3(h5(P0)),J0(.0));o=min(d5,o);}
#endif
#ifdef Bb
A2;
#endif
#if defined(ENABLE_CLIPPING)
if(ENABLE_CLIPPING&&N3!=.0){d z3;
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
D Q0=unpackHalf2x16(a1(i0));d F6=Q0.y;z3=max(F6==N3?Q0.x:J0(.0),J0(.0));
#else
z3=K0(i0).x;
#endif
z3=max(z3,J0(.0));o=min(o,z3);}
#endif
#ifdef DRAW_IMAGE_MESH
k*=K1;
#endif
#if!defined(FIXED_FUNCTION_COLOR_OUTPUT)
i O1=K0(l0);
#ifdef ENABLE_ADVANCED_BLEND
#ifdef FEATHER_ATLAS_BLIT
N p3=g3(f1);
#endif
#ifdef DRAW_IMAGE_MESH
N p3=C1;
#endif
if(ENABLE_ADVANCED_BLEND&&p3!=C4){
#ifdef DRAW_IMAGE_MESH
k.xyz=G6(k);
#endif
k.xyz=Z4(k.xyz,O1,p3)*k.w;}
#endif
k*=o;
#ifdef NEEDS_GAMMA_CORRECTION
if(NEEDS_GAMMA_CORRECTION){k=q3(k);}
#endif
k.xyz=L2(k.xyz,k.w,e0.xy,j.F3,j.G3);
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
k=O1*(1.-k.w)+k;
#endif
A0(l0,k);
#endif
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
h2(i0);h2(S0);
#else
A0(i0,E0(.0));
#endif
#ifdef Bb
B2;
#endif
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
k=(k*o);k.xyz=L2(k.xyz,k.w,e0.xy,j.F3,j.G3);E1=k;r3
#else
d2;
#endif
}
#endif
