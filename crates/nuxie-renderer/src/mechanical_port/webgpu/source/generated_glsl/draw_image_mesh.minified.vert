#ifdef VERTEX
e1(C3) K(0,c,PC);f1 e1(T3) K(1,c,QC);f1 e1(D1) K(sa,f,ZB);K(ta,f,SB);K(ua,f,PB);K(va,uint,AC);K(wa,uint,BC);K(xa,uint,CC);K(ya,uint,MC);K(Ga,f,HC);f1
#endif
v2 F0 W(0,c,d6);
#ifdef ENABLE_CLIPPING
OPTIONALLY_FLAT W(1,d,f4);
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
F0 W(2,f,V0);
#endif
OPTIONALLY_FLAT W(3,i,V1);
#ifdef ENABLE_ADVANCED_BLEND
h3 W(4,P,L1);
#endif
k2
#ifdef VERTEX
q4 r4 k7(RB,C3,D3,T3,j3,D1,j0,F){L(F,D3,PC,c);L(F,j3,QC,c);L(r,j0,ZB,f);L(r,j0,SB,f);L(r,j0,PB,f);L(r,j0,AC,uint);L(r,j0,BC,uint);L(r,j0,CC,uint);L(r,j0,MC,uint);L(r,j0,HC,f);V(d6,c);
#ifdef ENABLE_CLIPPING
V(f4,d);
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
V(V0,f);
#endif
V(V1,i);
#ifdef ENABLE_ADVANCED_BLEND
V(L1,P);
#endif
c i0=C0(p1(ZB),PC)+PB.xy;d6=QC*HC.zw+HC.xy;
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING){f4=j9(BC,j.w6);}
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){
#ifndef RENDER_MODE_DEPTH_STENCIL
V0=G8(p1(SB),PB.zw,i0 h5);
#else
ob(p1(SB),PB.zw,i0 h5);
#endif
}
#endif
f I=R3(i0);
#ifdef POST_INVERT_Y
I.y=-I.y;
#endif
#ifdef RENDER_MODE_DEPTH_STENCIL
I.z=k9(MC,0xffu);
#endif
V1=unpackUnorm4x8(AC);
#ifdef ENABLE_ADVANCED_BLEND
L1=U1(CC);
#endif
Z(d6);
#ifdef ENABLE_CLIPPING
Z(f4);
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
Z(V0);
#endif
Z(V1);
#ifdef ENABLE_ADVANCED_BLEND
Z(L1);
#endif
z1(I);}
#endif
