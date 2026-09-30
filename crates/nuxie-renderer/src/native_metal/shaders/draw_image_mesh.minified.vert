#ifdef VERTEX
h1(n3)K(0,c,OC);i1 h1(C3)K(1,c,PC);i1 h1(p1)K(y9,f,WB);K(z9,f,RB);K(A9,f,NB);K(B9,uint,XB);K(C9,uint,YB);K(D9,uint,ZB);K(E9,uint,LC);i1
#endif
q2 I0 W(0,c,L5);
#ifdef ENABLE_CLIPPING
OPTIONALLY_FLAT W(1,d,O3);
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
I0 W(2,f,O0);
#endif
OPTIONALLY_FLAT W(3,i,K1);
#ifdef ENABLE_ADVANCED_BLEND
V2 W(4,N,D1);
#endif
i2
#ifdef VERTEX
Y3 Z3 L6(EC,n3,o3,C3,D3,p1,h0,B){L(B,o3,OC,c);L(B,D3,PC,c);L(v,h0,WB,f);L(v,h0,RB,f);L(v,h0,NB,f);L(v,h0,XB,uint);L(v,h0,YB,uint);L(v,h0,ZB,uint);L(v,h0,LC,uint);U(L5,c);
#ifdef ENABLE_CLIPPING
U(O3,d);
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
U(O0,f);
#endif
U(K1,i);
#ifdef ENABLE_ADVANCED_BLEND
U(D1,N);
#endif
c l0=P0(L1(WB),OC)+NB.xy;L5=PC;
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING){O3=x8(YB,j.g6);}
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){
#ifndef RENDER_MODE_DEPTH_STENCIL
O0=W7(L1(RB),NB.zw,l0 C5);
#else
Rc(L1(RB),NB.zw,l0 C5);
#endif
}
#endif
f X=Q3(l0);
#ifdef POST_INVERT_Y
X.y=-X.y;
#endif
#ifdef RENDER_MODE_DEPTH_STENCIL
X.z=qa(LC);
#endif
K1=unpackUnorm4x8(XB);
#ifdef ENABLE_ADVANCED_BLEND
D1=a2(ZB);
#endif
c0(L5);
#ifdef ENABLE_CLIPPING
c0(O3);
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
c0(O0);
#endif
c0(K1);
#ifdef ENABLE_ADVANCED_BLEND
c0(D1);
#endif
C1(X);}
#endif
