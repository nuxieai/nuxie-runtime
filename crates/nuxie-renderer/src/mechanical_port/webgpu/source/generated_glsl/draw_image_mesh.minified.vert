#ifdef VERTEX
c1(w3) K(0,c,QC);d1 c1(K3) K(1,c,RC);d1 c1(A1) K(M9,f,YB);K(N9,f,SB);K(O9,f,PB);K(P9,uint,ZB);K(Q9,uint,AC);K(R9,uint,BC);K(S9,uint,MC);K(aa,f,IC);d1
#endif
l2 E0 W(0,c,W5);
#ifdef ENABLE_CLIPPING
OPTIONALLY_FLAT W(1,d,Z3);
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
E0 W(2,f,S0);
#endif
OPTIONALLY_FLAT W(3,i,Q1);
#ifdef ENABLE_ADVANCED_BLEND
a3 W(4,R,H1);
#endif
e2
#ifdef VERTEX
k4 l4 U6(RB,w3,x3,K3,L3,A1,h0,G){L(G,x3,QC,c);L(G,L3,RC,c);L(r,h0,YB,f);L(r,h0,SB,f);L(r,h0,PB,f);L(r,h0,ZB,uint);L(r,h0,AC,uint);L(r,h0,BC,uint);L(r,h0,MC,uint);L(r,h0,IC,f);T(W5,c);
#ifdef ENABLE_CLIPPING
T(Z3,d);
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
T(S0,f);
#endif
T(Q1,i);
#ifdef ENABLE_ADVANCED_BLEND
T(H1,R);
#endif
c k0=M0(n1(YB),QC)+PB.xy;W5=RC*IC.zw+IC.xy;
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING){Z3=m6(AC,j.U4);}
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){
#ifndef RENDER_MODE_DEPTH_STENCIL
S0=j8(n1(SB),PB.zw,k0 Z4);
#else
Ha(n1(SB),PB.zw,k0 Z4);
#endif
}
#endif
f I=I3(k0);
#ifdef POST_INVERT_Y
I.y=-I.y;
#endif
#ifdef RENDER_MODE_DEPTH_STENCIL
I.z=J8(MC,0xffu);
#endif
Q1=unpackUnorm4x8(ZB);
#ifdef ENABLE_ADVANCED_BLEND
H1=P1(BC);
#endif
Z(W5);
#ifdef ENABLE_CLIPPING
Z(Z3);
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
Z(S0);
#endif
Z(Q1);
#ifdef ENABLE_ADVANCED_BLEND
Z(H1);
#endif
w1(I);}
#endif
