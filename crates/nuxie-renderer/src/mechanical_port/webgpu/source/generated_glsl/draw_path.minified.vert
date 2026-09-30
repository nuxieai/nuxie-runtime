#undef H2
#ifdef ENABLE_FEATHER
#define H2 e
#else
#define H2 C
#endif
#ifdef VERTEX
c1(d0)
#if defined(DRAW_INTERIOR_TRIANGLES)||defined(FEATHER_ATLAS_BLIT)
K(0,c4,MB);
#else
K(0,e,WB);K(1,e,XB);
#endif
d1
#endif
l2 E0 W(0,e,a1);
#ifdef FEATHER_ATLAS_BLIT
E0 W(1,c,K2);
#elif!defined(RENDER_MODE_DEPTH_STENCIL)
#ifdef DRAW_INTERIOR_TRIANGLES
OPTIONALLY_FLAT W(1,d,m1);
#else
E0 W(2,H2,S);
#endif
OPTIONALLY_FLAT W(3,d,F0);
#endif
#ifdef ENABLE_CLIPPING
#ifdef FEATHER_ATLAS_BLIT
OPTIONALLY_FLAT W(4,d,Z3);
#else
OPTIONALLY_FLAT W(4,C,l1);
#endif
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
E0 W(5,e,R0);
#endif
#ifdef ENABLE_ADVANCED_BLEND
OPTIONALLY_FLAT W(6,d,Q0);
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
a3 W(7,O0,q3);W(8,c,F4);
#endif
#ifdef ENABLE_MODULATED_IMAGE
E0 W(9,P,F1);
#endif
d2
#ifdef VERTEX
r1(RB,d0,D,G,r){
#if defined(DRAW_INTERIOR_TRIANGLES)||defined(FEATHER_ATLAS_BLIT)
L(G,D,MB,P);
#else
L(G,D,WB,e);L(G,D,XB,e);
#endif
T(a1,e);
#if defined(ENABLE_MODULATED_IMAGE)
T(F1,P);
#endif
#ifdef FEATHER_ATLAS_BLIT
T(K2,c);
#elif!defined(RENDER_MODE_DEPTH_STENCIL)
#ifdef DRAW_INTERIOR_TRIANGLES
T(m1,d);
#else
T(S,H2);
#endif
T(F0,d);
#endif
#ifdef ENABLE_CLIPPING
#ifdef FEATHER_ATLAS_BLIT
T(Z3,d);
#else
T(l1,C);
#endif
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
T(R0,e);
#endif
#ifdef ENABLE_ADVANCED_BLEND
T(Q0,d);
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
T(q3,O0);T(F4,c);
#endif
bool Oe=false;uint a0;c k0;
#ifdef RENDER_MODE_DEPTH_STENCIL
R A9;
#endif
#ifdef FEATHER_ATLAS_BLIT
k0=kc(MB,a0,
#ifdef RENDER_MODE_DEPTH_STENCIL
A9,
#endif
K2 H3);
#elif defined(DRAW_INTERIOR_TRIANGLES)
k0=lc(MB,a0
#ifdef RENDER_MODE_DEPTH_STENCIL
,A9
#else
,m1
#endif
H3);
#else
e U;Oe=!K9(WB,XB,r,a0,k0
#ifndef RENDER_MODE_DEPTH_STENCIL
,U
#else
,A9
#endif
H3);
#ifndef RENDER_MODE_DEPTH_STENCIL
#ifdef ENABLE_FEATHER
S=U;
#else
S.xy=f8(U.xy);
#endif
#endif
#endif
O0 L0=k5(XC,a0);
#if!defined(FEATHER_ATLAS_BLIT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
F0=k6(a0,j.T4);if((L0.x&ea)!=0u) F0=-F0;
#endif
uint n2=L0.x&0xfu;
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING){uint Qb=(n2==n5?L0.y:L0.x)>>16;d X0=k6(Qb,j.T4);if(n2==n5) X0=-X0;
#ifdef FEATHER_ATLAS_BLIT
Z3=X0;
#else
l1.x=X0;
#endif
}
#endif
#ifdef ENABLE_ADVANCED_BLEND
if(ENABLE_ADVANCED_BLEND){Q0=float((L0.x>>4)&0xfu);}
#endif
c l0=k0;
#ifdef ENABLE_RENDER_TARGET_BOTTOM_UP
if(j.W9!=0u){l0.y=float(j.X9)-l0.y;}
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){Y C3=n1(p0(JB,a0*f2+2u));e Q3=p0(JB,a0*f2+3u);
#ifndef RENDER_MODE_DEPTH_STENCIL
R0=h8(C3,Q3.xy,l0);
#else
Ga(C3,Q3.xy,l0 Y4);
#endif
}
#endif
if(n2==fa){a1=e(unpackUnorm4x8(L0.y));}
#if defined(ENABLE_CLIPPING)&&!defined(FEATHER_ATLAS_BLIT)
else if(ENABLE_CLIPPING&&n2==n5){d E4=k6(L0.x>>16,j.T4);l1.y=E4;}
#endif
else{Y Rb=n1(p0(JB,a0*f2));e W7=p0(JB,a0*f2+1u);a1=Y9(l0,Rb,W7.xy,float(n2),W7.zw,uintBitsToFloat(L0.y));a1.w=-a1.w;}
#if defined(ENABLE_MODULATED_IMAGE)
if(ENABLE_MODULATED_IMAGE&&(L0.x&vd)!=0u){Y Sb=n1(p0(JB,a0*f2+4u));e X7=p0(JB,a0*f2+5u);c o3=K0(Sb,l0)+X7.xy;F1=P(o3.x,o3.y,1.+X7.z);}else{F1=P(0.0,0.0,0.0);}
#endif
e I;if(!Oe){I=I3(k0);
#ifdef POST_INVERT_Y
I.y=-I.y;
#endif
#ifdef RENDER_MODE_DEPTH_STENCIL
I.z=H8(A9,0xffu);
#elif defined(RENDER_MODE_CLOCKWISE_ATOMIC)
N d5=p0(LB,a0*4u+3u);q3=d5.xy;F4=k0+uintBitsToFloat(d5.zw);
#endif
}else{I=e(j.c3,j.c3,j.c3,j.c3);}Z(a1);
#if defined(ENABLE_MODULATED_IMAGE)
Z(F1);
#endif
#ifdef FEATHER_ATLAS_BLIT
Z(K2);
#elif!defined(RENDER_MODE_DEPTH_STENCIL)
#ifdef DRAW_INTERIOR_TRIANGLES
Z(m1);
#else
Z(S);
#endif
Z(F0);
#endif
#ifdef ENABLE_CLIPPING
#ifdef FEATHER_ATLAS_BLIT
Z(Z3);
#else
Z(l1);
#endif
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
Z(R0);
#endif
#ifdef ENABLE_ADVANCED_BLEND
Z(Q0);
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
Z(q3);Z(F4);
#endif
v1(I);}
#endif
#ifdef FRAGMENT
f4 g4 f i Y7(
#ifdef ENABLE_MODULATED_IMAGE
P Tb,
#endif
#ifdef ENABLE_ADVANCED_BLEND
R y3,
#endif
e e5 V6){
#ifdef ENABLE_ADVANCED_BLEND
bool m5=ENABLE_ADVANCED_BLEND&&y3!=L4;
#else
const bool m5=false;
#endif
i l;if(e5.w>=.0){l=q5(e5);}else{e5.w=-e5.w;d ia=d4(fract(e5.w)*(256./255.));e5.w=floor(e5.w)*j.wc+j.xc;c oa=Cc(e5);l=o2(FD,ha,oa,.0);if(!m5){l.xyz*=l.w;l.w*=ia;}}
#if defined(ENABLE_MODULATED_IMAGE)
if(ENABLE_MODULATED_IMAGE&&Tb.z>0.0){d Mi=Tb.z-1.;i p2=f7(IC,f6,Tb.xy,Mi);if(m5) p2=G0(P6(p2),p2.w);l*=p2;}
#endif
return l;}
#if!defined(DRAW_INTERIOR_TRIANGLES)&&!defined(FEATHER_ATLAS_BLIT)
f d Pe(H2 U S3){
#ifdef ENABLE_FEATHER
if(ENABLE_FEATHER&&yc(U)) return M4(U k1);else
#endif
return min(U.x,U.y);}f d Qe(H2 U S3){
#if defined(ENABLE_FEATHER)
if(ENABLE_FEATHER&&zc(U)) return p8(U k1);else
#endif
return U.x;}f d Ub(H2 U S3){if(e6(U)) return Pe(U k1);else return Qe(U k1);}f d Ni(d f5,H2 U S3){if(e6(U)){d z0=Pe(U k1);return max(z0,f5);}else{d z0=Qe(U k1);return f5+z0;}}
#endif
#endif
