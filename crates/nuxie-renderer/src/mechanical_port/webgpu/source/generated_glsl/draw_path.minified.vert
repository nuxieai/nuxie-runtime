#undef P2
#ifdef ENABLE_FEATHER
#define P2 f
#else
#define P2 D
#endif
#ifdef VERTEX
d1(f0)
#if defined(DRAW_INTERIOR_TRIANGLES)||defined(FEATHER_ATLAS_BLIT)
K(0,i4,LB);
#else
K(0,f,XB);K(1,f,YB);
#endif
e1
#endif
v2 F0 W(0,f,O0);
#ifdef FEATHER_ATLAS_BLIT
F0 W(1,c,S2);
#elif!defined(RENDER_MODE_DEPTH_STENCIL)
#ifdef DRAW_INTERIOR_TRIANGLES
OPTIONALLY_FLAT W(1,d,n1);
#else
F0 W(2,P2,S);
#endif
OPTIONALLY_FLAT W(3,d,G0);
#endif
#ifdef ENABLE_CLIPPING
#ifdef FEATHER_ATLAS_BLIT
OPTIONALLY_FLAT W(4,d,f4);
#else
OPTIONALLY_FLAT W(4,D,i2);
#endif
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
F0 W(5,f,V0);
#endif
#ifdef ENABLE_ADVANCED_BLEND
OPTIONALLY_FLAT W(6,d,P0);
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
g3 W(7,R0,z3);W(8,c,L4);
#endif
#ifdef ENABLE_MODULATED_IMAGE
F0 W(9,M,U0);
#endif
k2
#ifdef VERTEX
w1(RB,f0,B,F,r){
#if defined(DRAW_INTERIOR_TRIANGLES)||defined(FEATHER_ATLAS_BLIT)
L(F,B,LB,M);
#else
L(F,B,XB,f);L(F,B,YB,f);
#endif
V(O0,f);
#if defined(ENABLE_MODULATED_IMAGE)
V(U0,M);
#endif
#ifdef FEATHER_ATLAS_BLIT
V(S2,c);
#elif!defined(RENDER_MODE_DEPTH_STENCIL)
#ifdef DRAW_INTERIOR_TRIANGLES
V(n1,d);
#else
V(S,P2);
#endif
V(G0,d);
#endif
#ifdef ENABLE_CLIPPING
#ifdef FEATHER_ATLAS_BLIT
V(f4,d);
#else
V(i2,D);
#endif
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
V(V0,f);
#endif
#ifdef ENABLE_ADVANCED_BLEND
V(P0,d);
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
V(z3,R0);V(L4,c);
#endif
bool uf=false;uint c0;c i0;
#ifdef RENDER_MODE_DEPTH_STENCIL
P E6;
#endif
#ifdef FEATHER_ATLAS_BLIT
i0=Wc(LB,c0,
#ifdef RENDER_MODE_DEPTH_STENCIL
E6,
#endif
S2 Q3);
#elif defined(DRAW_INTERIOR_TRIANGLES)
i0=Xc(LB,c0
#ifdef RENDER_MODE_DEPTH_STENCIL
,E6
#else
,n1
#endif
Q3);
#else
f T;uf=!pa(XB,YB,r,c0,i0
#ifndef RENDER_MODE_DEPTH_STENCIL
,T
#else
,E6
#endif
Q3);
#ifndef RENDER_MODE_DEPTH_STENCIL
#ifdef ENABLE_FEATHER
S=T;
#else
S.xy=B8(T.xy);
#endif
#endif
#endif
R0 S0=w5(VC,c0);
#if!defined(FEATHER_ATLAS_BLIT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
G0=g9(c0,j.w6);if((S0.x&Ja)!=0u) G0=-G0;
#endif
uint w2=S0.x&0xfu;
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING){uint Ij=(w2==J8?S0.y:S0.x)>>16;d y1=g9(Ij,j.w6);if(w2==J8) y1=-y1;
#ifdef FEATHER_ATLAS_BLIT
f4=y1;
#else
i2.x=y1;
#endif
}
#endif
#ifdef ENABLE_ADVANCED_BLEND
if(ENABLE_ADVANCED_BLEND){P0=float((S0.x>>4)&0xfu);}
#endif
c l0=i0;
#ifdef ENABLE_RENDER_TARGET_BOTTOM_UP
if(j.Ba!=0u){l0.y=float(j.Ca)-l0.y;}
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){X I3=o1(p0(JB,c0*m2+2u));f X3=p0(JB,c0*m2+3u);
#ifndef RENDER_MODE_DEPTH_STENCIL
V0=D8(I3,X3.xy,l0);
#else
kb(I3,X3.xy,l0 h5);
#endif
}
#endif
if(w2==Ka){O0=f(unpackUnorm4x8(S0.y));}
#if defined(ENABLE_CLIPPING)&&!defined(FEATHER_ATLAS_BLIT)
else if(ENABLE_CLIPPING&&w2==J8){d e6=g9(S0.x>>16,j.w6);i2.y=e6;}
#endif
else{X Fb=o1(p0(JB,c0*m2));f R7=p0(JB,c0*m2+1u);float o4=uintBitsToFloat(S0.y);O0=Da(l0,Fb,R7.xy,R7.zw,w2,o4,1.0);}
#if defined(ENABLE_MODULATED_IMAGE)
if(ENABLE_MODULATED_IMAGE&&(S0.x&Yd)!=0u){X Gb=o1(p0(JB,c0*m2+4u));f S7=p0(JB,c0*m2+5u);c r3=B0(Gb,l0)+S7.xy;float vf=1.+S7.z;if((S0.x&th)!=0u){uint h4=(S0.x&vh)>>uh;vf=-(1.+float(h4));}U0=M(r3.x,r3.y,vf);}else{U0=M(0.0,0.0,0.0);}
#endif
f I;if(!uf){I=R3(i0);
#ifdef POST_INVERT_Y
I.y=-I.y;
#endif
#ifdef RENDER_MODE_DEPTH_STENCIL
I.z=h9(E6,0xffu);
#elif defined(RENDER_MODE_CLOCKWISE_ATOMIC)
O n5=p0(KB,c0*4u+3u);z3=n5.xy;L4=i0+uintBitsToFloat(n5.zw);
#endif
}else{I=f(j.h3,j.h3,j.h3,j.h3);}Z(O0);
#if defined(ENABLE_MODULATED_IMAGE)
Z(U0);
#endif
#ifdef FEATHER_ATLAS_BLIT
Z(S2);
#elif!defined(RENDER_MODE_DEPTH_STENCIL)
#ifdef DRAW_INTERIOR_TRIANGLES
Z(n1);
#else
Z(S);
#endif
Z(G0);
#endif
#ifdef ENABLE_CLIPPING
#ifdef FEATHER_ATLAS_BLIT
Z(f4);
#else
Z(i2);
#endif
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
Z(V0);
#endif
#ifdef ENABLE_ADVANCED_BLEND
Z(P0);
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
Z(z3);Z(L4);
#endif
x1(I);}
#endif
#ifdef FRAGMENT
k4 l4 e d Jj(i Fc,uint h4){d wf=dot(Fc.xyz,Z0(.30,.59,.11));if(h4==wh) return Fc.w;if(h4==xh) return 1.-Fc.w;if(h4==yh) return wf;return 1.-wf;}e i r8(
#ifdef ENABLE_MODULATED_IMAGE
M v8,
#endif
#ifdef ENABLE_ADVANCED_BLEND
P W1,
#endif
f ga j7){
#ifdef ENABLE_ADVANCED_BLEND
bool F2=ENABLE_ADVANCED_BLEND&&W1!=U3;
#else
const bool F2=false;
#endif
i l;if(ga.w>=.0){l=V4(ga);}else{c Ra=Sa(ga,j.L8,j.M8);l=n2(XC,N8,Ra,.0);if(!F2){d o4=ke(ga);l.xyz*=l.w;l.w*=o4;}}
#if defined(ENABLE_MODULATED_IMAGE)
if(ENABLE_MODULATED_IMAGE&&v8.z<0.0){return D5(TB,U4,v8.xy,J0(.0));}if(ENABLE_MODULATED_IMAGE&&v8.z>0.0){d Hb=v8.z-1.;i M1=D5(TB,U4,v8.xy,Hb);if(F2) M1=H0(i6(M1),M1.w);l*=M1;}
#endif
return l;}
#if!defined(DRAW_INTERIOR_TRIANGLES)&&!defined(FEATHER_ATLAS_BLIT)
e d xf(P2 T c4){
#ifdef ENABLE_FEATHER
if(ENABLE_FEATHER&&fd(T)) return T4(T m1);else
#endif
return min(T.x,T.y);}e d yf(P2 T c4){
#if defined(ENABLE_FEATHER)
if(ENABLE_FEATHER&&gd(T)) return Q8(T m1);else
#endif
return T.x;}e d Gc(P2 T c4){if(o6(T)) return xf(T m1);else return yf(T m1);}e d Kj(d o5,P2 T c4){if(o6(T)){d A0=xf(T m1);return max(A0,o5);}else{d A0=yf(T m1);return o5+A0;}}
#endif
#endif
