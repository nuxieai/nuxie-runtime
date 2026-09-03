#ifdef VERTEX
g1(i3)L(0,d,PC);h1 g1(y3)L(1,d,QC);h1 g1(n1)L(r9,g,XB);L(v9,g,SB);L(w9,g,OB);
#ifdef O3
L(x9,uint,YB);L(y9,uint,ZB);L(z9,uint,AC);L(A9,uint,BC);
#else
L(B9,G,IB);
#endif
h1
#endif
m2 H0 X(0,d,G5);
#ifdef ENABLE_CLIPPING
OPTIONALLY_FLAT X(1,c,K3);
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_MSAA)
H0 X(2,g,M0);
#endif
OPTIONALLY_FLAT X(3,c,H1);
#ifdef ENABLE_ADVANCED_BLEND
Q2 X(4,N,A1);
#endif
g2
#ifdef VERTEX
U3 V3 I6(HC,i3,j3,y3,z3,n1,g0,B){M(B,j3,PC,d);M(B,z3,QC,d);M(v,g0,XB,g);M(v,g0,SB,g);M(v,g0,OB,g);
#ifdef O3
M(v,g0,YB,uint);M(v,g0,ZB,uint);M(v,g0,AC,uint);M(v,g0,BC,uint);G IB=G(YB,ZB,AC,BC);
#else
M(v,g0,IB,G);
#endif
V(G5,d);
#ifdef ENABLE_CLIPPING
V(K3,c);
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_MSAA)
V(M0,g);
#endif
V(H1,c);
#ifdef ENABLE_ADVANCED_BLEND
V(A1,N);
#endif
d m0=R0(h2(XB),PC)+OB.xy;G5=QC;
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING){K3=r8(IB.y,m.d6);}
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){
#ifndef RENDER_MODE_MSAA
M0=T7(h2(SB),OB.zw,m0 x5);
#else
Bc(h2(SB),OB.zw,m0 x5);
#endif
}
#endif
g W=M3(m0);
#ifdef POST_INVERT_Y
W.y=-W.y;
#endif
#ifdef RENDER_MODE_MSAA
W.z=ja(IB.w);
#endif
H1=uintBitsToFloat(IB.x);
#ifdef ENABLE_ADVANCED_BLEND
A1=X1(IB.z);
#endif
a0(G5);
#ifdef ENABLE_CLIPPING
a0(K3);
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_MSAA)
a0(M0);
#endif
a0(H1);
#ifdef ENABLE_ADVANCED_BLEND
a0(A1);
#endif
z1(W);}
#endif
