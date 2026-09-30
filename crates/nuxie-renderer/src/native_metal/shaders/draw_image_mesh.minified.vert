#ifdef VERTEX
f1(j3)J(0,c,PC);g1 f1(z3)J(1,c,QC);g1 f1(m1)J(w9,f,WB);J(x9,f,SB);J(y9,f,NB);J(z9,uint,XB);J(A9,uint,YB);J(B9,uint,ZB);J(C9,uint,MC);g1
#endif
p2 H0 V(0,c,H5);
#ifdef ENABLE_CLIPPING
OPTIONALLY_FLAT V(1,d,K3);
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
H0 V(2,f,M0);
#endif
OPTIONALLY_FLAT V(3,i,H1);
#ifdef ENABLE_ADVANCED_BLEND
T2 V(4,L,A1);
#endif
h2
#ifdef VERTEX
U3 V3 K6(FC,j3,k3,z3,A3,m1,g0,B){K(B,k3,PC,c);K(B,A3,QC,c);K(v,g0,WB,f);K(v,g0,SB,f);K(v,g0,NB,f);K(v,g0,XB,uint);K(v,g0,YB,uint);K(v,g0,ZB,uint);K(v,g0,MC,uint);T(H5,c);
#ifdef ENABLE_CLIPPING
T(K3,d);
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
T(M0,f);
#endif
T(H1,i);
#ifdef ENABLE_ADVANCED_BLEND
T(A1,L);
#endif
c j0=N0(I1(WB),PC)+NB.xy;H5=QC;
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING){K3=v8(YB,n.f6);}
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){
#ifndef RENDER_MODE_DEPTH_STENCIL
M0=U7(I1(SB),NB.zw,j0 y5);
#else
Kc(I1(SB),NB.zw,j0 y5);
#endif
}
#endif
f W=M3(j0);
#ifdef POST_INVERT_Y
W.y=-W.y;
#endif
#ifdef RENDER_MODE_DEPTH_STENCIL
W.z=na(MC);
#endif
H1=unpackUnorm4x8(XB);
#ifdef ENABLE_ADVANCED_BLEND
A1=Y1(ZB);
#endif
a0(H5);
#ifdef ENABLE_CLIPPING
a0(K3);
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
a0(M0);
#endif
a0(H1);
#ifdef ENABLE_ADVANCED_BLEND
a0(A1);
#endif
z1(W);}
#endif
