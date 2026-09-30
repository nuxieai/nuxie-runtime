#ifdef VERTEX
h1(l3)J(0,c,OC);i1 h1(C3)J(1,c,PC);i1 h1(o1)J(y9,f,VB);J(z9,f,RB);J(A9,f,NB);J(B9,uint,WB);J(C9,uint,XB);J(D9,uint,YB);J(E9,uint,LC);i1
#endif
q2 I0 W(0,c,K5);
#ifdef ENABLE_CLIPPING
OPTIONALLY_FLAT W(1,d,O3);
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
I0 W(2,f,N0);
#endif
OPTIONALLY_FLAT W(3,i,J1);
#ifdef ENABLE_ADVANCED_BLEND
T2 W(4,L,C1);
#endif
i2
#ifdef VERTEX
X3 Y3 K6(EC,l3,m3,C3,D3,o1,h0,B){K(B,m3,OC,c);K(B,D3,PC,c);K(v,h0,VB,f);K(v,h0,RB,f);K(v,h0,NB,f);K(v,h0,WB,uint);K(v,h0,XB,uint);K(v,h0,YB,uint);K(v,h0,LC,uint);U(K5,c);
#ifdef ENABLE_CLIPPING
U(O3,d);
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
U(N0,f);
#endif
U(J1,i);
#ifdef ENABLE_ADVANCED_BLEND
U(C1,L);
#endif
c k0=O0(K1(VB),OC)+NB.xy;K5=PC;
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING){O3=x8(XB,j.f6);}
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){
#ifndef RENDER_MODE_DEPTH_STENCIL
N0=W7(K1(RB),NB.zw,k0 B5);
#else
Nc(K1(RB),NB.zw,k0 B5);
#endif
}
#endif
f X=Q3(k0);
#ifdef POST_INVERT_Y
X.y=-X.y;
#endif
#ifdef RENDER_MODE_DEPTH_STENCIL
X.z=pa(LC);
#endif
J1=unpackUnorm4x8(WB);
#ifdef ENABLE_ADVANCED_BLEND
C1=a2(YB);
#endif
c0(K5);
#ifdef ENABLE_CLIPPING
c0(O3);
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
c0(N0);
#endif
c0(J1);
#ifdef ENABLE_ADVANCED_BLEND
c0(C1);
#endif
B1(X);}
#endif
