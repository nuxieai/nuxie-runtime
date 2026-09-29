#ifdef VERTEX
g1(i3)O(0,d,OC);h1 g1(x3)O(1,d,PC);h1 g1(n1)O(r9,g,WB);O(v9,g,SB);O(w9,g,NB);O(x9,float,XB);O(y9,uint,YB);O(z9,uint,ZB);O(A9,uint,MC);h1
#endif
m2 H0 W(0,d,G5);
#ifdef ENABLE_CLIPPING
OPTIONALLY_FLAT W(1,c,J3);
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
H0 W(2,g,M0);
#endif
OPTIONALLY_FLAT W(3,c,I1);
#ifdef ENABLE_ADVANCED_BLEND
Q2 W(4,K,B1);
#endif
g2
#ifdef VERTEX
T3 U3 I6(FC,i3,j3,x3,y3,n1,i0,B){P(B,j3,OC,d);P(B,y3,PC,d);P(A,i0,WB,g);P(A,i0,SB,g);P(A,i0,NB,g);P(A,i0,XB,float);P(A,i0,YB,uint);P(A,i0,ZB,uint);P(A,i0,MC,uint);U(G5,d);
#ifdef ENABLE_CLIPPING
U(J3,c);
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
U(M0,g);
#endif
U(I1,c);
#ifdef ENABLE_ADVANCED_BLEND
U(B1,K);
#endif
d m0=R0(h2(WB),OC)+NB.xy;G5=PC;
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING){J3=r8(YB,m.d6);}
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){
#ifndef RENDER_MODE_DEPTH_STENCIL
M0=T7(h2(SB),NB.zw,m0 x5);
#else
Dc(h2(SB),NB.zw,m0 x5);
#endif
}
#endif
g V=L3(m0);
#ifdef POST_INVERT_Y
V.y=-V.y;
#endif
#ifdef RENDER_MODE_DEPTH_STENCIL
V.z=ia(MC);
#endif
I1=XB;
#ifdef ENABLE_ADVANCED_BLEND
B1=X1(ZB);
#endif
c0(G5);
#ifdef ENABLE_CLIPPING
c0(J3);
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
c0(M0);
#endif
c0(I1);
#ifdef ENABLE_ADVANCED_BLEND
c0(B1);
#endif
A1(V);}
#endif
