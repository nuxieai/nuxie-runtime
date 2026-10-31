#ifdef VERTEX
f1(H3) K(0,c,QC);g1 f1(W3) K(1,c,RC);g1 f1(G1) K(wa,e,AC);K(xa,e,VB);K(ya,e,QB);K(za,uint,BC);K(Aa,uint,CC);K(Ba,uint,DC);K(Ca,uint,NC);K(Ka,e,IC);g1
#endif
n2 I0 W(0,c,h6);
#ifdef ENABLE_CLIPPING
OPTIONALLY_FLAT W(1,d,j4);
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
I0 W(2,e,Y0);
#endif
OPTIONALLY_FLAT W(3,i,a2);
#ifdef ENABLE_ADVANCED_BLEND
m3 W(4,O,Q1);
#endif
P1
#ifdef VERTEX
A4 B4 k7(UB,H3,I3,W3,o3,G1,j0,C){L(C,I3,QC,c);L(C,o3,RC,c);L(r,j0,AC,e);L(r,j0,VB,e);L(r,j0,QB,e);L(r,j0,BC,uint);L(r,j0,CC,uint);L(r,j0,DC,uint);L(r,j0,NC,uint);L(r,j0,IC,e);V(h6,c);
#ifdef ENABLE_CLIPPING
V(j4,d);
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
V(Y0,e);
#endif
V(a2,i);
#ifdef ENABLE_ADVANCED_BLEND
V(Q1,O);
#endif
c i0=F0(v1(AC),QC)+QB.xy;h6=RC*IC.zw+IC.xy;
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING){j4=n9(CC,j.z6);}
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){
#ifndef RENDER_MODE_DEPTH_STENCIL
Y0=L8(v1(VB),QB.zw,i0 H4);
#else
tb(v1(VB),QB.zw,i0 H4);
#endif
}
#endif
e I=U3(i0);
#ifdef POST_INVERT_Y
I.y=-I.y;
#endif
#ifdef RENDER_MODE_DEPTH_STENCIL
I.z=o9(NC,0xffu);
#endif
a2=unpackUnorm4x8(BC);
#ifdef ENABLE_ADVANCED_BLEND
Q1=Z1(DC);
#endif
e0(h6);
#ifdef ENABLE_CLIPPING
e0(j4);
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
e0(Y0);
#endif
e0(a2);
#ifdef ENABLE_ADVANCED_BLEND
e0(Q1);
#endif
B1(I);}
#endif
