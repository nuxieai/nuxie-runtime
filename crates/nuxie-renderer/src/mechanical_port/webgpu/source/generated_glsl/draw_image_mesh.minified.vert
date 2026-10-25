#ifdef VERTEX
e1(E3) K(0,c,PC);f1 e1(W3) K(1,c,QC);f1 e1(E1) K(ta,e,AC);K(ua,e,UB);K(va,e,PB);K(wa,uint,BC);K(xa,uint,CC);K(ya,uint,DC);K(za,uint,MC);K(Ha,e,IC);f1
#endif
m2 H0 W(0,c,g6);
#ifdef ENABLE_CLIPPING
OPTIONALLY_FLAT W(1,d,i4);
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
H0 W(2,e,W0);
#endif
OPTIONALLY_FLAT W(3,i,Y1);
#ifdef ENABLE_ADVANCED_BLEND
h3 W(4,P,N1);
#endif
M1
#ifdef VERTEX
x4 y4 k7(TB,E3,F3,W3,j3,E1,j0,C){L(C,F3,PC,c);L(C,j3,QC,c);L(r,j0,AC,e);L(r,j0,UB,e);L(r,j0,PB,e);L(r,j0,BC,uint);L(r,j0,CC,uint);L(r,j0,DC,uint);L(r,j0,MC,uint);L(r,j0,IC,e);V(g6,c);
#ifdef ENABLE_CLIPPING
V(i4,d);
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
V(W0,e);
#endif
V(Y1,i);
#ifdef ENABLE_ADVANCED_BLEND
V(N1,P);
#endif
c i0=D0(q1(AC),PC)+PB.xy;g6=QC*IC.zw+IC.xy;
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING){i4=k9(CC,j.z6);}
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){
#ifndef RENDER_MODE_DEPTH_STENCIL
W0=I8(q1(UB),PB.zw,i0 E4);
#else
pb(q1(UB),PB.zw,i0 E4);
#endif
}
#endif
e I=U3(i0);
#ifdef POST_INVERT_Y
I.y=-I.y;
#endif
#ifdef RENDER_MODE_DEPTH_STENCIL
I.z=l9(MC,0xffu);
#endif
Y1=unpackUnorm4x8(BC);
#ifdef ENABLE_ADVANCED_BLEND
N1=X1(DC);
#endif
a0(g6);
#ifdef ENABLE_CLIPPING
a0(i4);
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
a0(W0);
#endif
a0(Y1);
#ifdef ENABLE_ADVANCED_BLEND
a0(N1);
#endif
A1(I);}
#endif
