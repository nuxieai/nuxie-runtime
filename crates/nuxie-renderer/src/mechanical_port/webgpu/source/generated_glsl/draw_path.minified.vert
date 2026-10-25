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
m2 F0 W(0,e,P0);
#ifdef FEATHER_ATLAS_BLIT
F0 W(1,c,U2);
#elif!defined(RENDER_MODE_DEPTH_STENCIL)
#ifdef DRAW_INTERIOR_TRIANGLES
OPTIONALLY_FLAT W(1,d,p1);
#else
F0 W(2,R2,T);
#endif
OPTIONALLY_FLAT W(3,d,G0);
#endif
#ifdef ENABLE_CLIPPING
#ifdef FEATHER_ATLAS_BLIT
OPTIONALLY_FLAT W(4,d,h4);
#else
OPTIONALLY_FLAT W(4,E,k2);
#endif
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
F0 W(5,e,W0);
#endif
#ifdef ENABLE_ADVANCED_BLEND
OPTIONALLY_FLAT W(6,d,Q0);
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
h3 W(7,R0,A3);W(8,c,N4);
#endif
#ifdef ENABLE_MODULATED_IMAGE
F0 W(9,M,V0);
#endif
M1
#ifdef VERTEX
o1(SB,e0,A,C,r){
#if defined(DRAW_INTERIOR_TRIANGLES)||defined(FEATHER_ATLAS_BLIT)
L(C,A,LB,M);
#else
L(C,A,YB,e);L(C,A,ZB,e);
#endif
V(P0,e);
#if defined(ENABLE_MODULATED_IMAGE)
V(V0,M);
#endif
#ifdef FEATHER_ATLAS_BLIT
V(U2,c);
#elif!defined(RENDER_MODE_DEPTH_STENCIL)
#ifdef DRAW_INTERIOR_TRIANGLES
V(p1,d);
#else
V(T,R2);
#endif
V(G0,d);
#endif
#ifdef ENABLE_CLIPPING
#ifdef FEATHER_ATLAS_BLIT
V(h4,d);
#else
V(k2,E);
#endif
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
V(W0,e);
#endif
#ifdef ENABLE_ADVANCED_BLEND
V(Q0,d);
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
V(A3,R0);V(N4,c);
#endif
bool zf=false;uint f0;c i0;
#ifdef RENDER_MODE_DEPTH_STENCIL
P I6;
#endif
#ifdef FEATHER_ATLAS_BLIT
i0=bd(LB,f0,
#ifdef RENDER_MODE_DEPTH_STENCIL
I6,
#endif
U2 S3);
#elif defined(DRAW_INTERIOR_TRIANGLES)
i0=cd(LB,f0
#ifdef RENDER_MODE_DEPTH_STENCIL
,I6
#else
,p1
#endif
S3);
#else
e U;zf=!ra(YB,ZB,r,f0,i0
#ifndef RENDER_MODE_DEPTH_STENCIL
,U
#else
,I6
#endif
S3);
#ifndef RENDER_MODE_DEPTH_STENCIL
#ifdef ENABLE_FEATHER
T=U;
#else
T.xy=E8(U.xy);
#endif
#endif
#endif
R0 S0=y5(XC,f0);
#if!defined(FEATHER_ATLAS_BLIT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
G0=j9(f0,j.z6);if((S0.x&La)!=0u) G0=-G0;
#endif
uint w2=S0.x&0xfu;
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING){uint Qj=(w2==M8?S0.y:S0.x)>>16;d B1=j9(Qj,j.z6);if(w2==M8) B1=-B1;
#ifdef FEATHER_ATLAS_BLIT
h4=B1;
#else
k2.x=B1;
#endif
}
#endif
#ifdef ENABLE_ADVANCED_BLEND
if(ENABLE_ADVANCED_BLEND){Q0=float((S0.x>>4)&0xfu);}
#endif
c l0=i0;
#ifdef ENABLE_RENDER_TARGET_BOTTOM_UP
if(j.Da!=0u){l0.y=float(j.Ea)-l0.y;}
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){X J3=q1(q0(JB,f0*o2+2u));e Z3=q0(JB,f0*o2+3u);
#ifndef RENDER_MODE_DEPTH_STENCIL
W0=G8(J3,Z3.xy,l0);
#else
ob(J3,Z3.xy,l0 C4);
#endif
}
#endif
if(w2==Ma){P0=e(unpackUnorm4x8(S0.y));}
#if defined(ENABLE_CLIPPING)&&!defined(FEATHER_ATLAS_BLIT)
else if(ENABLE_CLIPPING&&w2==M8){d h6=j9(S0.x>>16,j.z6);k2.y=h6;}
#endif
else{X Kb=q1(q0(JB,f0*o2));e U7=q0(JB,f0*o2+1u);float q4=uintBitsToFloat(S0.y);P0=Fa(l0,Kb,U7.xy,U7.zw,w2,q4,1.0);}
#if defined(ENABLE_MODULATED_IMAGE)
if(ENABLE_MODULATED_IMAGE&&(S0.x&ee)!=0u){X Lb=q1(q0(JB,f0*o2+4u));e V7=q0(JB,f0*o2+5u);c w3=C0(Lb,l0)+V7.xy;float Af=1.+V7.z;if((S0.x&Ah)!=0u){uint j4=(S0.x&Ch)>>Bh;Af=-(1.+float(j4));}V0=M(w3.x,w3.y,Af);}else{V0=M(0.0,0.0,0.0);}
#endif
e I;if(!zf){I=T3(i0);
#ifdef POST_INVERT_Y
I.y=-I.y;
#endif
#ifdef RENDER_MODE_DEPTH_STENCIL
I.z=k9(I6,0xffu);
#elif defined(RENDER_MODE_CLOCKWISE_ATOMIC)
O p5=q0(KB,f0*4u+3u);A3=p5.xy;N4=i0+uintBitsToFloat(p5.zw);
#endif
}else{I=e(j.i3,j.i3,j.i3,j.i3);}a0(P0);
#if defined(ENABLE_MODULATED_IMAGE)
a0(V0);
#endif
#ifdef FEATHER_ATLAS_BLIT
a0(U2);
#elif!defined(RENDER_MODE_DEPTH_STENCIL)
#ifdef DRAW_INTERIOR_TRIANGLES
a0(p1);
#else
a0(T);
#endif
a0(G0);
#endif
#ifdef ENABLE_CLIPPING
#ifdef FEATHER_ATLAS_BLIT
a0(h4);
#else
a0(k2);
#endif
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
a0(W0);
#endif
#ifdef ENABLE_ADVANCED_BLEND
a0(Q0);
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
a0(A3);a0(N4);
#endif
A1(I);}
#endif
#ifdef FRAGMENT
m4 n4 f d Rj(i Kc,uint j4){d Bf=dot(Kc.xyz,Z0(.30,.59,.11));if(j4==Dh) return Kc.w;if(j4==Eh) return 1.-Kc.w;if(j4==Fh) return Bf;return 1.-Bf;}f i x8(
#ifdef ENABLE_MODULATED_IMAGE
M y8,
#endif
#ifdef ENABLE_ADVANCED_BLEND
P Z1,
#endif
e ia o7){
#ifdef ENABLE_ADVANCED_BLEND
bool G2=ENABLE_ADVANCED_BLEND&&Z1!=W3;
#else
const bool G2=false;
#endif
i k;if(ia.w>=.0){k=X4(ia);}else{c Ua=Va(ia,j.O8,j.P8);k=p2(ZC,Q8,Ua,.0);if(!G2){d q4=pe(ia);k.xyz*=k.w;k.w*=q4;}}
#if defined(ENABLE_MODULATED_IMAGE)
if(ENABLE_MODULATED_IMAGE&&y8.z<0.0){return F5(UB,W4,y8.xy,J0(.0));}if(ENABLE_MODULATED_IMAGE&&y8.z>0.0){d Mb=y8.z-1.;i R1=F5(UB,W4,y8.xy,Mb);if(G2) R1=H0(l6(R1),R1.w);k*=R1;}
#endif
return k;}
#if!defined(DRAW_INTERIOR_TRIANGLES)&&!defined(FEATHER_ATLAS_BLIT)
f d Cf(R2 U e4){
#ifdef ENABLE_FEATHER
if(ENABLE_FEATHER&&kd(U)) return V4(U n1);else
#endif
return min(U.x,U.y);}f d Df(R2 U e4){
#if defined(ENABLE_FEATHER)
if(ENABLE_FEATHER&&ld(U)) return T8(U n1);else
#endif
return U.x;}f d Lc(R2 U e4){if(r6(U)) return Cf(U n1);else return Df(U n1);}f d Sj(d q5,R2 U e4){if(r6(U)){d B0=Cf(U n1);return max(B0,q5);}else{d B0=Df(U n1);return q5+B0;}}
#endif
#endif
