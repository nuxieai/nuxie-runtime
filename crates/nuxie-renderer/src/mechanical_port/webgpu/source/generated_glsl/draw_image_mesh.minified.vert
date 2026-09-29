#ifdef VERTEX
g1(i3)L(0,d,PC);h1 g1(y3)L(1,d,QC);h1 g1(n1)L(r9,g,XB);L(v9,g,TB);L(w9,g,OB);
#ifdef O3
L(x9,uint,YB);L(y9,uint,ZB);L(z9,uint,AC);L(A9,uint,BC);
#else
L(B9,G,IB);
#endif
h1
#endif
m2 H0 X(0,d,H5);
#ifdef ENABLE_CLIPPING
OPTIONALLY_FLAT X(1,c,K3);
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
H0 X(2,g,M0);
#endif
OPTIONALLY_FLAT X(3,c,I1);
#ifdef ENABLE_ADVANCED_BLEND
Q2 X(4,N,B1);
#endif
g2
#ifdef VERTEX
U3 V3 J6(HC,i3,j3,y3,z3,n1,g0,B){M(B,j3,PC,d);M(B,z3,QC,d);M(v,g0,XB,g);M(v,g0,TB,g);M(v,g0,OB,g);
#ifdef O3
M(v,g0,YB,uint);M(v,g0,ZB,uint);M(v,g0,AC,uint);M(v,g0,BC,uint);G IB=G(YB,ZB,AC,BC);
#else
M(v,g0,IB,G);
#endif
V(H5,d);
#ifdef ENABLE_CLIPPING
V(K3,c);
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
V(M0,g);
#endif
V(I1,c);
#ifdef ENABLE_ADVANCED_BLEND
V(B1,N);
#endif
d m0=R0(h2(XB),PC)+OB.xy;H5=QC;
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING){K3=r8(IB.y,m.e6);}
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){
#ifndef RENDER_MODE_DEPTH_STENCIL
M0=T7(h2(TB),OB.zw,m0 y5);
#else
Cc(h2(TB),OB.zw,m0 y5);
#endif
}
#endif
g W=M3(m0);
#ifdef POST_INVERT_Y
W.y=-W.y;
#endif
#ifdef RENDER_MODE_DEPTH_STENCIL
W.z=ja(IB.w);
#endif
I1=uintBitsToFloat(IB.x);
#ifdef ENABLE_ADVANCED_BLEND
B1=X1(IB.z);
#endif
c0(H5);
#ifdef ENABLE_CLIPPING
c0(K3);
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
c0(M0);
#endif
c0(I1);
#ifdef ENABLE_ADVANCED_BLEND
c0(B1);
#endif
A1(W);}
#endif
