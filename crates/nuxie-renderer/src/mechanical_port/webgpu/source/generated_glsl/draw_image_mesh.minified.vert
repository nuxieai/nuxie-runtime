#ifdef VERTEX
e1(D3) K(0,c,PC);f1 e1(V3) K(1,c,QC);f1 e1(E1) K(sa,e,AC);K(ta,e,TB);K(ua,e,PB);K(va,uint,BC);K(wa,uint,CC);K(xa,uint,DC);K(ya,uint,MC);K(Ga,e,IC);f1
#endif
m2 F0 W(0,c,g6);
#ifdef ENABLE_CLIPPING
OPTIONALLY_FLAT W(1,d,h4);
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
F0 W(2,e,W0);
#endif
OPTIONALLY_FLAT W(3,i,Y1);
#ifdef ENABLE_ADVANCED_BLEND
h3 W(4,P,N1);
#endif
M1
#ifdef VERTEX
v4 w4 l7(SB,D3,E3,V3,j3,E1,j0,C){L(C,E3,PC,c);L(C,j3,QC,c);L(r,j0,AC,e);L(r,j0,TB,e);L(r,j0,PB,e);L(r,j0,BC,uint);L(r,j0,CC,uint);L(r,j0,DC,uint);L(r,j0,MC,uint);L(r,j0,IC,e);V(g6,c);
#ifdef ENABLE_CLIPPING
V(h4,d);
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
V(W0,e);
#endif
V(Y1,i);
#ifdef ENABLE_ADVANCED_BLEND
V(N1,P);
#endif
c i0=C0(q1(AC),PC)+PB.xy;g6=QC*IC.zw+IC.xy;
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING){h4=j9(CC,j.z6);}
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){
#ifndef RENDER_MODE_DEPTH_STENCIL
W0=H8(q1(TB),PB.zw,i0 C4);
#else
ob(q1(TB),PB.zw,i0 C4);
#endif
}
#endif
e I=T3(i0);
#ifdef POST_INVERT_Y
I.y=-I.y;
#endif
#ifdef RENDER_MODE_DEPTH_STENCIL
I.z=k9(MC,0xffu);
#endif
Y1=unpackUnorm4x8(BC);
#ifdef ENABLE_ADVANCED_BLEND
N1=X1(DC);
#endif
a0(g6);
#ifdef ENABLE_CLIPPING
a0(h4);
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
