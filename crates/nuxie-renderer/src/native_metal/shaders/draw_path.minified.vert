#undef G5
#ifdef NEVER_GENERATE_PREMULTIPLIED_PAINT_COLORS
#define G5 true
#elif defined(ENABLE_ADVANCED_BLEND)
#define G5 ENABLE_ADVANCED_BLEND
#else
#define G5 false
#endif
#undef z2
#ifdef ENABLE_FEATHER
#define z2 g
#else
#define z2 E
#endif
#ifdef VERTEX
g1(e0)
#if defined(DRAW_INTERIOR_TRIANGLES)||defined(FEATHER_ATLAS_BLIT)
L(0,N3,LB);
#else
L(0,g,VB);L(1,g,WB);
#endif
h1
#endif
l2 H0 X(0,g,f1);
#ifdef FEATHER_ATLAS_BLIT
H0 X(1,d,D2);
#elif!defined(RENDER_MODE_MSAA)
#ifdef DRAW_INTERIOR_TRIANGLES
OPTIONALLY_FLAT X(1,c,i1);
#else
H0 X(2,z2,O);
#endif
OPTIONALLY_FLAT X(3,c,B0);
#endif
#ifdef ENABLE_CLIPPING
#ifdef FEATHER_ATLAS_BLIT
OPTIONALLY_FLAT X(4,c,K3);
#else
OPTIONALLY_FLAT X(4,E,U1);
#endif
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_MSAA)
H0 X(5,g,M0);
#endif
#ifdef ENABLE_ADVANCED_BLEND
OPTIONALLY_FLAT X(6,c,e2);
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
Q2 X(7,a1,f3);X(8,d,n4);
#endif
#ifdef ENABLE_MODULATED_IMAGE
H0 X(9,R,A2);
#endif
f2
#ifdef VERTEX
#ifdef EMULATE_DYNAMIC_COLOR_WRITE_DISABLE
layout(push_constant)uniform Ui{float Lh;}Mh;
#endif
y1(HC,e0,F,B,v){
#if defined(DRAW_INTERIOR_TRIANGLES)||defined(FEATHER_ATLAS_BLIT)
M(B,F,LB,R);
#else
M(B,F,VB,g);M(B,F,WB,g);
#endif
V(f1,g);
#if defined(ENABLE_MODULATED_IMAGE)
V(A2,R);
#endif
#ifdef FEATHER_ATLAS_BLIT
V(D2,d);
#elif!defined(RENDER_MODE_MSAA)
#ifdef DRAW_INTERIOR_TRIANGLES
V(i1,c);
#else
V(O,z2);
#endif
V(B0,c);
#endif
#ifdef ENABLE_CLIPPING
#ifdef FEATHER_ATLAS_BLIT
V(K3,c);
#else
V(U1,E);
#endif
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_MSAA)
V(M0,g);
#endif
#ifdef ENABLE_ADVANCED_BLEND
V(e2,c);
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
V(f3,a1);V(n4,d);
#endif
bool ee=false;uint l0;d m0;
#ifdef RENDER_MODE_MSAA
N g9;
#endif
#ifdef FEATHER_ATLAS_BLIT
m0=Fb(LB,l0,
#ifdef RENDER_MODE_MSAA
g9,
#endif
D2 w3);
#elif defined(DRAW_INTERIOR_TRIANGLES)
m0=Gb(LB,l0
#ifdef RENDER_MODE_MSAA
,g9
#else
,i1
#endif
w3);
#else
g P;ee=!q9(VB,WB,v,l0,m0
#ifndef RENDER_MODE_MSAA
,P
#else
,g9
#endif
w3);
#ifndef RENDER_MODE_MSAA
#ifdef ENABLE_FEATHER
O=P;
#else
O.xy=Q7(P.xy);
#endif
#endif
#endif
a1 p1=O5(BD,l0);
#if!defined(FEATHER_ATLAS_BLIT)&&!defined(RENDER_MODE_MSAA)
B0=q8(l0,m.c6);if((p1.x&K9)!=0u)B0=-B0;
#endif
uint l3=p1.x&0xfu;
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING){uint Nh=(l3==Y7?p1.y:p1.x)>>16;c k1=q8(Nh,m.c6);if(l3==Y7)k1=-k1;
#ifdef FEATHER_ATLAS_BLIT
K3=k1;
#else
U1.x=k1;
#endif
}
#endif
#ifdef ENABLE_ADVANCED_BLEND
if(ENABLE_ADVANCED_BLEND){e2=float((p1.x>>4)&0xfu);}
#endif
d L0=m0;
#ifdef FRAMEBUFFER_BOTTOM_UP
L0.y=float(m.Lg)-L0.y;
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){f0 Z3=g2(J0(RB,l0*A3+2u));g G4=J0(RB,l0*A3+3u);
#ifndef RENDER_MODE_MSAA
M0=S7(Z3,G4.xy,L0);
#else
Ac(Z3,G4.xy,L0 w5);
#endif
}
#endif
if(l3==Nb){i j=unpackUnorm4x8(p1.y);if(G5){}else{j.xyz*=j.w;}f1=g(j);}
#if defined(ENABLE_CLIPPING)&&!defined(FEATHER_ATLAS_BLIT)
else if(ENABLE_CLIPPING&&l3==Y7){c H5=q8(p1.x>>16,m.c6);U1.y=H5;}
#endif
else{f0 mb=g2(J0(RB,l0*A3));g nb=J0(RB,l0*A3+1u);d x4=R0(mb,L0)+nb.xy;if(l3==M9||l3==Hf){f1.w=-uintBitsToFloat(p1.y);float Oh=nb.z;if(Oh>.9){f1.z=2.;}else{f1.z=nb.w;}if(l3==M9){f1.y=.0;f1.x=x4.x;}else{f1.z=-f1.z;f1.xy=x4.xy;}}}
#ifdef EMULATE_DYNAMIC_COLOR_WRITE_DISABLE
if(EMULATE_DYNAMIC_COLOR_WRITE_DISABLE){f1*=Mh.Lh;}
#endif
#if defined(ENABLE_MODULATED_IMAGE)
if(ENABLE_MODULATED_IMAGE&&(p1.x&If)!=0u){f0 mb=g2(J0(RB,l0*A3+4u));g fe=J0(RB,l0*A3+5u);d x4=R0(mb,L0)+fe.xy;A2=R(x4.x,x4.y,1.+fe.z);}else{A2=R(0.0,0.0,0.0);}
#endif
g W;if(!ee){W=M3(m0);
#ifdef POST_INVERT_Y
W.y=-W.y;
#endif
#ifdef RENDER_MODE_MSAA
W.z=ja(g9);
#elif defined(RENDER_MODE_CLOCKWISE_ATOMIC)
G P4=J0(QB,l0*4u+3u);f3=P4.xy;n4=m0+uintBitsToFloat(P4.zw);
#endif
}else{W=g(m.R2,m.R2,m.R2,m.R2);}a0(f1);
#if defined(ENABLE_MODULATED_IMAGE)
a0(A2);
#endif
#ifdef FEATHER_ATLAS_BLIT
a0(D2);
#elif!defined(RENDER_MODE_MSAA)
#ifdef DRAW_INTERIOR_TRIANGLES
a0(i1);
#else
a0(O);
#endif
a0(B0);
#endif
#ifdef ENABLE_CLIPPING
#ifdef FEATHER_ATLAS_BLIT
a0(K3);
#else
a0(U1);
#endif
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_MSAA)
a0(M0);
#endif
#ifdef ENABLE_ADVANCED_BLEND
a0(e2);
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
a0(f3);a0(n4);
#endif
z1(W);}
#endif
#ifdef FRAGMENT
Q3 R3 e i L7(g I5,
#ifdef ENABLE_MODULATED_IMAGE
R ob,
#endif
float n J6){i j;if(I5.w>=.0){j=Z4(I5);if(G5)j.w*=n;else j*=n;}else{float t=I5.z>.0?I5.x:length(I5.xy);t=clamp(t,.0,1.);float ge=abs(I5.z);float x=ge>1.?(1.-1./la)*t+(.5/la):(1./la)*t+ge;float Ph=-I5.w;j=n2(MD,Ob,d(x,Ph),.0);j.w*=n;if(G5){}else{j.xyz*=j.w;}}
#if defined(ENABLE_MODULATED_IMAGE)
if(ENABLE_MODULATED_IMAGE&&ob.z>0.0){c Qh=ob.z-1.;i G2=S6(JC,U5,ob.xy,Qh);if(G5)G2=C0(E6(G2),G2.w);j*=G2;}
#endif
return j;}
#if!defined(DRAW_INTERIOR_TRIANGLES)&&!defined(FEATHER_ATLAS_BLIT)
e c he(z2 P I3){
#ifdef ENABLE_FEATHER
if(ENABLE_FEATHER&&Pb(P))return y4(P d1);else
#endif
return min(P.x,P.y);}e c ie(z2 P I3){
#if defined(ENABLE_FEATHER)
if(ENABLE_FEATHER&&Qb(P))return c8(P d1);else
#endif
return P.x;}e c pb(z2 P I3){if(T5(P))return he(P d1);else return ie(P d1);}e c Rh(c Q4,z2 P I3){if(T5(P)){c r0=he(P d1);return max(r0,Q4);}else{c r0=ie(P d1);return Q4+r0;}}
#endif
#endif
