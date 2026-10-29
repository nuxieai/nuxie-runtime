#undef T2
#ifdef ENABLE_FEATHER
#define T2 e
#else
#define T2 E
#endif
#ifdef VERTEX
d1(e0)
#if defined(DRAW_INTERIOR_TRIANGLES)||defined(FEATHER_ATLAS_BLIT)
K(0,j4,LB);
#else
K(0,e,YB);K(1,e,ZB);
#endif
e1
#endif
n2 H0 W(0,e,M0);
#ifdef FEATHER_ATLAS_BLIT
H0 W(1,c,W2);
#elif!defined(RENDER_MODE_DEPTH_STENCIL)
#ifdef DRAW_INTERIOR_TRIANGLES
OPTIONALLY_FLAT W(1,d,o1);
#else
H0 W(2,T2,S);
#endif
OPTIONALLY_FLAT W(3,d,I0);
#endif
#ifdef ENABLE_CLIPPING
#ifdef FEATHER_ATLAS_BLIT
OPTIONALLY_FLAT W(4,d,h4);
#else
OPTIONALLY_FLAT W(4,E,l2);
#endif
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
H0 W(5,e,V0);
#endif
#ifdef ENABLE_ADVANCED_BLEND
OPTIONALLY_FLAT W(6,d,R0);
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
i3 W(7,U0,B3);W(8,c,P4);
#endif
#ifdef ENABLE_MODULATED_IMAGE
H0 W(9,M,N0);
#endif
M1
#ifdef VERTEX
n1(TB,e0,A,C,r){
#if defined(DRAW_INTERIOR_TRIANGLES)||defined(FEATHER_ATLAS_BLIT)
L(C,A,LB,M);
#else
L(C,A,YB,e);L(C,A,ZB,e);
#endif
V(M0,e);
#if defined(ENABLE_MODULATED_IMAGE)
V(N0,M);
#endif
#ifdef FEATHER_ATLAS_BLIT
V(W2,c);
#elif!defined(RENDER_MODE_DEPTH_STENCIL)
#ifdef DRAW_INTERIOR_TRIANGLES
V(o1,d);
#else
V(S,T2);
#endif
V(I0,d);
#endif
#ifdef ENABLE_CLIPPING
#ifdef FEATHER_ATLAS_BLIT
V(h4,d);
#else
V(l2,E);
#endif
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
V(V0,e);
#endif
#ifdef ENABLE_ADVANCED_BLEND
V(R0,d);
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
V(B3,U0);V(P4,c);
#endif
bool Hf=false;uint g0;c i0;
#ifdef RENDER_MODE_DEPTH_STENCIL
O G6;
#endif
#ifdef FEATHER_ATLAS_BLIT
i0=fd(LB,g0,
#ifdef RENDER_MODE_DEPTH_STENCIL
G6,
#endif
W2 T3);
#elif defined(DRAW_INTERIOR_TRIANGLES)
i0=gd(LB,g0
#ifdef RENDER_MODE_DEPTH_STENCIL
,G6
#else
,o1
#endif
T3);
#else
e T;Hf=!sa(YB,ZB,r,g0,i0
#ifndef RENDER_MODE_DEPTH_STENCIL
,T
#else
,G6
#endif
T3);
#ifndef RENDER_MODE_DEPTH_STENCIL
#ifdef ENABLE_FEATHER
S=T;
#else
S.xy=E8(T.xy);
#endif
#endif
#endif
U0 r1=n0(JB,g0*4u+1u).zw;
#if!defined(FEATHER_ATLAS_BLIT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
I0=i9(g0,j.w6);if((r1.x&Na)!=0u) I0=-I0;
#endif
uint y2=r1.x&0xfu;
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING){uint Wj=(y2==M8?r1.y:r1.x)>>16;d B1=i9(Wj,j.w6);if(y2==M8) B1=-B1;
#ifdef FEATHER_ATLAS_BLIT
h4=B1;
#else
l2.x=B1;
#endif
}
#endif
#ifdef ENABLE_ADVANCED_BLEND
if(ENABLE_ADVANCED_BLEND){R0=float((r1.x>>4)&0xfu);}
#endif
c l0=i0;
#ifdef ENABLE_RENDER_TARGET_BOTTOM_UP
if(j.Ea!=0u){l0.y=float(j.Fa)-l0.y;}
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){X L3=p1(n0(KB,g0*p2+2u));e a4=n0(KB,g0*p2+3u);
#ifndef RENDER_MODE_DEPTH_STENCIL
V0=G8(L3,a4.xy,l0);
#else
qb(L3,a4.xy,l0 E4);
#endif
}
#endif
if(y2==Oa){M0=e(unpackUnorm4x8(r1.y));}
#if defined(ENABLE_CLIPPING)&&!defined(FEATHER_ATLAS_BLIT)
else if(ENABLE_CLIPPING&&y2==M8){d f6=i9(r1.x>>16,j.w6);l2.y=f6;}
#endif
else{X Mb=p1(n0(KB,g0*p2));e R7=n0(KB,g0*p2+1u);float r4=uintBitsToFloat(r1.y);M0=Ga(l0,Mb,R7.xy,R7.zw,y2,r4,1.0);}
#if defined(ENABLE_MODULATED_IMAGE)
if(ENABLE_MODULATED_IMAGE&&(r1.x&ie)!=0u){X Nb=p1(n0(KB,g0*p2+4u));e S7=n0(KB,g0*p2+5u);c x3=D0(Nb,l0)+S7.xy;float T7=1.+S7.z;if((r1.x&je)!=0u){uint E3=(r1.x&le)>>ke;T7=-(1.+float(E3));}N0=M(x3.x,x3.y,T7);}else{N0=M(0.0,0.0,0.0);}
#endif
e I;if(!Hf){I=U3(i0);
#ifdef POST_INVERT_Y
I.y=-I.y;
#endif
#ifdef RENDER_MODE_DEPTH_STENCIL
I.z=j9(G6,0xffu);
#elif defined(RENDER_MODE_CLOCKWISE_ATOMIC)
R q5=n0(JB,g0*4u+3u);B3=q5.xy;P4=i0+uintBitsToFloat(q5.zw);
#endif
}else{I=e(j.j3,j.j3,j.j3,j.j3);}a0(M0);
#if defined(ENABLE_MODULATED_IMAGE)
a0(N0);
#endif
#ifdef FEATHER_ATLAS_BLIT
a0(W2);
#elif!defined(RENDER_MODE_DEPTH_STENCIL)
#ifdef DRAW_INTERIOR_TRIANGLES
a0(o1);
#else
a0(S);
#endif
a0(I0);
#endif
#ifdef ENABLE_CLIPPING
#ifdef FEATHER_ATLAS_BLIT
a0(h4);
#else
a0(l2);
#endif
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
a0(V0);
#endif
#ifdef ENABLE_ADVANCED_BLEND
a0(R0);
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
a0(B3);a0(P4);
#endif
A1(I);}
#endif
#ifdef FRAGMENT
l4 n4 f i x8(
#ifdef ENABLE_MODULATED_IMAGE
M y8,
#endif
#ifdef ENABLE_ADVANCED_BLEND
O Z1,
#endif
e ja l7){
#ifdef ENABLE_ADVANCED_BLEND
bool H2=ENABLE_ADVANCED_BLEND&&Z1!=X3;
#else
const bool H2=false;
#endif
i k;if(ja.w>=.0){k=Y4(ja);}else{c Wa=Xa(ja,j.O8,j.P8);k=q2(YC,Q8,Wa,.0);if(!H2){d r4=ve(ja);k.xyz*=k.w;k.w*=r4;}}
#if defined(ENABLE_MODULATED_IMAGE)
if(ENABLE_MODULATED_IMAGE&&y8.z<0.0){return e5(RB,v4,y8.xy,C0(.0));}if(ENABLE_MODULATED_IMAGE&&y8.z>0.0){d Ob=y8.z-1.;i R1=e5(RB,v4,y8.xy,Ob);if(H2) R1=E0(j6(R1),R1.w);k*=R1;}
#endif
return k;}
#if!defined(DRAW_INTERIOR_TRIANGLES)&&!defined(FEATHER_ATLAS_BLIT)
f d If(T2 T e4){
#ifdef ENABLE_FEATHER
if(ENABLE_FEATHER&&od(T)) return X4(T m1);else
#endif
return min(T.x,T.y);}f d Jf(T2 T e4){
#if defined(ENABLE_FEATHER)
if(ENABLE_FEATHER&&pd(T)) return T8(T m1);else
#endif
return T.x;}f d Pc(T2 T e4){if(o6(T)) return If(T m1);else return Jf(T m1);}f d Xj(d r5,T2 T e4){if(o6(T)){d B0=If(T m1);return max(B0,r5);}else{d B0=Jf(T m1);return r5+B0;}}
#endif
#endif
