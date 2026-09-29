#undef I5
#ifdef NEVER_GENERATE_PREMULTIPLIED_PAINT_COLORS
#define I5 true
#elif defined(ENABLE_ADVANCED_BLEND)
#define I5 ENABLE_ADVANCED_BLEND
#else
#define I5 false
#endif
#undef z2
#ifdef ENABLE_FEATHER
#define z2 g
#else
#define z2 E
#endif
#ifdef VERTEX
g1(e0)
#if defined(DRAW_INTERIOR_TRIANGLES)||defined(FEATHER_ATLAS_BLIT)
L(0,N3,LB);
#else
L(0,g,VB);L(1,g,WB);
#endif
h1
#endif
m2 H0 X(0,g,f1);
#ifdef FEATHER_ATLAS_BLIT
H0 X(1,d,D2);
#elif!defined(RENDER_MODE_DEPTH_STENCIL)
#ifdef DRAW_INTERIOR_TRIANGLES
OPTIONALLY_FLAT X(1,c,i1);
#else
H0 X(2,z2,O);
#endif
OPTIONALLY_FLAT X(3,c,B0);
#endif
#ifdef ENABLE_CLIPPING
#ifdef FEATHER_ATLAS_BLIT
OPTIONALLY_FLAT X(4,c,K3);
#else
OPTIONALLY_FLAT X(4,E,V1);
#endif
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
H0 X(5,g,M0);
#endif
#ifdef ENABLE_ADVANCED_BLEND
OPTIONALLY_FLAT X(6,c,f2);
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
Q2 X(7,a1,f3);X(8,d,o4);
#endif
#ifdef ENABLE_MODULATED_IMAGE
H0 X(9,R,A2);
#endif
g2
#ifdef VERTEX
#ifdef EMULATE_DYNAMIC_COLOR_WRITE_DISABLE
Bd(gh)Cd(float,Rh)Dd(Sh)
#endif
z1(HC,e0,F,B,v){
#if defined(DRAW_INTERIOR_TRIANGLES)||defined(FEATHER_ATLAS_BLIT)
M(B,F,LB,R);
#else
M(B,F,VB,g);M(B,F,WB,g);
#endif
V(f1,g);
#if defined(ENABLE_MODULATED_IMAGE)
V(A2,R);
#endif
#ifdef FEATHER_ATLAS_BLIT
V(D2,d);
#elif!defined(RENDER_MODE_DEPTH_STENCIL)
#ifdef DRAW_INTERIOR_TRIANGLES
V(i1,c);
#else
V(O,z2);
#endif
V(B0,c);
#endif
#ifdef ENABLE_CLIPPING
#ifdef FEATHER_ATLAS_BLIT
V(K3,c);
#else
V(V1,E);
#endif
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
V(M0,g);
#endif
#ifdef ENABLE_ADVANCED_BLEND
V(f2,c);
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
V(f3,a1);V(o4,d);
#endif
bool je=false;uint l0;d m0;
#ifdef RENDER_MODE_DEPTH_STENCIL
N h9;
#endif
#ifdef FEATHER_ATLAS_BLIT
m0=Ib(LB,l0,
#ifdef RENDER_MODE_DEPTH_STENCIL
h9,
#endif
D2 w3);
#elif defined(DRAW_INTERIOR_TRIANGLES)
m0=Jb(LB,l0
#ifdef RENDER_MODE_DEPTH_STENCIL
,h9
#else
,i1
#endif
w3);
#else
g P;je=!r9(VB,WB,v,l0,m0
#ifndef RENDER_MODE_DEPTH_STENCIL
,P
#else
,h9
#endif
w3);
#ifndef RENDER_MODE_DEPTH_STENCIL
#ifdef ENABLE_FEATHER
O=P;
#else
O.xy=S7(P.xy);
#endif
#endif
#endif
a1 p1=Q5(BD,l0);
#if!defined(FEATHER_ATLAS_BLIT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
B0=v8(l0,m.e6);if((p1.x&L9)!=0u)B0=-B0;
#endif
uint l3=p1.x&0xfu;
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING){uint Th=(l3==a8?p1.y:p1.x)>>16;c k1=v8(Th,m.e6);if(l3==a8)k1=-k1;
#ifdef FEATHER_ATLAS_BLIT
K3=k1;
#else
V1.x=k1;
#endif
}
#endif
#ifdef ENABLE_ADVANCED_BLEND
if(ENABLE_ADVANCED_BLEND){f2=float((p1.x>>4)&0xfu);}
#endif
d L0=m0;
#ifdef FRAMEBUFFER_BOTTOM_UP
L0.y=float(m.Qg)-L0.y;
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){f0 Z3=h2(J0(RB,l0*A3+2u));g I4=J0(RB,l0*A3+3u);
#ifndef RENDER_MODE_DEPTH_STENCIL
M0=U7(Z3,I4.xy,L0);
#else
Dc(Z3,I4.xy,L0 y5);
#endif
}
#endif
if(l3==Qb){i j=unpackUnorm4x8(p1.y);if(I5){}else{j.xyz*=j.w;}f1=g(j);}
#if defined(ENABLE_CLIPPING)&&!defined(FEATHER_ATLAS_BLIT)
else if(ENABLE_CLIPPING&&l3==a8){c J5=v8(p1.x>>16,m.e6);V1.y=J5;}
#endif
else{f0 pb=h2(J0(RB,l0*A3));g qb=J0(RB,l0*A3+1u);d y4=R0(pb,L0)+qb.xy;if(l3==N9||l3==Mf){f1.w=-uintBitsToFloat(p1.y);float Uh=qb.z;if(Uh>.9){f1.z=2.;}else{f1.z=qb.w;}if(l3==N9){f1.y=.0;f1.x=y4.x;}else{f1.z=-f1.z;f1.xy=y4.xy;}}}
#ifdef EMULATE_DYNAMIC_COLOR_WRITE_DISABLE
if(EMULATE_DYNAMIC_COLOR_WRITE_DISABLE){f1*=Sh.Rh;}
#endif
#if defined(ENABLE_MODULATED_IMAGE)
if(ENABLE_MODULATED_IMAGE&&(p1.x&Nf)!=0u){f0 pb=h2(J0(RB,l0*A3+4u));g ke=J0(RB,l0*A3+5u);d y4=R0(pb,L0)+ke.xy;A2=R(y4.x,y4.y,1.+ke.z);}else{A2=R(0.0,0.0,0.0);}
#endif
g W;if(!je){W=M3(m0);
#ifdef POST_INVERT_Y
W.y=-W.y;
#endif
#ifdef RENDER_MODE_DEPTH_STENCIL
W.z=ka(h9);
#elif defined(RENDER_MODE_CLOCKWISE_ATOMIC)
H S4=J0(QB,l0*4u+3u);f3=S4.xy;o4=m0+uintBitsToFloat(S4.zw);
#endif
}else{W=g(m.R2,m.R2,m.R2,m.R2);}c0(f1);
#if defined(ENABLE_MODULATED_IMAGE)
c0(A2);
#endif
#ifdef FEATHER_ATLAS_BLIT
c0(D2);
#elif!defined(RENDER_MODE_DEPTH_STENCIL)
#ifdef DRAW_INTERIOR_TRIANGLES
c0(i1);
#else
c0(O);
#endif
c0(B0);
#endif
#ifdef ENABLE_CLIPPING
#ifdef FEATHER_ATLAS_BLIT
c0(K3);
#else
c0(V1);
#endif
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
c0(M0);
#endif
#ifdef ENABLE_ADVANCED_BLEND
c0(f2);
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
c0(f3);c0(o4);
#endif
A1(W);}
#endif
#ifdef FRAGMENT
Q3 R3 e i N7(g K5,
#ifdef ENABLE_MODULATED_IMAGE
R rb,
#endif
float n M6){i j;if(K5.w>=.0){j=d5(K5);if(I5)j.w*=n;else j*=n;}else{float t=K5.z>.0?K5.x:length(K5.xy);t=clamp(t,.0,1.);float le=abs(K5.z);float x=le>1.?(1.-1./oa)*t+(.5/oa):(1./oa)*t+le;float Vh=-K5.w;j=o2(ND,Rb,d(x,Vh),.0);j.w*=n;if(I5){}else{j.xyz*=j.w;}}
#if defined(ENABLE_MODULATED_IMAGE)
if(ENABLE_MODULATED_IMAGE&&rb.z>0.0){c Wh=rb.z-1.;i G2=V6(JC,W5,rb.xy,Wh);if(I5)G2=C0(H6(G2),G2.w);j*=G2;}
#endif
return j;}
#if!defined(DRAW_INTERIOR_TRIANGLES)&&!defined(FEATHER_ATLAS_BLIT)
e c me(z2 P I3){
#ifdef ENABLE_FEATHER
if(ENABLE_FEATHER&&Sb(P))return z4(P d1);else
#endif
return min(P.x,P.y);}e c ne(z2 P I3){
#if defined(ENABLE_FEATHER)
if(ENABLE_FEATHER&&Tb(P))return e8(P d1);else
#endif
return P.x;}e c sb(z2 P I3){if(V5(P))return me(P d1);else return ne(P d1);}e c Xh(c T4,z2 P I3){if(V5(P)){c v0=me(P d1);return max(v0,T4);}else{c v0=ne(P d1);return T4+v0;}}
#endif
#endif
