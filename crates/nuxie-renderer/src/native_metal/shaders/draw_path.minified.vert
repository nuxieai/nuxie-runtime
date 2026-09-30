#undef B2
#ifdef ENABLE_FEATHER
#define B2 f
#else
#define B2 E
#endif
#ifdef VERTEX
h1(g0)
#if defined(DRAW_INTERIOR_TRIANGLES)||defined(FEATHER_ATLAS_BLIT)
K(0,R3,JB);
#else
K(0,f,UB);K(1,f,VB);
#endif
i1
#endif
q2 I0 W(0,f,X1);
#ifdef FEATHER_ATLAS_BLIT
I0 W(1,c,F2);
#elif!defined(RENDER_MODE_DEPTH_STENCIL)
#ifdef DRAW_INTERIOR_TRIANGLES
OPTIONALLY_FLAT W(1,d,j1);
#else
I0 W(2,B2,O);
#endif
OPTIONALLY_FLAT W(3,d,D0);
#endif
#ifdef ENABLE_CLIPPING
#ifdef FEATHER_ATLAS_BLIT
OPTIONALLY_FLAT W(4,d,O3);
#else
OPTIONALLY_FLAT W(4,E,Y1);
#endif
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
I0 W(5,f,O0);
#endif
#ifdef ENABLE_ADVANCED_BLEND
OPTIONALLY_FLAT W(6,d,g1);
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
V2 W(7,N0,k3);W(8,c,v4);
#endif
#ifdef ENABLE_MODULATED_IMAGE
I0 W(9,R,C2);
#endif
i2
#ifdef VERTEX
#ifdef EMULATE_DYNAMIC_COLOR_WRITE_DISABLE
Qd(Fh)Rd(float,qi)Sd(ri)
#endif
B1(EC,g0,F,B,v){
#if defined(DRAW_INTERIOR_TRIANGLES)||defined(FEATHER_ATLAS_BLIT)
L(B,F,JB,R);
#else
L(B,F,UB,f);L(B,F,VB,f);
#endif
U(X1,f);
#if defined(ENABLE_MODULATED_IMAGE)
U(C2,R);
#endif
#ifdef FEATHER_ATLAS_BLIT
U(F2,c);
#elif!defined(RENDER_MODE_DEPTH_STENCIL)
#ifdef DRAW_INTERIOR_TRIANGLES
U(j1,d);
#else
U(O,B2);
#endif
U(D0,d);
#endif
#ifdef ENABLE_CLIPPING
#ifdef FEATHER_ATLAS_BLIT
U(O3,d);
#else
U(Y1,E);
#endif
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
U(O0,f);
#endif
#ifdef ENABLE_ADVANCED_BLEND
U(g1,d);
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
U(k3,N0);U(v4,c);
#endif
bool Ae=false;uint o0;c l0;
#ifdef RENDER_MODE_DEPTH_STENCIL
N k9;
#endif
#ifdef FEATHER_ATLAS_BLIT
l0=Nb(JB,o0,
#ifdef RENDER_MODE_DEPTH_STENCIL
k9,
#endif
F2 A3);
#elif defined(DRAW_INTERIOR_TRIANGLES)
l0=Ob(JB,o0
#ifdef RENDER_MODE_DEPTH_STENCIL
,k9
#else
,j1
#endif
A3);
#else
f P;Ae=!x9(UB,VB,v,o0,l0
#ifndef RENDER_MODE_DEPTH_STENCIL
,P
#else
,k9
#endif
A3);
#ifndef RENDER_MODE_DEPTH_STENCIL
#ifdef ENABLE_FEATHER
O=P;
#else
O.xy=U7(P.xy);
#endif
#endif
#endif
N0 r1=U5(CD,o0);
#if!defined(FEATHER_ATLAS_BLIT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
D0=x8(o0,j.g6);if((r1.x&N9)!=0u)D0=-D0;
#endif
uint X3=r1.x&0xfu;
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING){uint si=(X3==d8?r1.y:r1.x)>>16;d m1=x8(si,j.g6);if(X3==d8)m1=-m1;
#ifdef FEATHER_ATLAS_BLIT
O3=m1;
#else
Y1.x=m1;
#endif
}
#endif
#ifdef ENABLE_ADVANCED_BLEND
if(ENABLE_ADVANCED_BLEND){g1=float((r1.x>>4)&0xfu);}
#endif
c v0=l0;
#ifdef ENABLE_RENDER_TARGET_BOTTOM_UP
if(j.Qb!=0u){v0.y=float(j.Rb)-v0.y;}
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){e0 e4=L1(L0(PB,o0*E3+2u));f L4=L0(PB,o0*E3+3u);
#ifndef RENDER_MODE_DEPTH_STENCIL
O0=W7(e4,L4.xy,v0);
#else
Rc(e4,L4.xy,v0 C5);
#endif
}
#endif
if(X3==Zb){X1=f(unpackUnorm4x8(r1.y));}
#if defined(ENABLE_CLIPPING)&&!defined(FEATHER_ATLAS_BLIT)
else if(ENABLE_CLIPPING&&X3==d8){d M5=x8(r1.x>>16,j.g6);Y1.y=M5;}
#endif
else{e0 ti=L1(L0(PB,o0*E3));f Be=L0(PB,o0*E3+1u);X1=Sb(v0,ti,Be.xy,float(X3),Be.zw,uintBitsToFloat(r1.y));X1.w=-X1.w;}
#ifdef EMULATE_DYNAMIC_COLOR_WRITE_DISABLE
if(EMULATE_DYNAMIC_COLOR_WRITE_DISABLE){X1*=ri.qi;}
#endif
#if defined(ENABLE_MODULATED_IMAGE)
if(ENABLE_MODULATED_IMAGE&&(r1.x&kg)!=0u){e0 ui=L1(L0(PB,o0*E3+4u));f Ce=L0(PB,o0*E3+5u);c k4=P0(ui,v0)+Ce.xy;C2=R(k4.x,k4.y,1.+Ce.z);}else{C2=R(0.0,0.0,0.0);}
#endif
f X;if(!Ae){X=Q3(l0);
#ifdef POST_INVERT_Y
X.y=-X.y;
#endif
#ifdef RENDER_MODE_DEPTH_STENCIL
X.z=qa(k9);
#elif defined(RENDER_MODE_CLOCKWISE_ATOMIC)
Y V4=L0(OB,o0*4u+3u);k3=V4.xy;v4=l0+uintBitsToFloat(V4.zw);
#endif
}else{X=f(j.W2,j.W2,j.W2,j.W2);}c0(X1);
#if defined(ENABLE_MODULATED_IMAGE)
c0(C2);
#endif
#ifdef FEATHER_ATLAS_BLIT
c0(F2);
#elif!defined(RENDER_MODE_DEPTH_STENCIL)
#ifdef DRAW_INTERIOR_TRIANGLES
c0(j1);
#else
c0(O);
#endif
c0(D0);
#endif
#ifdef ENABLE_CLIPPING
#ifdef FEATHER_ATLAS_BLIT
c0(O3);
#else
c0(Y1);
#endif
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
c0(O0);
#endif
#ifdef ENABLE_ADVANCED_BLEND
c0(g1);
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
c0(k3);c0(v4);
#endif
C1(X);}
#endif
#ifdef FRAGMENT
U3 V3 e i O7(
#ifdef ENABLE_MODULATED_IMAGE
R wb,
#endif
#ifdef ENABLE_ADVANCED_BLEND
N p3,
#endif
f W4 O6){
#ifdef ENABLE_ADVANCED_BLEND
bool d5=ENABLE_ADVANCED_BLEND&&p3!=B4;
#else
const bool d5=false;
#endif
i k;if(W4.w>=.0){k=g5(W4);}else{W4.w=-W4.w;d Q9=S3(fract(W4.w)*(256./255.));W4.w=floor(W4.w)*j.dc+j.ec;c W9=jc(W4);k=j2(DD,P9,W9,.0);if(!d5){k.xyz*=k.w;k.w*=Q9;}}
#if defined(ENABLE_MODULATED_IMAGE)
if(ENABLE_MODULATED_IMAGE&&wb.z>0.0){d vi=wb.z-1.;i k2=X6(GC,Z5,wb.xy,vi);if(d5)k2=E0(I6(k2),k2.w);k*=k2;}
#endif
return k;}
#if!defined(DRAW_INTERIOR_TRIANGLES)&&!defined(FEATHER_ATLAS_BLIT)
e d De(B2 P L3){
#ifdef ENABLE_FEATHER
if(ENABLE_FEATHER&&fc(P))return C4(P e1);else
#endif
return min(P.x,P.y);}e d Ee(B2 P L3){
#if defined(ENABLE_FEATHER)
if(ENABLE_FEATHER&&gc(P))return g8(P e1);else
#endif
return P.x;}e d xb(B2 P L3){if(Y5(P))return De(P e1);else return Ee(P e1);}e d wi(d X4,B2 P L3){if(Y5(P)){d y0=De(P e1);return max(y0,X4);}else{d y0=Ee(P e1);return X4+y0;}}
#endif
#endif
