#undef T2
#ifdef ENABLE_FEATHER
#define T2 e
#else
#define T2 F
#endif
#ifdef VERTEX
f1(d0)
#if defined(DRAW_INTERIOR_TRIANGLES)||defined(FEATHER_ATLAS_BLIT)
K(0,l4,MB);
#else
K(0,e,YB);K(1,e,ZB);
#endif
g1
#endif
n2 I0 W(0,e,N0);
#ifdef FEATHER_ATLAS_BLIT
I0 W(1,c,W2);
#elif!defined(RENDER_MODE_DEPTH_STENCIL)
#ifdef DRAW_INTERIOR_TRIANGLES
OPTIONALLY_FLAT W(1,d,r1);
#else
I0 W(2,T2,S);
#endif
OPTIONALLY_FLAT W(3,d,J0);
#endif
#ifdef ENABLE_CLIPPING
#ifdef FEATHER_ATLAS_BLIT
OPTIONALLY_FLAT W(4,d,j4);
#else
OPTIONALLY_FLAT W(4,F,l2);
#endif
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
I0 W(5,e,Y0);
#endif
#ifdef ENABLE_ADVANCED_BLEND
OPTIONALLY_FLAT W(6,d,S0);
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
m3 W(7,R0,D3);W(8,c,V4);
#endif
#ifdef ENABLE_MODULATED_IMAGE
I0 W(9,M,O0);
#endif
P1
#ifdef VERTEX
l1(UB,d0,A,C,r){
#if defined(DRAW_INTERIOR_TRIANGLES)||defined(FEATHER_ATLAS_BLIT)
L(C,A,MB,M);
#else
L(C,A,YB,e);L(C,A,ZB,e);
#endif
V(N0,e);
#if defined(ENABLE_MODULATED_IMAGE)
V(O0,M);
#endif
#ifdef FEATHER_ATLAS_BLIT
V(W2,c);
#elif!defined(RENDER_MODE_DEPTH_STENCIL)
#ifdef DRAW_INTERIOR_TRIANGLES
V(r1,d);
#else
V(S,T2);
#endif
V(J0,d);
#endif
#ifdef ENABLE_CLIPPING
#ifdef FEATHER_ATLAS_BLIT
V(j4,d);
#else
V(l2,F);
#endif
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
V(Y0,e);
#endif
#ifdef ENABLE_ADVANCED_BLEND
V(S0,d);
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
V(D3,R0);V(V4,c);
#endif
bool Pf=false;uint g0;c i0;
#ifdef RENDER_MODE_DEPTH_STENCIL
O I6;
#endif
#ifdef FEATHER_ATLAS_BLIT
i0=jd(MB,g0,
#ifdef RENDER_MODE_DEPTH_STENCIL
I6,
#endif
W2 T3);
#elif defined(DRAW_INTERIOR_TRIANGLES)
i0=kd(MB,g0
#ifdef RENDER_MODE_DEPTH_STENCIL
,I6
#else
,r1
#endif
T3);
#else
e T;Pf=!va(YB,ZB,r,g0,i0
#ifndef RENDER_MODE_DEPTH_STENCIL
,T
#else
,I6
#endif
T3);
#ifndef RENDER_MODE_DEPTH_STENCIL
#ifdef ENABLE_FEATHER
S=T;
#else
S.xy=J8(T.xy);
#endif
#endif
#endif
R0 x1=o0(JB,g0*4u+1u).zw;
#if!defined(FEATHER_ATLAS_BLIT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
J0=n9(g0,j.z6);if((x1.x&Qa)!=0u) J0=-J0;
#endif
uint A2=x1.x&0xfu;
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING){uint hk=(A2==R8?x1.y:x1.x)>>16;d C1=n9(hk,j.z6);if(A2==R8) C1=-C1;
#ifdef FEATHER_ATLAS_BLIT
j4=C1;
#else
l2.x=C1;
#endif
}
#endif
#ifdef ENABLE_ADVANCED_BLEND
if(ENABLE_ADVANCED_BLEND){S0=float((x1.x>>4)&0xfu);}
#endif
c m0=i0;
#ifdef ENABLE_RENDER_TARGET_BOTTOM_UP
if(j.Ha!=0u){m0.y=float(j.Ia)-m0.y;}
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){X N3=v1(o0(KB,g0*p2+2u));e a4=o0(KB,g0*p2+3u);
#ifndef RENDER_MODE_DEPTH_STENCIL
Y0=L8(N3,a4.xy,m0);
#else
tb(N3,a4.xy,m0 H4);
#endif
}
#endif
if(A2==Ra){N0=e(unpackUnorm4x8(x1.y));}
#if defined(ENABLE_CLIPPING)&&!defined(FEATHER_ATLAS_BLIT)
else if(ENABLE_CLIPPING&&A2==R8){d i6=n9(x1.x>>16,j.z6);l2.y=i6;}
#endif
else{X Pb=v1(o0(KB,g0*p2));e U7=o0(KB,g0*p2+1u);float w4=uintBitsToFloat(x1.y);N0=Ja(m0,Pb,U7.xy,U7.zw,A2,w4,1.0);}
#if defined(ENABLE_MODULATED_IMAGE)
if(ENABLE_MODULATED_IMAGE&&(x1.x&le)!=0u){X Qb=v1(o0(KB,g0*p2+4u));e V7=o0(KB,g0*p2+5u);c z3=F0(Qb,m0)+V7.xy;float W7=1.+V7.z;if((x1.x&me)!=0u){uint G3=(x1.x&oe)>>ne;W7=-(1.+float(G3));}O0=M(z3.x,z3.y,W7);}else{O0=M(0.0,0.0,0.0);}
#endif
e I;if(!Pf){I=U3(i0);
#ifdef POST_INVERT_Y
I.y=-I.y;
#endif
#ifdef RENDER_MODE_DEPTH_STENCIL
I.z=o9(I6,0xffu);
#elif defined(RENDER_MODE_CLOCKWISE_ATOMIC)
R A5=o0(JB,g0*4u+3u);D3=A5.xy;V4=i0+uintBitsToFloat(A5.zw);
#endif
}else{I=e(j.n3,j.n3,j.n3,j.n3);}e0(N0);
#if defined(ENABLE_MODULATED_IMAGE)
e0(O0);
#endif
#ifdef FEATHER_ATLAS_BLIT
e0(W2);
#elif!defined(RENDER_MODE_DEPTH_STENCIL)
#ifdef DRAW_INTERIOR_TRIANGLES
e0(r1);
#else
e0(S);
#endif
e0(J0);
#endif
#ifdef ENABLE_CLIPPING
#ifdef FEATHER_ATLAS_BLIT
e0(j4);
#else
e0(l2);
#endif
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
e0(Y0);
#endif
#ifdef ENABLE_ADVANCED_BLEND
e0(S0);
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
e0(D3);e0(V4);
#endif
B1(I);}
#endif
#ifdef FRAGMENT
n4 p4 f i C8(
#ifdef ENABLE_MODULATED_IMAGE
M D8,
#endif
#ifdef ENABLE_ADVANCED_BLEND
O c2,
#endif
e ma n7){
#ifdef ENABLE_ADVANCED_BLEND
bool J2=ENABLE_ADVANCED_BLEND&&c2!=X3;
#else
const bool J2=false;
#endif
i k;if(ma.w>=.0){k=y4(ma);}else{c Za=ab(ma,j.T8,j.U8);k=q2(YC,V8,Za,.0);if(!J2){d w4=ze(ma);k.xyz*=k.w;k.w*=w4;}}
#if defined(ENABLE_MODULATED_IMAGE)
if(ENABLE_MODULATED_IMAGE&&D8.z<0.0){return j5(SB,x4,D8.xy,A0(.0));}if(ENABLE_MODULATED_IMAGE&&D8.z>0.0){d Rb=D8.z-1.;i T1=j5(SB,x4,D8.xy,Rb);if(J2) T1=z0(m6(T1),T1.w);k*=T1;}
#endif
return k;}
#if!defined(DRAW_INTERIOR_TRIANGLES)&&!defined(FEATHER_ATLAS_BLIT)
f d Qf(T2 T g4){
#ifdef ENABLE_FEATHER
if(ENABLE_FEATHER&&sd(T)) return e5(T p1);else
#endif
return min(T.x,T.y);}f d Rf(T2 T g4){
#if defined(ENABLE_FEATHER)
if(ENABLE_FEATHER&&td(T)) return Y8(T p1);else
#endif
return T.x;}f d Tc(T2 T g4){if(r6(T)) return Qf(T p1);else return Rf(T p1);}f d ik(d B5,T2 T g4){if(r6(T)){d E0=Qf(T p1);return max(E0,B5);}else{d E0=Rf(T p1);return B5+E0;}}
#endif
#endif
