#undef C2
#ifdef ENABLE_FEATHER
#define C2 f
#else
#define C2 D
#endif
#ifdef VERTEX
g1(h0)
#if defined(DRAW_INTERIOR_TRIANGLES)||defined(FEATHER_ATLAS_BLIT)
I(0,Q3,JB);
#else
I(0,f,VB);I(1,f,WB);
#endif
h1
#endif
r2 I0 W(0,f,X1);
#ifdef FEATHER_ATLAS_BLIT
I0 W(1,c,G2);
#elif!defined(RENDER_MODE_DEPTH_STENCIL)
#ifdef DRAW_INTERIOR_TRIANGLES
OPTIONALLY_FLAT W(1,d,i1);
#else
I0 W(2,C2,O);
#endif
OPTIONALLY_FLAT W(3,d,D0);
#endif
#ifdef ENABLE_CLIPPING
#ifdef FEATHER_ATLAS_BLIT
OPTIONALLY_FLAT W(4,d,N3);
#else
OPTIONALLY_FLAT W(4,D,Y1);
#endif
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
I0 W(5,f,P0);
#endif
#ifdef ENABLE_ADVANCED_BLEND
OPTIONALLY_FLAT W(6,d,f1);
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
W2 W(7,O0,k3);W(8,c,w4);
#endif
#ifdef ENABLE_MODULATED_IMAGE
I0 W(9,S,D2);
#endif
i2
#ifdef VERTEX
#ifdef EMULATE_DYNAMIC_COLOR_WRITE_DISABLE
Kd(Dh) Ld(float,ri) Md(si)
#endif
A1(EC,h0,F,A,q){
#if defined(DRAW_INTERIOR_TRIANGLES)||defined(FEATHER_ATLAS_BLIT)
J(A,F,JB,S);
#else
J(A,F,VB,f);J(A,F,WB,f);
#endif
V(X1,f);
#if defined(ENABLE_MODULATED_IMAGE)
V(D2,S);
#endif
#ifdef FEATHER_ATLAS_BLIT
V(G2,c);
#elif!defined(RENDER_MODE_DEPTH_STENCIL)
#ifdef DRAW_INTERIOR_TRIANGLES
V(i1,d);
#else
V(O,C2);
#endif
V(D0,d);
#endif
#ifdef ENABLE_CLIPPING
#ifdef FEATHER_ATLAS_BLIT
V(N3,d);
#else
V(Y1,D);
#endif
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
V(P0,f);
#endif
#ifdef ENABLE_ADVANCED_BLEND
V(f1,d);
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
V(k3,O0);V(w4,c);
#endif
bool ve=false;uint o0;c k0;
#ifdef RENDER_MODE_DEPTH_STENCIL
N h9;
#endif
#ifdef FEATHER_ATLAS_BLIT
k0=Jb(JB,o0,
#ifdef RENDER_MODE_DEPTH_STENCIL
h9,
#endif
G2 A3);
#elif defined(DRAW_INTERIOR_TRIANGLES)
k0=Kb(JB,o0
#ifdef RENDER_MODE_DEPTH_STENCIL
,h9
#else
,i1
#endif
A3);
#else
f P;ve=!r9(VB,WB,q,o0,k0
#ifndef RENDER_MODE_DEPTH_STENCIL
,P
#else
,h9
#endif
A3);
#ifndef RENDER_MODE_DEPTH_STENCIL
#ifdef ENABLE_FEATHER
O=P;
#else
O.xy=Q7(P.xy);
#endif
#endif
#endif
O0 r1=R5(DD,o0);
#if!defined(FEATHER_ATLAS_BLIT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
D0=r8(o0,j.c6);if((r1.x&L9)!=0u) D0=-D0;
#endif
uint W3=r1.x&0xfu;
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING){uint ti=(W3==Y7?r1.y:r1.x)>>16;d l1=r8(ti,j.c6);if(W3==Y7) l1=-l1;
#ifdef FEATHER_ATLAS_BLIT
N3=l1;
#else
Y1.x=l1;
#endif
}
#endif
#ifdef ENABLE_ADVANCED_BLEND
if(ENABLE_ADVANCED_BLEND){f1=float((r1.x>>4)&0xfu);}
#endif
c v0=k0;
#ifdef ENABLE_RENDER_TARGET_BOTTOM_UP
if(j.Mb!=0u){v0.y=float(j.Nb)-v0.y;}
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){Y e4=L1(L0(PB,o0*E3+2u));f K4=L0(PB,o0*E3+3u);
#ifndef RENDER_MODE_DEPTH_STENCIL
P0=S7(e4,K4.xy,v0);
#else
Mc(e4,K4.xy,v0 A5);
#endif
}
#endif
if(W3==Vb){X1=f(unpackUnorm4x8(r1.y));}
#if defined(ENABLE_CLIPPING)&&!defined(FEATHER_ATLAS_BLIT)
else if(ENABLE_CLIPPING&&W3==Y7){d K5=r8(r1.x>>16,j.c6);Y1.y=K5;}
#endif
else{Y ui=L1(L0(PB,o0*E3));f we=L0(PB,o0*E3+1u);X1=Ob(v0,ui,we.xy,float(W3),we.zw,uintBitsToFloat(r1.y));X1.w=-X1.w;}
#ifdef EMULATE_DYNAMIC_COLOR_WRITE_DISABLE
if(EMULATE_DYNAMIC_COLOR_WRITE_DISABLE){X1*=si.ri;}
#endif
#if defined(ENABLE_MODULATED_IMAGE)
if(ENABLE_MODULATED_IMAGE&&(r1.x&ig)!=0u){Y vi=L1(L0(PB,o0*E3+4u));f xe=L0(PB,o0*E3+5u);c l4=N0(vi,v0)+xe.xy;D2=S(l4.x,l4.y,1.+xe.z);}else{D2=S(0.0,0.0,0.0);}
#endif
f X;if(!ve){X=P3(k0);
#ifdef POST_INVERT_Y
X.y=-X.y;
#endif
#ifdef RENDER_MODE_DEPTH_STENCIL
X.z=ma(h9,0xffu);
#elif defined(RENDER_MODE_CLOCKWISE_ATOMIC)
R V4=L0(OB,o0*4u+3u);k3=V4.xy;w4=k0+uintBitsToFloat(V4.zw);
#endif
}else{X=f(j.X2,j.X2,j.X2,j.X2);}c0(X1);
#if defined(ENABLE_MODULATED_IMAGE)
c0(D2);
#endif
#ifdef FEATHER_ATLAS_BLIT
c0(G2);
#elif!defined(RENDER_MODE_DEPTH_STENCIL)
#ifdef DRAW_INTERIOR_TRIANGLES
c0(i1);
#else
c0(O);
#endif
c0(D0);
#endif
#ifdef ENABLE_CLIPPING
#ifdef FEATHER_ATLAS_BLIT
c0(N3);
#else
c0(Y1);
#endif
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
c0(P0);
#endif
#ifdef ENABLE_ADVANCED_BLEND
c0(f1);
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
c0(k3);c0(w4);
#endif
B1(X);}
#endif
#ifdef FRAGMENT
T3 U3 e i K7(
#ifdef ENABLE_MODULATED_IMAGE
S sb,
#endif
#ifdef ENABLE_ADVANCED_BLEND
N p3,
#endif
f W4 M6){
#ifdef ENABLE_ADVANCED_BLEND
bool e5=ENABLE_ADVANCED_BLEND&&p3!=C4;
#else
const bool e5=false;
#endif
i k;if(W4.w>=.0){k=h5(W4);}else{W4.w=-W4.w;d O9=R3(fract(W4.w)*(256./255.));W4.w=floor(W4.w)*j.Zb+j.ac;c U9=fc(W4);k=j2(ED,N9,U9,.0);if(!e5){k.xyz*=k.w;k.w*=O9;}}
#if defined(ENABLE_MODULATED_IMAGE)
if(ENABLE_MODULATED_IMAGE&&sb.z>0.0){d wi=sb.z-1.;i k2=V6(HC,W5,sb.xy,wi);if(e5) k2=E0(G6(k2),k2.w);k*=k2;}
#endif
return k;}
#if!defined(DRAW_INTERIOR_TRIANGLES)&&!defined(FEATHER_ATLAS_BLIT)
e d ye(C2 P K3){
#ifdef ENABLE_FEATHER
if(ENABLE_FEATHER&&bc(P)) return D4(P e1);else
#endif
return min(P.x,P.y);}e d ze(C2 P K3){
#if defined(ENABLE_FEATHER)
if(ENABLE_FEATHER&&cc(P)) return c8(P e1);else
#endif
return P.x;}e d tb(C2 P K3){if(V5(P)) return ye(P e1);else return ze(P e1);}e d xi(d X4,C2 P K3){if(V5(P)){d y0=ye(P e1);return max(y0,X4);}else{d y0=ze(P e1);return X4+y0;}}
#endif
#endif
