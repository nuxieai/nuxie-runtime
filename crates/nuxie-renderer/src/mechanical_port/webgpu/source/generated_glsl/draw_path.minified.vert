#undef H5
#ifdef NEVER_GENERATE_PREMULTIPLIED_PAINT_COLORS
#define H5 true
#elif defined(ENABLE_ADVANCED_BLEND)
#define H5 ENABLE_ADVANCED_BLEND
#else
#define H5 false
#endif
#undef B2
#ifdef ENABLE_FEATHER
#define B2 f
#else
#define B2 E
#endif
#ifdef VERTEX
f1(f0)
#if defined(DRAW_INTERIOR_TRIANGLES)||defined(FEATHER_ATLAS_BLIT)
J(0,N3,KB);
#else
J(0,f,UB);J(1,f,VB);
#endif
g1
#endif
p2 H0 V(0,f,V1);
#ifdef FEATHER_ATLAS_BLIT
H0 V(1,c,F2);
#elif!defined(RENDER_MODE_DEPTH_STENCIL)
#ifdef DRAW_INTERIOR_TRIANGLES
OPTIONALLY_FLAT V(1,d,h1);
#else
H0 V(2,B2,M);
#endif
OPTIONALLY_FLAT V(3,d,C0);
#endif
#ifdef ENABLE_CLIPPING
#ifdef FEATHER_ATLAS_BLIT
OPTIONALLY_FLAT V(4,d,K3);
#else
OPTIONALLY_FLAT V(4,E,W1);
#endif
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
H0 V(5,f,M0);
#endif
#ifdef ENABLE_ADVANCED_BLEND
OPTIONALLY_FLAT V(6,d,g2);
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
S2 V(7,a1,g3);V(8,c,p4);
#endif
#ifdef ENABLE_MODULATED_IMAGE
H0 V(9,Q,C2);
#endif
h2
#ifdef VERTEX
#ifdef EMULATE_DYNAMIC_COLOR_WRITE_DISABLE
Jd(xh)Kd(float,ii)Ld(ji)
#endif
y1(FC,f0,F,B,v){
#if defined(DRAW_INTERIOR_TRIANGLES)||defined(FEATHER_ATLAS_BLIT)
K(B,F,KB,Q);
#else
K(B,F,UB,f);K(B,F,VB,f);
#endif
T(V1,f);
#if defined(ENABLE_MODULATED_IMAGE)
T(C2,Q);
#endif
#ifdef FEATHER_ATLAS_BLIT
T(F2,c);
#elif!defined(RENDER_MODE_DEPTH_STENCIL)
#ifdef DRAW_INTERIOR_TRIANGLES
T(h1,d);
#else
T(M,B2);
#endif
T(C0,d);
#endif
#ifdef ENABLE_CLIPPING
#ifdef FEATHER_ATLAS_BLIT
T(K3,d);
#else
T(W1,E);
#endif
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
T(M0,f);
#endif
#ifdef ENABLE_ADVANCED_BLEND
T(g2,d);
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
T(g3,a1);T(p4,c);
#endif
bool te=false;uint m0;c j0;
#ifdef RENDER_MODE_DEPTH_STENCIL
L i9;
#endif
#ifdef FEATHER_ATLAS_BLIT
j0=Kb(KB,m0,
#ifdef RENDER_MODE_DEPTH_STENCIL
i9,
#endif
F2 w3);
#elif defined(DRAW_INTERIOR_TRIANGLES)
j0=Lb(KB,m0
#ifdef RENDER_MODE_DEPTH_STENCIL
,i9
#else
,h1
#endif
w3);
#else
f N;te=!v9(UB,VB,v,m0,j0
#ifndef RENDER_MODE_DEPTH_STENCIL
,N
#else
,i9
#endif
w3);
#ifndef RENDER_MODE_DEPTH_STENCIL
#ifdef ENABLE_FEATHER
M=N;
#else
M.xy=S7(N.xy);
#endif
#endif
#endif
a1 o1=Q5(DD,m0);
#if!defined(FEATHER_ATLAS_BLIT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
C0=v8(m0,n.e6);if((o1.x&L9)!=0u)C0=-C0;
#endif
uint S3=o1.x&0xfu;
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING){uint ki=(S3==a8?o1.y:o1.x)>>16;d j1=v8(ki,n.e6);if(S3==a8)j1=-j1;
#ifdef FEATHER_ATLAS_BLIT
K3=j1;
#else
W1.x=j1;
#endif
}
#endif
#ifdef ENABLE_ADVANCED_BLEND
if(ENABLE_ADVANCED_BLEND){g2=float((o1.x>>4)&0xfu);}
#endif
c q0=j0;
#ifdef FRAMEBUFFER_BOTTOM_UP
q0.y=float(n.Nb)-q0.y;
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){d0 Z3=I1(K0(QB,m0*A3+2u));f H4=K0(QB,m0*A3+3u);
#ifndef RENDER_MODE_DEPTH_STENCIL
M0=U7(Z3,H4.xy,q0);
#else
Kc(Z3,H4.xy,q0 x5);
#endif
}
#endif
if(S3==Vb){i j=unpackUnorm4x8(o1.y);if(H5){}else{j.xyz*=j.w;}V1=f(j);}
#if defined(ENABLE_CLIPPING)&&!defined(FEATHER_ATLAS_BLIT)
else if(ENABLE_CLIPPING&&S3==a8){d I5=v8(o1.x>>16,n.e6);W1.y=I5;}
#endif
else{d0 li=I1(K0(QB,m0*A3));f ue=K0(QB,m0*A3+1u);V1=Ob(q0,li,ue.xy,float(S3),ue.zw,uintBitsToFloat(o1.y));V1.w=-V1.w;}
#ifdef EMULATE_DYNAMIC_COLOR_WRITE_DISABLE
if(EMULATE_DYNAMIC_COLOR_WRITE_DISABLE){V1*=ji.ii;}
#endif
#if defined(ENABLE_MODULATED_IMAGE)
if(ENABLE_MODULATED_IMAGE&&(o1.x&cg)!=0u){d0 mi=I1(K0(QB,m0*A3+4u));f ve=K0(QB,m0*A3+5u);c g4=N0(mi,q0)+ve.xy;C2=Q(g4.x,g4.y,1.+ve.z);}else{C2=Q(0.0,0.0,0.0);}
#endif
f W;if(!te){W=M3(j0);
#ifdef POST_INVERT_Y
W.y=-W.y;
#endif
#ifdef RENDER_MODE_DEPTH_STENCIL
W.z=na(i9);
#elif defined(RENDER_MODE_CLOCKWISE_ATOMIC)
X R4=K0(PB,m0*4u+3u);g3=R4.xy;p4=j0+uintBitsToFloat(R4.zw);
#endif
}else{W=f(n.T2,n.T2,n.T2,n.T2);}a0(V1);
#if defined(ENABLE_MODULATED_IMAGE)
a0(C2);
#endif
#ifdef FEATHER_ATLAS_BLIT
a0(F2);
#elif!defined(RENDER_MODE_DEPTH_STENCIL)
#ifdef DRAW_INTERIOR_TRIANGLES
a0(h1);
#else
a0(M);
#endif
a0(C0);
#endif
#ifdef ENABLE_CLIPPING
#ifdef FEATHER_ATLAS_BLIT
a0(K3);
#else
a0(W1);
#endif
#endif
#if defined(ENABLE_CLIP_RECT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
a0(M0);
#endif
#ifdef ENABLE_ADVANCED_BLEND
a0(g2);
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
a0(g3);a0(p4);
#endif
z1(W);}
#endif
#ifdef FRAGMENT
P3 Q3 e i M7(f N7,
#ifdef ENABLE_MODULATED_IMAGE
Q tb,
#endif
float o L6){i j;if(N7.w>=.0){j=c5(N7);if(H5)j.w*=o;else j*=o;}else{N7.w=-N7.w;c T9=cc(N7);j=i2(ED,N9,T9,.0);j.w*=o;if(H5){}else{j.xyz*=j.w;}}
#if defined(ENABLE_MODULATED_IMAGE)
if(ENABLE_MODULATED_IMAGE&&tb.z>0.0){d ni=tb.z-1.;i j2=U6(HC,W5,tb.xy,ni);if(H5)j2=D0(G6(j2),j2.w);j*=j2;}
#endif
return j;}
#if!defined(DRAW_INTERIOR_TRIANGLES)&&!defined(FEATHER_ATLAS_BLIT)
e d we(B2 N I3){
#ifdef ENABLE_FEATHER
if(ENABLE_FEATHER&&Yb(N))return y4(N d1);else
#endif
return min(N.x,N.y);}e d xe(B2 N I3){
#if defined(ENABLE_FEATHER)
if(ENABLE_FEATHER&&Zb(N))return e8(N d1);else
#endif
return N.x;}e d ub(B2 N I3){if(V5(N))return we(N d1);else return xe(N d1);}e d oi(d S4,B2 N I3){if(V5(N)){d w0=we(N d1);return max(w0,S4);}else{d w0=xe(N d1);return S4+w0;}}
#endif
#endif
