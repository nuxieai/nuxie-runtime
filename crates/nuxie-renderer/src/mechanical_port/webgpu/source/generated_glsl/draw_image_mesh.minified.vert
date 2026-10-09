#ifdef VERTEX
d1(C3) K(0,c,OC);e1 d1(T3) K(1,c,PC);e1 d1(C1) K(qa,f,ZB);K(ra,f,SB);K(sa,f,PB);K(ta,uint,AC);K(ua,uint,BC);K(va,uint,CC);K(wa,uint,LC);K(Ea,f,HC);e1
#endif
v2 F0 W(0,c,d6);
#ifdef ENABLE_CLIPPING
OPTIONALLY_FLAT W(1,d,f4);
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
F0 W(2,f,V0);
#endif
OPTIONALLY_FLAT W(3,i,T1);
#ifdef ENABLE_ADVANCED_BLEND
g3 W(4,P,J1);
#endif
k2
#ifdef VERTEX
q4 r4 g7(RB,C3,D3,T3,i3,C1,j0,F){L(F,D3,OC,c);L(F,i3,PC,c);L(r,j0,ZB,f);L(r,j0,SB,f);L(r,j0,PB,f);L(r,j0,AC,uint);L(r,j0,BC,uint);L(r,j0,CC,uint);L(r,j0,LC,uint);L(r,j0,HC,f);V(d6,c);
#ifdef ENABLE_CLIPPING
V(f4,d);
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
V(V0,f);
#endif
V(T1,i);
#ifdef ENABLE_ADVANCED_BLEND
V(J1,P);
#endif
c i0=B0(o1(ZB),OC)+PB.xy;d6=PC*HC.zw+HC.xy;
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING){f4=g9(BC,j.w6);}
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){
#ifndef RENDER_MODE_DEPTH_STENCIL
V0=D8(o1(SB),PB.zw,i0 h5);
#else
kb(o1(SB),PB.zw,i0 h5);
#endif
}
#endif
f I=R3(i0);
#ifdef POST_INVERT_Y
I.y=-I.y;
#endif
#ifdef RENDER_MODE_DEPTH_STENCIL
I.z=h9(LC,0xffu);
#endif
T1=unpackUnorm4x8(AC);
#ifdef ENABLE_ADVANCED_BLEND
J1=S1(CC);
#endif
Z(d6);
#ifdef ENABLE_CLIPPING
Z(f4);
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
Z(V0);
#endif
Z(T1);
#ifdef ENABLE_ADVANCED_BLEND
Z(J1);
#endif
x1(I);}
#endif
