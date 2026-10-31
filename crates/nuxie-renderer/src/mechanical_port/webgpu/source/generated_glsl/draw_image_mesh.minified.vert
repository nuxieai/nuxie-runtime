#ifdef VERTEX
d1(F3) K(0,c,PC);e1 d1(W3) K(1,c,QC);e1 d1(E1) K(ta,e,AC);K(ua,e,UB);K(va,e,PB);K(wa,uint,BC);K(xa,uint,CC);K(ya,uint,DC);K(za,uint,MC);K(Ha,e,IC);e1
#endif
n2 H0 W(0,c,e6);
#ifdef ENABLE_CLIPPING
OPTIONALLY_FLAT W(1,d,h4);
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
H0 W(2,e,V0);
#endif
OPTIONALLY_FLAT W(3,i,Y1);
#ifdef ENABLE_ADVANCED_BLEND
i3 W(4,O,O1);
#endif
N1
#ifdef VERTEX
x4 y4 k7(TB,F3,G3,W3,k3,E1,j0,C){L(C,G3,PC,c);L(C,k3,QC,c);L(r,j0,AC,e);L(r,j0,UB,e);L(r,j0,PB,e);L(r,j0,BC,uint);L(r,j0,CC,uint);L(r,j0,DC,uint);L(r,j0,MC,uint);L(r,j0,IC,e);V(e6,c);
#ifdef ENABLE_CLIPPING
V(h4,d);
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
V(V0,e);
#endif
V(Y1,i);
#ifdef ENABLE_ADVANCED_BLEND
V(O1,O);
#endif
c i0=D0(p1(AC),PC)+PB.xy;e6=QC*IC.zw+IC.xy;
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING){h4=j9(CC,j.w6);}
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){
#ifndef RENDER_MODE_DEPTH_STENCIL
V0=H8(p1(UB),PB.zw,i0 E4);
#else
qb(p1(UB),PB.zw,i0 E4);
#endif
}
#endif
e I=U3(i0);
#ifdef POST_INVERT_Y
I.y=-I.y;
#endif
#ifdef RENDER_MODE_DEPTH_STENCIL
I.z=k9(MC,0xffu);
#endif
Y1=unpackUnorm4x8(BC);
#ifdef ENABLE_ADVANCED_BLEND
O1=X1(DC);
#endif
a0(e6);
#ifdef ENABLE_CLIPPING
a0(h4);
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
a0(V0);
#endif
a0(Y1);
#ifdef ENABLE_ADVANCED_BLEND
a0(O1);
#endif
A1(I);}
#endif
