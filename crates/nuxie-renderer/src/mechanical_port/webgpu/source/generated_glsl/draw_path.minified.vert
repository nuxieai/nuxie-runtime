#undef Q2
#ifdef ENABLE_FEATHER
#define Q2 e
#else
#define Q2 D
#endif
#ifdef VERTEX
f1(f0)
#if defined(DRAW_INTERIOR_TRIANGLES)||defined(FEATHER_ATLAS_BLIT)
K(0,h4,LB);
#else
K(0,e,XB);K(1,e,YB);
#endif
g1
#endif
w2 F0 X(0,e,O0);
#ifdef FEATHER_ATLAS_BLIT
F0 X(1,c,T2);
#elif!defined(RENDER_MODE_DEPTH_STENCIL)
#ifdef DRAW_INTERIOR_TRIANGLES
OPTIONALLY_FLAT X(1,d,o1);
#else
F0 X(2,Q2,S);
#endif
OPTIONALLY_FLAT X(3,d,G0);
#endif
#ifdef ENABLE_CLIPPING
#ifdef FEATHER_ATLAS_BLIT
OPTIONALLY_FLAT X(4,d,e4);
#else
OPTIONALLY_FLAT X(4,D,j2);
#endif
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
F0 X(5,e,W0);
#endif
#ifdef ENABLE_ADVANCED_BLEND
OPTIONALLY_FLAT X(6,d,P0);
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
g3 X(7,S0,y3);X(8,c,J4);
#endif
#ifdef ENABLE_MODULATED_IMAGE
F0 X(9,M,V0);
#endif
l2
#ifdef VERTEX
x1(RB,f0,B,F,r){
#if defined(DRAW_INTERIOR_TRIANGLES)||defined(FEATHER_ATLAS_BLIT)
L(F,B,LB,M);
#else
L(F,B,XB,e);L(F,B,YB,e);
#endif
V(O0,e);
#if defined(ENABLE_MODULATED_IMAGE)
V(V0,M);
#endif
#ifdef FEATHER_ATLAS_BLIT
V(T2,c);
#elif!defined(RENDER_MODE_DEPTH_STENCIL)
#ifdef DRAW_INTERIOR_TRIANGLES
V(o1,d);
#else
V(S,Q2);
#endif
V(G0,d);
#endif
#ifdef ENABLE_CLIPPING
#ifdef FEATHER_ATLAS_BLIT
V(e4,d);
#else
V(j2,D);
#endif
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
V(W0,e);
#endif
#ifdef ENABLE_ADVANCED_BLEND
V(P0,d);
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
V(y3,S0);V(J4,c);
#endif
bool rf=false;uint c0;c i0;
#ifdef RENDER_MODE_DEPTH_STENCIL
P C6;
#endif
#ifdef FEATHER_ATLAS_BLIT
i0=Oc(LB,c0,
#ifdef RENDER_MODE_DEPTH_STENCIL
C6,
#endif
T2 P3);
#elif defined(DRAW_INTERIOR_TRIANGLES)
i0=Pc(LB,c0
#ifdef RENDER_MODE_DEPTH_STENCIL
,C6
#else
,o1
#endif
P3);
#else
e T;rf=!ia(XB,YB,r,c0,i0
#ifndef RENDER_MODE_DEPTH_STENCIL
,T
#else
,C6
#endif
P3);
#ifndef RENDER_MODE_DEPTH_STENCIL
#ifdef ENABLE_FEATHER
S=T;
#else
S.xy=y8(T.xy);
#endif
#endif
#endif
S0 T0=q5(WC,c0);
#if!defined(FEATHER_ATLAS_BLIT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
G0=a9(c0,j.p6);if((T0.x&Ba)!=0u) G0=-G0;
#endif
uint j3=T0.x&0xfu;
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING){uint Fj=(j3==G8?T0.y:T0.x)>>16;d z1=a9(Fj,j.p6);if(j3==G8) z1=-z1;
#ifdef FEATHER_ATLAS_BLIT
e4=z1;
#else
j2.x=z1;
#endif
}
#endif
#ifdef ENABLE_ADVANCED_BLEND
if(ENABLE_ADVANCED_BLEND){P0=float((T0.x>>4)&0xfu);}
#endif
c l0=i0;
#ifdef ENABLE_RENDER_TARGET_BOTTOM_UP
if(j.ua!=0u){l0.y=float(j.va)-l0.y;}
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){W H3=p1(p0(JB,c0*n2+2u));e W3=p0(JB,c0*n2+3u);
#ifndef RENDER_MODE_DEPTH_STENCIL
W0=A8(H3,W3.xy,l0);
#else
db(H3,W3.xy,l0 e5);
#endif
}
#endif
if(j3==Ca){O0=e(unpackUnorm4x8(T0.y));}
#if defined(ENABLE_CLIPPING)&&!defined(FEATHER_ATLAS_BLIT)
else if(ENABLE_CLIPPING&&j3==G8){d a6=a9(T0.x>>16,j.p6);j2.y=a6;}
#endif
else{W zb=p1(p0(JB,c0*n2));e O7=p0(JB,c0*n2+1u);O0=Rc(l0,zb,O7.xy,float(j3),O7.zw,uintBitsToFloat(T0.y));O0.w=-O0.w;}
#if defined(ENABLE_MODULATED_IMAGE)
if(ENABLE_MODULATED_IMAGE&&(T0.x&Vd)!=0u){W Ab=p1(p0(JB,c0*n2+4u));e P7=p0(JB,c0*n2+5u);c r3=y0(Ab,l0)+P7.xy;float sf=1.+P7.z;if((T0.x&oh)!=0u){uint g4=(T0.x&qh)>>ph;sf=-(1.+float(g4));}V0=M(r3.x,r3.y,sf);}else{V0=M(0.0,0.0,0.0);}
#endif
e I;if(!rf){I=Q3(i0);
#ifdef POST_INVERT_Y
I.y=-I.y;
#endif
#ifdef RENDER_MODE_DEPTH_STENCIL
I.z=c9(C6,0xffu);
#elif defined(RENDER_MODE_CLOCKWISE_ATOMIC)
O k5=p0(KB,c0*4u+3u);y3=k5.xy;J4=i0+uintBitsToFloat(k5.zw);
#endif
}else{I=e(j.h3,j.h3,j.h3,j.h3);}Z(O0);
#if defined(ENABLE_MODULATED_IMAGE)
Z(V0);
#endif
#ifdef FEATHER_ATLAS_BLIT
Z(T2);
#elif!defined(RENDER_MODE_DEPTH_STENCIL)
#ifdef DRAW_INTERIOR_TRIANGLES
Z(o1);
#else
Z(S);
#endif
Z(G0);
#endif
#ifdef ENABLE_CLIPPING
#ifdef FEATHER_ATLAS_BLIT
Z(e4);
#else
Z(j2);
#endif
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
Z(W0);
#endif
#ifdef ENABLE_ADVANCED_BLEND
Z(P0);
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
Z(y3);Z(J4);
#endif
y1(I);}
#endif
#ifdef FRAGMENT
k4 l4 f d Gj(i xc,uint g4){d tf=dot(xc.xyz,a1(.30,.59,.11));if(g4==rh) return xc.w;if(g4==sh) return 1.-xc.w;if(g4==th) return tf;return 1.-tf;}f i o8(
#ifdef ENABLE_MODULATED_IMAGE
M p8,
#endif
#ifdef ENABLE_ADVANCED_BLEND
P X1,
#endif
e l5 f7){
#ifdef ENABLE_ADVANCED_BLEND
bool F2=ENABLE_ADVANCED_BLEND&&X1!=T3;
#else
const bool F2=false;
#endif
i n;if(l5.w>=.0){n=T4(l5);}else{l5.w=-l5.w;d Fa=i4(fract(l5.w)*(256./255.));l5.w=floor(l5.w)*j.ad+j.g7;c La=fd(l5);n=o2(YC,H8,La,.0);if(!F2){n.xyz*=n.w;n.w*=Fa;}}
#if defined(ENABLE_MODULATED_IMAGE)
if(ENABLE_MODULATED_IMAGE&&p8.z<0.0){return A5(TB,S4,p8.xy,I0(.0));}if(ENABLE_MODULATED_IMAGE&&p8.z>0.0){d Db=p8.z-1.;i O1=A5(TB,S4,p8.xy,Db);if(F2) O1=H0(f6(O1),O1.w);n*=O1;}
#endif
return n;}
#if!defined(DRAW_INTERIOR_TRIANGLES)&&!defined(FEATHER_ATLAS_BLIT)
f d uf(Q2 T a4){
#ifdef ENABLE_FEATHER
if(ENABLE_FEATHER&&bd(T)) return R4(T n1);else
#endif
return min(T.x,T.y);}f d vf(Q2 T a4){
#if defined(ENABLE_FEATHER)
if(ENABLE_FEATHER&&cd(T)) return K8(T n1);else
#endif
return T.x;}f d yc(Q2 T a4){if(l6(T)) return uf(T n1);else return vf(T n1);}f d Hj(d m5,Q2 T a4){if(l6(T)){d B0=uf(T n1);return max(B0,m5);}else{d B0=vf(T n1);return m5+B0;}}
#endif
#endif
