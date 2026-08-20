#ifdef FRAGMENT
#if(defined(FIXED_FUNCTION_COLOR_OUTPUT)&&!defined(ENABLE_CLIPPING))||defined(RENDER_MODE_CLOCKWISE_ATOMIC)
#undef vb
#else
#define vb
#endif
I1
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
w0(Q2,j0);
#endif
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
j1(R2,h0);
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
w0(d6,i4);
#endif
j1(G6,P0);
#else
w0(R2,h0);
#endif
J1
#ifdef DRAW_IMAGE_MESH
C3 X2(Z4,U3,IC);D3 a5 V3(S5)c5 O3 P3
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
A(f1,g);A(C2,d);
#endif
#ifdef ENABLE_CLIPPING
A(I3,c);
#endif
#ifdef ENABLE_CLIP_RECT
A(L0,g);
#endif
#if defined(FEATHER_ATLAS_BLIT)&&defined(ENABLE_ADVANCED_BLEND)
A(e2,c);
#endif
#ifdef DRAW_IMAGE_MESH
A(E5,d);A(H1,c);
#ifdef ENABLE_ADVANCED_BLEND
A(A1,N);
#endif
#endif
#ifdef FEATHER_ATLAS_BLIT
i j=J7(f1,1. S2);c o=clamp(n2(BD,Q9,C2,.0).x,G0(.0),G0(1.));
#endif
#ifdef DRAW_IMAGE_MESH
i j=x7(IC,S5,E5,n.qd);c o=1.;
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){c U4=max(f3(Y4(L0)),G0(.0));o=min(U4,o);}
#endif
#ifdef vb
w2;
#endif
#if defined(ENABLE_CLIPPING)
if(ENABLE_CLIPPING&&I3!=.0){c r3;
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
E M0=unpackHalf2x16(Y0(h0));c B6=M0.y;r3=max(B6==I3?M0.x:G0(.0),G0(.0));
#else
r3=H0(h0).x;
#endif
r3=max(r3,G0(.0));o=min(o,r3);}
#endif
#ifdef DRAW_IMAGE_MESH
o*=H1;
#endif
#if!defined(FIXED_FUNCTION_COLOR_OUTPUT)
i K1=H0(j0);
#ifdef ENABLE_ADVANCED_BLEND
if(ENABLE_ADVANCED_BLEND){
#ifdef FEATHER_ATLAS_BLIT
N R3=X5(e2);
#endif
#ifdef DRAW_IMAGE_MESH
j.xyz=C6(j);N R3=A1;
#endif
if(R3!=N5){j.xyz=Q4(j.xyz,K1,R3);}j.w*=o;j.xyz*=j.w;}else
#endif
{j*=o;}
#ifdef NEEDS_GAMMA_CORRECTION
if(NEEDS_GAMMA_CORRECTION){j=k3(j);}
#endif
j.xyz=E2(j.xyz,j.w,Y.xy,n.z3,n.A3);
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
#ifdef vb
x2;
#endif
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
j=(j*o);j.xyz=E2(j.xyz,j.w,Y.xy,n.z3,n.A3);C1=j;l3
#else
Y1;
#endif
}
#endif
