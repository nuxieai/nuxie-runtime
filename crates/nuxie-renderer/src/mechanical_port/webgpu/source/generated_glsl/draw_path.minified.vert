#undef F5
#ifdef NEVER_GENERATE_PREMULTIPLIED_PAINT_COLORS
#define F5 true
#elif defined(ENABLE_ADVANCED_BLEND)
#define F5 ENABLE_ADVANCED_BLEND
#else
#define F5 false
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
L(0,L3,KB);
#else
L(0,g,UB);L(1,g,VB);
#endif
h1
#endif
k2 J0 W(0,g,X0);
#ifdef FEATHER_ATLAS_BLIT
J0 W(1,d,C2);
#elif!defined(RENDER_MODE_MSAA)
#ifdef DRAW_INTERIOR_TRIANGLES
OPTIONALLY_FLAT W(1,c,i1);
#else
J0 W(2,z2,O);
#endif
OPTIONALLY_FLAT W(3,c,B0);
#endif
#ifdef ENABLE_CLIPPING
#ifdef FEATHER_ATLAS_BLIT
OPTIONALLY_FLAT W(4,c,I3);
#else
OPTIONALLY_FLAT W(4,E,U1);
#endif
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_MSAA)
J0 W(5,g,L0);
#endif
#ifdef ENABLE_ADVANCED_BLEND
OPTIONALLY_FLAT W(6,c,e2);
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
O2 W(7,c1,d3);W(8,d,l4);
#endif
f2
#ifdef VERTEX
#ifdef EMULATE_DYNAMIC_COLOR_WRITE_DISABLE
layout(push_constant)uniform Pi{float Fh;}Gh;
#endif
y1(GC,e0,F,B,r){
#if defined(DRAW_INTERIOR_TRIANGLES)||defined(FEATHER_ATLAS_BLIT)
M(B,F,KB,c0);
#else
M(B,F,UB,g);M(B,F,VB,g);
#endif
V(X0,g);
#ifdef FEATHER_ATLAS_BLIT
V(C2,d);
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
V(I3,c);
#else
V(U1,E);
#endif
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_MSAA)
V(L0,g);
#endif
#ifdef ENABLE_ADVANCED_BLEND
V(e2,c);
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
V(d3,c1);V(l4,d);
#endif
bool ce=false;uint o0;d l0;
#ifdef RENDER_MODE_MSAA
N e9;
#endif
#ifdef FEATHER_ATLAS_BLIT
l0=Db(KB,o0,
#ifdef RENDER_MODE_MSAA
e9,
#endif
C2 v3);
#elif defined(DRAW_INTERIOR_TRIANGLES)
l0=Eb(KB,o0
#ifdef RENDER_MODE_MSAA
,e9
#else
,i1
#endif
v3);
#else
g P;ce=!q9(UB,VB,r,o0,l0
#ifndef RENDER_MODE_MSAA
,P
#else
,e9
#endif
v3);
#ifndef RENDER_MODE_MSAA
#ifdef ENABLE_FEATHER
O=P;
#else
O.xy=O7(P.xy);
#endif
#endif
#endif
c1 p1=M5(AD,o0);
#if!defined(FEATHER_ATLAS_BLIT)&&!defined(RENDER_MODE_MSAA)
B0=o8(o0,n.Z5);if((p1.x&K9)!=0u)B0=-B0;
#endif
uint j3=p1.x&0xfu;
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING){uint Hh=(j3==W7?p1.y:p1.x)>>16;c k1=o8(Hh,n.Z5);if(j3==W7)k1=-k1;
#ifdef FEATHER_ATLAS_BLIT
I3=k1;
#else
U1.x=k1;
#endif
}
#endif
#ifdef ENABLE_ADVANCED_BLEND
if(ENABLE_ADVANCED_BLEND){e2=float((p1.x>>4)&0xfu);}
#endif
d K0=l0;
#ifdef FRAMEBUFFER_BOTTOM_UP
K0.y=float(n.Gg)-K0.y;
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){g0 X3=l2(N0(RB,o0*4u+2u));g E4=N0(RB,o0*4u+3u);
#ifndef RENDER_MODE_MSAA
L0=Q7(X3,E4.xy,K0);
#else
yc(X3,E4.xy,K0 v5);
#endif
}
#endif
if(j3==Lb){i j=unpackUnorm4x8(p1.y);if(F5){}else{j.xyz*=j.w;}X0=g(j);}
#if defined(ENABLE_CLIPPING)&&!defined(FEATHER_ATLAS_BLIT)
else if(ENABLE_CLIPPING&&j3==W7){c G5=o8(p1.x>>16,n.Z5);U1.y=G5;}
#endif
else{g0 Ih=l2(N0(RB,o0*4u));g f9=N0(RB,o0*4u+1u);d V4=U0(Ih,K0)+f9.xy;if(j3==M9||j3==Ef){X0.w=-uintBitsToFloat(p1.y);float Jh=f9.z;if(Jh>.9){X0.z=2.;}else{X0.z=f9.w;}if(j3==M9){X0.y=.0;X0.x=V4.x;}else{X0.z=-X0.z;X0.xy=V4.xy;}}else{float g9=uintBitsToFloat(p1.y);float mb=f9.z;X0=g(V4.x,V4.y,g9,-2.-mb);}}
#ifdef EMULATE_DYNAMIC_COLOR_WRITE_DISABLE
if(EMULATE_DYNAMIC_COLOR_WRITE_DISABLE){X0*=Gh.Fh;}
#endif
g U;if(!ce){U=K3(l0);
#ifdef POST_INVERT_Y
U.y=-U.y;
#endif
#ifdef RENDER_MODE_MSAA
U.z=ja(e9);
#elif defined(RENDER_MODE_CLOCKWISE_ATOMIC)
G N4=N0(PB,o0*4u+3u);d3=N4.xy;l4=l0+uintBitsToFloat(N4.zw);
#endif
}else{U=g(n.P2,n.P2,n.P2,n.P2);}a0(X0);
#ifdef FEATHER_ATLAS_BLIT
a0(C2);
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
a0(I3);
#else
a0(U1);
#endif
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_MSAA)
a0(L0);
#endif
#ifdef ENABLE_ADVANCED_BLEND
a0(e2);
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
a0(d3);a0(l4);
#endif
z1(U);}
#endif
#ifdef FRAGMENT
O3 P3 e i J7(g q3,float o H6){i j;if(q3.w>=.0){j=Y4(q3);if(F5)j.w*=o;else j*=o;}else if(q3.w>-1.){float t=q3.z>.0?q3.x:length(q3.xy);t=clamp(t,.0,1.);float de=abs(q3.z);float x=de>1.?(1.-1./la)*t+(.5/la):(1./la)*t+de;float Kh=-q3.w;j=n2(LD,Mb,d(x,Kh),.0);j.w*=o;if(F5){}else{j.xyz*=j.w;}}else{c mb=-q3.w-2.;j=Q6(IC,S5,q3.xy,mb);c g9=q3.z*o;if(F5)j=C0(C6(j),j.w*g9);else j*=g9;}return j;}
#if!defined(DRAW_INTERIOR_TRIANGLES)&&!defined(FEATHER_ATLAS_BLIT)
e c ee(z2 P G3){
#ifdef ENABLE_FEATHER
if(ENABLE_FEATHER&&Nb(P))return v4(P e1);else
#endif
return min(P.x,P.y);}e c fe(z2 P G3){
#if defined(ENABLE_FEATHER)
if(ENABLE_FEATHER&&Ob(P))return Z7(P e1);else
#endif
return P.x;}e c nb(z2 P G3){if(R5(P))return ee(P e1);else return fe(P e1);}e c Lh(c O4,z2 P G3){if(R5(P)){c r0=ee(P e1);return max(r0,O4);}else{c r0=fe(P e1);return O4+r0;}}
#endif
#endif
