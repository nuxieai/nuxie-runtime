#ifdef FRAGMENT
#if(defined(FIXED_FUNCTION_COLOR_OUTPUT)&&!defined(ENABLE_CLIPPING))||defined(RENDER_MODE_CLOCKWISE_ATOMIC)
#undef Eb
#else
#define Eb
#endif
L1
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
z0(G2,l0);
#endif
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
k1(V2,i0);
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
z0(i6,o4);
#endif
k1(L6,R0);
#else
z0(V2,i0);
#endif
M1
#ifdef DRAW_IMAGE_MESH
I3 c3(g5,Z3,GC);J3 h5 a4(Y5)i5 T3 U3
#endif
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
#ifdef DRAW_IMAGE_MESH
v2(HB)
#else
v2(HB)
#endif
#else
#ifdef DRAW_IMAGE_MESH
O1(HB)
#else
O1(HB)
#endif
#endif
{
#ifdef FEATHER_ATLAS_BLIT
r(X1,f);
#if defined(ENABLE_MODULATED_IMAGE)
r(C2,R);
#endif
r(F2,c);
#endif
#ifdef ENABLE_CLIPPING
r(O3,d);
#endif
#ifdef ENABLE_CLIP_RECT
r(N0,f);
#endif
#if defined(FEATHER_ATLAS_BLIT)&&defined(ENABLE_ADVANCED_BLEND)
r(g1,d);
#endif
#ifdef DRAW_IMAGE_MESH
r(K5,c);r(J1,i);
#ifdef ENABLE_ADVANCED_BLEND
r(C1,L);
#endif
#endif
#ifdef FEATHER_ATLAS_BLIT
i k=N7(
#ifdef ENABLE_MODULATED_IMAGE
C2,
#endif
#ifdef ENABLE_ADVANCED_BLEND
e3(g1),
#endif
X1 W2);d o=clamp(j2(ED,T9,F2,.0).x,J0(.0),J0(1.));
#endif
#ifdef DRAW_IMAGE_MESH
i k=C7(GC,Y5,K5,j.Fd);d o=1.;
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){d a5=max(k3(f5(N0)),J0(.0));o=min(a5,o);}
#endif
#ifdef Eb
z2;
#endif
#if defined(ENABLE_CLIPPING)
if(ENABLE_CLIPPING&&O3!=.0){d z3;
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
E P0=unpackHalf2x16(Z0(i0));d G6=P0.y;z3=max(G6==O3?P0.x:J0(.0),J0(.0));
#else
z3=K0(i0).x;
#endif
z3=max(z3,J0(.0));o=min(o,z3);}
#endif
#ifdef DRAW_IMAGE_MESH
k*=J1;
#endif
#if!defined(FIXED_FUNCTION_COLOR_OUTPUT)
i N1=K0(l0);
#ifdef ENABLE_ADVANCED_BLEND
#ifdef FEATHER_ATLAS_BLIT
L n3=e3(g1);
#endif
#ifdef DRAW_IMAGE_MESH
L n3=C1;
#endif
if(ENABLE_ADVANCED_BLEND&&n3!=A4){
#ifdef DRAW_IMAGE_MESH
k.xyz=H6(k);
#endif
k.xyz=X4(k.xyz,N1,n3)*k.w;}
#endif
k*=o;
#ifdef NEEDS_GAMMA_CORRECTION
if(NEEDS_GAMMA_CORRECTION){k=o3(k);}
#endif
k.xyz=J2(k.xyz,k.w,d0.xy,j.F3,j.G3);
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
k=N1*(1.-k.w)+k;
#endif
A0(l0,k);
#endif
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
h2(i0);h2(R0);
#else
A0(i0,E0(.0));
#endif
#ifdef Eb
A2;
#endif
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
k=(k*o);k.xyz=J2(k.xyz,k.w,d0.xy,j.F3,j.G3);E1=k;p3
#else
d2;
#endif
}
#endif
