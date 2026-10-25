#undef R2
#ifdef ENABLE_FEATHER
#define R2 e
#else
#define R2 E
#endif
#ifdef VERTEX
e1(e0)
#if defined(DRAW_INTERIOR_TRIANGLES)||defined(FEATHER_ATLAS_BLIT)
K(0,k4,LB);
#else
K(0,e,YB);K(1,e,ZB);
#endif
f1
#endif
m2 H0 W(0,e,N0);
#ifdef FEATHER_ATLAS_BLIT
H0 W(1,c,U2);
#elif!defined(RENDER_MODE_DEPTH_STENCIL)
#ifdef DRAW_INTERIOR_TRIANGLES
OPTIONALLY_FLAT W(1,d,p1);
#else
H0 W(2,R2,T);
#endif
OPTIONALLY_FLAT W(3,d,I0);
#endif
#ifdef ENABLE_CLIPPING
#ifdef FEATHER_ATLAS_BLIT
OPTIONALLY_FLAT W(4,d,i4);
#else
OPTIONALLY_FLAT W(4,E,k2);
#endif
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
H0 W(5,e,W0);
#endif
#ifdef ENABLE_ADVANCED_BLEND
OPTIONALLY_FLAT W(6,d,S0);
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
h3 W(7,T0,A3);W(8,c,P4);
#endif
#ifdef ENABLE_MODULATED_IMAGE
H0 W(9,M,O0);
#endif
M1
#ifdef VERTEX
o1(TB,e0,A,C,r){
#if defined(DRAW_INTERIOR_TRIANGLES)||defined(FEATHER_ATLAS_BLIT)
L(C,A,LB,M);
#else
L(C,A,YB,e);L(C,A,ZB,e);
#endif
V(N0,e);
#if defined(ENABLE_MODULATED_IMAGE)
V(O0,M);
#endif
#ifdef FEATHER_ATLAS_BLIT
V(U2,c);
#elif!defined(RENDER_MODE_DEPTH_STENCIL)
#ifdef DRAW_INTERIOR_TRIANGLES
V(p1,d);
#else
V(T,R2);
#endif
V(I0,d);
#endif
#ifdef ENABLE_CLIPPING
#ifdef FEATHER_ATLAS_BLIT
V(i4,d);
#else
V(k2,E);
#endif
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
V(W0,e);
#endif
#ifdef ENABLE_ADVANCED_BLEND
V(S0,d);
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
V(A3,T0);V(P4,c);
#endif
bool Hf=false;uint f0;c i0;
#ifdef RENDER_MODE_DEPTH_STENCIL
P I6;
#endif
#ifdef FEATHER_ATLAS_BLIT
i0=ed(LB,f0,
#ifdef RENDER_MODE_DEPTH_STENCIL
I6,
#endif
U2 T3);
#elif defined(DRAW_INTERIOR_TRIANGLES)
i0=fd(LB,f0
#ifdef RENDER_MODE_DEPTH_STENCIL
,I6
#else
,p1
#endif
T3);
#else
e U;Hf=!sa(YB,ZB,r,f0,i0
#ifndef RENDER_MODE_DEPTH_STENCIL
,U
#else
,I6
#endif
T3);
#ifndef RENDER_MODE_DEPTH_STENCIL
#ifdef ENABLE_FEATHER
T=U;
#else
T.xy=G8(U.xy);
#endif
#endif
#endif
T0 L0=z5(XC,f0);
#if!defined(FEATHER_ATLAS_BLIT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
I0=k9(f0,j.z6);if((L0.x&Ma)!=0u) I0=-I0;
#endif
uint w2=L0.x&0xfu;
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING){uint Wj=(w2==O8?L0.y:L0.x)>>16;d B1=k9(Wj,j.z6);if(w2==O8) B1=-B1;
#ifdef FEATHER_ATLAS_BLIT
i4=B1;
#else
k2.x=B1;
#endif
}
#endif
#ifdef ENABLE_ADVANCED_BLEND
if(ENABLE_ADVANCED_BLEND){S0=float((L0.x>>4)&0xfu);}
#endif
c l0=i0;
#ifdef ENABLE_RENDER_TARGET_BOTTOM_UP
if(j.Ea!=0u){l0.y=float(j.Fa)-l0.y;}
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){X K3=q1(q0(JB,f0*o2+2u));e a4=q0(JB,f0*o2+3u);
#ifndef RENDER_MODE_DEPTH_STENCIL
W0=I8(K3,a4.xy,l0);
#else
pb(K3,a4.xy,l0 E4);
#endif
}
#endif
if(w2==Na){N0=e(unpackUnorm4x8(L0.y));}
#if defined(ENABLE_CLIPPING)&&!defined(FEATHER_ATLAS_BLIT)
else if(ENABLE_CLIPPING&&w2==O8){d h6=k9(L0.x>>16,j.z6);k2.y=h6;}
#endif
else{X Lb=q1(q0(JB,f0*o2));e U7=q0(JB,f0*o2+1u);float r4=uintBitsToFloat(L0.y);N0=Ga(l0,Lb,U7.xy,U7.zw,w2,r4,1.0);}
#if defined(ENABLE_MODULATED_IMAGE)
if(ENABLE_MODULATED_IMAGE&&(L0.x&he)!=0u){X Mb=q1(q0(JB,f0*o2+4u));e V7=q0(JB,f0*o2+5u);c w3=D0(Mb,l0)+V7.xy;float W7=1.+V7.z;if((L0.x&ie)!=0u){uint D3=(L0.x&ke)>>je;W7=-(1.+float(D3));}O0=M(w3.x,w3.y,W7);}else{O0=M(0.0,0.0,0.0);}
#endif
e I;if(!Hf){I=U3(i0);
#ifdef POST_INVERT_Y
I.y=-I.y;
#endif
#ifdef RENDER_MODE_DEPTH_STENCIL
I.z=l9(I6,0xffu);
#elif defined(RENDER_MODE_CLOCKWISE_ATOMIC)
O r5=q0(KB,f0*4u+3u);A3=r5.xy;P4=i0+uintBitsToFloat(r5.zw);
#endif
}else{I=e(j.i3,j.i3,j.i3,j.i3);}a0(N0);
#if defined(ENABLE_MODULATED_IMAGE)
a0(O0);
#endif
#ifdef FEATHER_ATLAS_BLIT
a0(U2);
#elif!defined(RENDER_MODE_DEPTH_STENCIL)
#ifdef DRAW_INTERIOR_TRIANGLES
a0(p1);
#else
a0(T);
#endif
a0(I0);
#endif
#ifdef ENABLE_CLIPPING
#ifdef FEATHER_ATLAS_BLIT
a0(i4);
#else
a0(k2);
#endif
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
a0(W0);
#endif
#ifdef ENABLE_ADVANCED_BLEND
a0(S0);
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
a0(A3);a0(P4);
#endif
A1(I);}
#endif
#ifdef FRAGMENT
m4 n4 f i z8(
#ifdef ENABLE_MODULATED_IMAGE
M A8,
#endif
#ifdef ENABLE_ADVANCED_BLEND
P Z1,
#endif
e ja n7){
#ifdef ENABLE_ADVANCED_BLEND
bool G2=ENABLE_ADVANCED_BLEND&&Z1!=X3;
#else
const bool G2=false;
#endif
i k;if(ja.w>=.0){k=Y4(ja);}else{c Va=Wa(ja,j.Q8,j.R8);k=p2(ZC,S8,Va,.0);if(!G2){d r4=ve(ja);k.xyz*=k.w;k.w*=r4;}}
#if defined(ENABLE_MODULATED_IMAGE)
if(ENABLE_MODULATED_IMAGE&&A8.z<0.0){return e5(RB,v4,A8.xy,C0(.0));}if(ENABLE_MODULATED_IMAGE&&A8.z>0.0){d Nb=A8.z-1.;i R1=e5(RB,v4,A8.xy,Nb);if(G2) R1=E0(l6(R1),R1.w);k*=R1;}
#endif
return k;}
#if!defined(DRAW_INTERIOR_TRIANGLES)&&!defined(FEATHER_ATLAS_BLIT)
f d If(R2 U f4){
#ifdef ENABLE_FEATHER
if(ENABLE_FEATHER&&nd(U)) return X4(U n1);else
#endif
return min(U.x,U.y);}f d Jf(R2 U f4){
#if defined(ENABLE_FEATHER)
if(ENABLE_FEATHER&&od(U)) return V8(U n1);else
#endif
return U.x;}f d Oc(R2 U f4){if(r6(U)) return If(U n1);else return Jf(U n1);}f d Xj(d v5,R2 U f4){if(r6(U)){d B0=If(U n1);return max(B0,v5);}else{d B0=Jf(U n1);return v5+B0;}}
#endif
#endif
