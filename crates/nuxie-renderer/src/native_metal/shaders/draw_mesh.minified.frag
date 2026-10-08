#ifdef FRAGMENT
#if(defined(FIXED_FUNCTION_COLOR_OUTPUT)&&!defined(ENABLE_CLIPPING))||defined(RENDER_MODE_CLOCKWISE_ATOMIC)
#undef Ic
#else
#define Ic
#endif
V1
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
C0(U2,n0);
#endif
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
q1(i3,m0);
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
C0(w6,G4);
#endif
q1(d7,Z0);
#else
C0(i3,m0);
#endif
W1
#ifdef DRAW_IMAGE_MESH
U3 p3(x5,q4,TB);V3 y5 r4(S4) z5 k4 l4
#endif
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
#ifdef DRAW_IMAGE_MESH
G2(IB)
#else
G2(IB)
#endif
#else
#ifdef DRAW_IMAGE_MESH
Y1(IB)
#else
Y1(IB)
#endif
#endif
{
#ifdef FEATHER_ATLAS_BLIT
q(P0,e);
#if defined(ENABLE_MODULATED_IMAGE)
q(V0,M);
#endif
q(T2,c);
#endif
#ifdef ENABLE_CLIPPING
q(e4,d);
#endif
#ifdef ENABLE_CLIP_RECT
q(W0,e);
#endif
#if defined(FEATHER_ATLAS_BLIT)&&defined(ENABLE_ADVANCED_BLEND)
q(Q0,d);
#endif
#ifdef DRAW_IMAGE_MESH
q(Z5,c);q(U1,i);
#ifdef ENABLE_ADVANCED_BLEND
q(K1,P);
#endif
#endif
#ifdef FEATHER_ATLAS_BLIT
i n=p8(
#ifdef ENABLE_MODULATED_IMAGE
V0,
#endif
#ifdef ENABLE_ADVANCED_BLEND
X2(Q0),
#endif
P0 l3);d l=clamp(o2(HD,La,T2,.0).x,J0(.0),J0(1.));
#endif
#ifdef DRAW_IMAGE_MESH
i n=d8(TB,S4,Z5,j.Ee);d l=1.;
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){d r5=max(A3(T4(W0)),J0(.0));l=min(r5,l);}
#endif
#ifdef Ic
O2;
#endif
#if defined(ENABLE_CLIPPING)
if(ENABLE_CLIPPING&&e4!=.0){d O3;
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
D X0=unpackHalf2x16(l1(m0));d Y6=X0.y;O3=max(Y6==e4?X0.x:J0(.0),J0(.0));
#else
O3=R0(m0).x;
#endif
O3=max(O3,J0(.0));l=min(l,O3);}
#endif
#ifdef DRAW_IMAGE_MESH
n*=U1;
#endif
#if!defined(FIXED_FUNCTION_COLOR_OUTPUT)
i A1=R0(n0);
#ifdef ENABLE_ADVANCED_BLEND
#ifdef FEATHER_ATLAS_BLIT
P X1=X2(Q0);
#endif
#ifdef DRAW_IMAGE_MESH
P X1=K1;
#endif
if(ENABLE_ADVANCED_BLEND&&X1!=T3){
#ifdef DRAW_IMAGE_MESH
n.xyz=f6(n);
#endif
n.xyz=L4(n.xyz,A1,X1)*n.w;}
#endif
n*=l;n.xyz=I2(n.xyz,n.w,d0.xy,j.E3,j.F3);
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
n=A1*(1.-n.w)+n;
#endif
z0(n0,n);
#endif
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
h2(m0);h2(Z0);
#else
z0(m0,H0(.0));
#endif
#ifdef Ic
P2;
#endif
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
n=(n*l);n.xyz=I2(n.xyz,n.w,d0.xy,j.E3,j.F3);N1=n;D3
#else
p2;
#endif
}
#endif
