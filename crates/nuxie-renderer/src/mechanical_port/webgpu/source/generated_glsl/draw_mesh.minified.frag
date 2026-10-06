#ifdef FRAGMENT
#if(defined(FIXED_FUNCTION_COLOR_OUTPUT)&&!defined(ENABLE_CLIPPING))||defined(RENDER_MODE_CLOCKWISE_ATOMIC)
#undef dc
#else
#define dc
#endif
S1
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
B0(K2,n0);
#endif
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
o1(c3,m0);
#ifndef FIXED_FUNCTION_COLOR_OUTPUT
B0(o6,C4);
#endif
o1(U6,V0);
#else
B0(c3,m0);
#endif
T1
#ifdef DRAW_IMAGE_MESH
O3 i3(w5,m4,CC);P3 x5 n4(r5) y5 g4 h4
#endif
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
#ifdef DRAW_IMAGE_MESH
z2(IB)
#else
z2(IB)
#endif
#else
#ifdef DRAW_IMAGE_MESH
U1(IB)
#else
U1(IB)
#endif
#endif
{
#ifdef FEATHER_ATLAS_BLIT
q(a1,e);
#if defined(ENABLE_MODULATED_IMAGE)
q(v1,O);
#endif
q(J2,c);
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
q(V5,c);q(R1,i);
#ifdef ENABLE_ADVANCED_BLEND
q(I1,Q);
#endif
#endif
#ifdef FEATHER_ATLAS_BLIT
i p=X7(
#ifdef ENABLE_MODULATED_IMAGE
v1,
#endif
#ifdef ENABLE_ADVANCED_BLEND
k3(Q0),
#endif
a1 e3);d n=clamp(o2(FD,na,J2,.0).x,H0(.0),H0(1.));
#endif
#ifdef DRAW_IMAGE_MESH
i p=J7(CC,r5,V5,j.Wd);d n=1.;
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){d m5=max(w3(v5(R0)),H0(.0));n=min(m5,n);}
#endif
#ifdef dc
E2;
#endif
#if defined(ENABLE_CLIPPING)
if(ENABLE_CLIPPING&&Z3!=.0){d G3;
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
C T0=unpackHalf2x16(h1(m0));d P6=T0.y;G3=max(P6==Z3?T0.x:H0(.0),H0(.0));
#else
G3=N0(m0).x;
#endif
G3=max(G3,H0(.0));n=min(n,G3);}
#endif
#ifdef DRAW_IMAGE_MESH
p*=R1;
#endif
#if!defined(FIXED_FUNCTION_COLOR_OUTPUT)
i J1=N0(n0);
#ifdef ENABLE_ADVANCED_BLEND
#ifdef FEATHER_ATLAS_BLIT
Q z3=k3(Q0);
#endif
#ifdef DRAW_IMAGE_MESH
Q z3=I1;
#endif
if(ENABLE_ADVANCED_BLEND&&z3!=M4){
#ifdef DRAW_IMAGE_MESH
p.xyz=Q6(p);
#endif
p.xyz=h5(p.xyz,J1,z3)*p.w;}
#endif
p*=n;p.xyz=M2(p.xyz,p.w,f0.xy,j.M3,j.N3);
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
p=J1*(1.-p.w)+p;
#endif
y0(n0,p);
#endif
#ifndef RENDER_MODE_CLOCKWISE_ATOMIC
a2(m0);a2(V0);
#else
y0(m0,I0(.0));
#endif
#ifdef dc
F2;
#endif
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
p=(p*n);p.xyz=M2(p.xyz,p.w,f0.xy,j.M3,j.N3);L1=p;A3
#else
h2;
#endif
}
#endif
