#undef H2
#ifdef ENABLE_FEATHER
#define H2 f
#else
#define H2 C
#endif
#ifdef VERTEX
c1(d0)
#if defined(DRAW_INTERIOR_TRIANGLES)||defined(FEATHER_ATLAS_BLIT)
K(0,d4,MB);
#else
K(0,f,WB);K(1,f,XB);
#endif
d1
#endif
l2 E0 W(0,f,a1);
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
E0 W(5,f,S0);
#endif
#ifdef ENABLE_ADVANCED_BLEND
OPTIONALLY_FLAT W(6,d,Q0);
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
a3 W(7,O0,q3);W(8,c,G4);
#endif
#ifdef ENABLE_MODULATED_IMAGE
E0 W(9,P,r1);
#endif
e2
#ifdef VERTEX
v1(RB,d0,D,G,r){
#if defined(DRAW_INTERIOR_TRIANGLES)||defined(FEATHER_ATLAS_BLIT)
L(G,D,MB,P);
#else
L(G,D,WB,f);L(G,D,XB,f);
#endif
T(a1,f);
#if defined(ENABLE_MODULATED_IMAGE)
T(r1,P);
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
T(S0,f);
#endif
#ifdef ENABLE_ADVANCED_BLEND
T(Q0,d);
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
T(q3,O0);T(G4,c);
#endif
bool Pe=false;uint a0;c k0;
#ifdef RENDER_MODE_DEPTH_STENCIL
R C9;
#endif
#ifdef FEATHER_ATLAS_BLIT
k0=lc(MB,a0,
#ifdef RENDER_MODE_DEPTH_STENCIL
C9,
#endif
K2 H3);
#elif defined(DRAW_INTERIOR_TRIANGLES)
k0=mc(MB,a0
#ifdef RENDER_MODE_DEPTH_STENCIL
,C9
#else
,m1
#endif
H3);
#else
f U;Pe=!L9(WB,XB,r,a0,k0
#ifndef RENDER_MODE_DEPTH_STENCIL
,U
#else
,C9
#endif
H3);
#ifndef RENDER_MODE_DEPTH_STENCIL
#ifdef ENABLE_FEATHER
S=U;
#else
S.xy=h8(U.xy);
#endif
#endif
#endif
O0 H0=m5(XC,a0);
#if!defined(FEATHER_ATLAS_BLIT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
F0=m6(a0,j.U4);if((H0.x&fa)!=0u) F0=-F0;
#endif
uint n2=H0.x&0xfu;
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING){uint Rb=(n2==p5?H0.y:H0.x)>>16;d X0=m6(Rb,j.U4);if(n2==p5) X0=-X0;
#ifdef FEATHER_ATLAS_BLIT
Z3=X0;
#else
l1.x=X0;
#endif
}
#endif
#ifdef ENABLE_ADVANCED_BLEND
if(ENABLE_ADVANCED_BLEND){Q0=float((H0.x>>4)&0xfu);}
#endif
c l0=k0;
#ifdef ENABLE_RENDER_TARGET_BOTTOM_UP
if(j.X9!=0u){l0.y=float(j.Y9)-l0.y;}
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){Y C3=n1(p0(JB,a0*g2+2u));f Q3=p0(JB,a0*g2+3u);
#ifndef RENDER_MODE_DEPTH_STENCIL
S0=j8(C3,Q3.xy,l0);
#else
Ha(C3,Q3.xy,l0 Z4);
#endif
}
#endif
if(n2==ga){a1=f(unpackUnorm4x8(H0.y));}
#if defined(ENABLE_CLIPPING)&&!defined(FEATHER_ATLAS_BLIT)
else if(ENABLE_CLIPPING&&n2==p5){d F4=m6(H0.x>>16,j.U4);l1.y=F4;}
#endif
else{Y Sb=n1(p0(JB,a0*g2));f X7=p0(JB,a0*g2+1u);a1=Z9(l0,Sb,X7.xy,float(n2),X7.zw,uintBitsToFloat(H0.y));a1.w=-a1.w;}
#if defined(ENABLE_MODULATED_IMAGE)
if(ENABLE_MODULATED_IMAGE&&(H0.x&wd)!=0u){Y Tb=n1(p0(JB,a0*g2+4u));f Y7=p0(JB,a0*g2+5u);c o3=M0(Tb,l0)+Y7.xy;float Qe=1.+Y7.z;if((H0.x&Jg)!=0u){uint c4=(H0.x&Lg)>>Kg;Qe=-(1.+float(c4));}r1=P(o3.x,o3.y,Qe);}else{r1=P(0.0,0.0,0.0);}
#endif
f I;if(!Pe){I=I3(k0);
#ifdef POST_INVERT_Y
I.y=-I.y;
#endif
#ifdef RENDER_MODE_DEPTH_STENCIL
I.z=J8(C9,0xffu);
#elif defined(RENDER_MODE_CLOCKWISE_ATOMIC)
N e5=p0(LB,a0*4u+3u);q3=e5.xy;G4=k0+uintBitsToFloat(e5.zw);
#endif
}else{I=f(j.c3,j.c3,j.c3,j.c3);}Z(a1);
#if defined(ENABLE_MODULATED_IMAGE)
Z(r1);
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
Z(S0);
#endif
#ifdef ENABLE_ADVANCED_BLEND
Z(Q0);
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
Z(q3);Z(G4);
#endif
w1(I);}
#endif
#ifdef FRAGMENT
g4 h4 e d Wi(i Ub,uint c4){d Re=dot(Ub.xyz,R0(.30,.59,.11));if(c4==Mg) return Ub.w;if(c4==Ng) return 1.-Ub.w;if(c4==Og) return Re;return 1.-Re;}e i Z7(
#ifdef ENABLE_MODULATED_IMAGE
P a8,
#endif
#ifdef ENABLE_ADVANCED_BLEND
R y3,
#endif
f f5 X6){
#ifdef ENABLE_ADVANCED_BLEND
bool o5=ENABLE_ADVANCED_BLEND&&y3!=M4;
#else
const bool o5=false;
#endif
i l;if(f5.w>=.0){l=w5(f5);}else{f5.w=-f5.w;d ja=e4(fract(f5.w)*(256./255.));f5.w=floor(f5.w)*j.xc+j.yc;c pa=Dc(f5);l=o2(FD,ia,pa,.0);if(!o5){l.xyz*=l.w;l.w*=ja;}}
#if defined(ENABLE_MODULATED_IMAGE)
if(ENABLE_MODULATED_IMAGE&&a8.z<0.0){return j6(DC,v5,a8.xy,I0(.0));}if(ENABLE_MODULATED_IMAGE&&a8.z>0.0){d Xi=a8.z-1.;i p2=j6(DC,v5,a8.xy,Xi);if(o5) p2=G0(R6(p2),p2.w);l*=p2;}
#endif
return l;}
#if!defined(DRAW_INTERIOR_TRIANGLES)&&!defined(FEATHER_ATLAS_BLIT)
e d Se(H2 U S3){
#ifdef ENABLE_FEATHER
if(ENABLE_FEATHER&&zc(U)) return N4(U k1);else
#endif
return min(U.x,U.y);}e d Te(H2 U S3){
#if defined(ENABLE_FEATHER)
if(ENABLE_FEATHER&&Ac(U)) return r8(U k1);else
#endif
return U.x;}e d Vb(H2 U S3){if(g6(U)) return Se(U k1);else return Te(U k1);}e d Yi(d g5,H2 U S3){if(g6(U)){d A0=Se(U k1);return max(A0,g5);}else{d A0=Te(U k1);return g5+A0;}}
#endif
#endif
