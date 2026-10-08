#ifdef VERTEX
f1(B3) K(0,c,PC);g1 f1(S3) K(1,c,QC);g1 f1(D1) K(la,e,ZB);K(ma,e,SB);K(na,e,PB);K(oa,uint,AC);K(pa,uint,BC);K(qa,uint,CC);K(ra,uint,LC);K(ya,e,HC);g1
#endif
w2 F0 X(0,c,Z5);
#ifdef ENABLE_CLIPPING
OPTIONALLY_FLAT X(1,d,e4);
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
F0 X(2,e,W0);
#endif
OPTIONALLY_FLAT X(3,i,U1);
#ifdef ENABLE_ADVANCED_BLEND
g3 X(4,P,K1);
#endif
l2
#ifdef VERTEX
o4 p4 c7(RB,B3,C3,S3,x2,D1,j0,F){L(F,C3,PC,c);L(F,x2,QC,c);L(r,j0,ZB,e);L(r,j0,SB,e);L(r,j0,PB,e);L(r,j0,AC,uint);L(r,j0,BC,uint);L(r,j0,CC,uint);L(r,j0,LC,uint);L(r,j0,HC,e);V(Z5,c);
#ifdef ENABLE_CLIPPING
V(e4,d);
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
V(W0,e);
#endif
V(U1,i);
#ifdef ENABLE_ADVANCED_BLEND
V(K1,P);
#endif
c i0=y0(p1(ZB),PC)+PB.xy;Z5=QC*HC.zw+HC.xy;
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING){e4=c9(BC,j.p6);}
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){
#ifndef RENDER_MODE_DEPTH_STENCIL
W0=B8(p1(SB),PB.zw,i0 e5);
#else
fb(p1(SB),PB.zw,i0 e5);
#endif
}
#endif
e I=Q3(i0);
#ifdef POST_INVERT_Y
I.y=-I.y;
#endif
#ifdef RENDER_MODE_DEPTH_STENCIL
I.z=d9(LC,0xffu);
#endif
U1=unpackUnorm4x8(AC);
#ifdef ENABLE_ADVANCED_BLEND
K1=T1(CC);
#endif
Z(Z5);
#ifdef ENABLE_CLIPPING
Z(e4);
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
Z(W0);
#endif
Z(U1);
#ifdef ENABLE_ADVANCED_BLEND
Z(K1);
#endif
y1(I);}
#endif
