#ifdef VERTEX
g1(n3) I(0,c,PC);h1 g1(C3) I(1,c,QC);h1 g1(p1) I(v9,f,XB);I(w9,f,RB);I(x9,f,NB);I(y9,uint,YB);I(z9,uint,ZB);I(A9,uint,AC);I(B9,uint,LC);I(G9,f,GC);h1
#endif
r2 I0 W(0,c,J5);
#ifdef ENABLE_CLIPPING
OPTIONALLY_FLAT W(1,d,N3);
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
I0 W(2,f,P0);
#endif
OPTIONALLY_FLAT W(3,i,K1);
#ifdef ENABLE_ADVANCED_BLEND
W2 W(4,N,C1);
#endif
i2
#ifdef VERTEX
Y3 Z3 J6(EC,n3,o3,C3,D3,p1,g0,A){J(A,o3,PC,c);J(A,D3,QC,c);J(q,g0,XB,f);J(q,g0,RB,f);J(q,g0,NB,f);J(q,g0,YB,uint);J(q,g0,ZB,uint);J(q,g0,AC,uint);J(q,g0,LC,uint);J(q,g0,GC,f);V(J5,c);
#ifdef ENABLE_CLIPPING
V(N3,d);
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
V(P0,f);
#endif
V(K1,i);
#ifdef ENABLE_ADVANCED_BLEND
V(C1,N);
#endif
c k0=N0(L1(XB),PC)+NB.xy;J5=QC*GC.zw+GC.xy;
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING){N3=r8(ZB,j.c6);}
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){
#ifndef RENDER_MODE_DEPTH_STENCIL
P0=S7(L1(RB),NB.zw,k0 A5);
#else
Mc(L1(RB),NB.zw,k0 A5);
#endif
}
#endif
f X=P3(k0);
#ifdef POST_INVERT_Y
X.y=-X.y;
#endif
#ifdef RENDER_MODE_DEPTH_STENCIL
X.z=ma(LC,0xffu);
#endif
K1=unpackUnorm4x8(YB);
#ifdef ENABLE_ADVANCED_BLEND
C1=a2(AC);
#endif
c0(J5);
#ifdef ENABLE_CLIPPING
c0(N3);
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
c0(P0);
#endif
c0(K1);
#ifdef ENABLE_ADVANCED_BLEND
c0(C1);
#endif
B1(X);}
#endif
