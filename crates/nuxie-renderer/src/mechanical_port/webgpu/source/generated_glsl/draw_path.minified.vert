#undef C6
#ifdef ENABLE_ADVANCED_BLEND
#define C6 ENABLE_ADVANCED_BLEND
#else
#define C6 false
#endif
#undef A2
#ifdef ENABLE_FEATHER
#define A2 f
#else
#define A2 E
#endif
#ifdef VERTEX
f1(f0)
#if defined(DRAW_INTERIOR_TRIANGLES)||defined(FEATHER_ATLAS_BLIT)
J(0,O3,KB);
#else
J(0,f,UB);J(1,f,VB);
#endif
g1
#endif
p2 H0 V(0,f,V1);
#ifdef FEATHER_ATLAS_BLIT
H0 V(1,c,E2);
#elif!defined(RENDER_MODE_DEPTH_STENCIL)
#ifdef DRAW_INTERIOR_TRIANGLES
OPTIONALLY_FLAT V(1,d,h1);
#else
H0 V(2,A2,M);
#endif
OPTIONALLY_FLAT V(3,d,C0);
#endif
#ifdef ENABLE_CLIPPING
#ifdef FEATHER_ATLAS_BLIT
OPTIONALLY_FLAT V(4,d,L3);
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
H0 V(9,Q,B2);
#endif
h2
#ifdef VERTEX
#ifdef EMULATE_DYNAMIC_COLOR_WRITE_DISABLE
Ld(zh)Md(float,ki)Nd(li)
#endif
y1(FC,f0,F,B,v){
#if defined(DRAW_INTERIOR_TRIANGLES)||defined(FEATHER_ATLAS_BLIT)
K(B,F,KB,Q);
#else
K(B,F,UB,f);K(B,F,VB,f);
#endif
T(V1,f);
#if defined(ENABLE_MODULATED_IMAGE)
T(B2,Q);
#endif
#ifdef FEATHER_ATLAS_BLIT
T(E2,c);
#elif!defined(RENDER_MODE_DEPTH_STENCIL)
#ifdef DRAW_INTERIOR_TRIANGLES
T(h1,d);
#else
T(M,A2);
#endif
T(C0,d);
#endif
#ifdef ENABLE_CLIPPING
#ifdef FEATHER_ATLAS_BLIT
T(L3,d);
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
bool ve=false;uint m0;c j0;
#ifdef RENDER_MODE_DEPTH_STENCIL
L i9;
#endif
#ifdef FEATHER_ATLAS_BLIT
j0=Lb(KB,m0,
#ifdef RENDER_MODE_DEPTH_STENCIL
i9,
#endif
E2 x3);
#elif defined(DRAW_INTERIOR_TRIANGLES)
j0=Mb(KB,m0
#ifdef RENDER_MODE_DEPTH_STENCIL
,i9
#else
,h1
#endif
x3);
#else
f N;ve=!w9(UB,VB,v,m0,j0
#ifndef RENDER_MODE_DEPTH_STENCIL
,N
#else
,i9
#endif
x3);
#ifndef RENDER_MODE_DEPTH_STENCIL
#ifdef ENABLE_FEATHER
M=N;
#else
M.xy=S7(N.xy);
#endif
#endif
#endif
a1 o1=P5(DD,m0);
#if!defined(FEATHER_ATLAS_BLIT)&&!defined(RENDER_MODE_DEPTH_STENCIL)
C0=v8(m0,l.d6);if((o1.x&M9)!=0u)C0=-C0;
#endif
uint T3=o1.x&0xfu;
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING){uint mi=(T3==a8?o1.y:o1.x)>>16;d j1=v8(mi,l.d6);if(T3==a8)j1=-j1;
#ifdef FEATHER_ATLAS_BLIT
L3=j1;
#else
W1.x=j1;
#endif
}
#endif
#ifdef ENABLE_ADVANCED_BLEND
if(ENABLE_ADVANCED_BLEND){g2=float((o1.x>>4)&0xfu);}
#endif
c r0=j0;
#ifdef ENABLE_RENDER_TARGET_BOTTOM_UP
if(l.Ob!=0u){r0.y=float(l.Pb)-r0.y;}
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){d0 a4=I1(K0(QB,m0*B3+2u));f H4=K0(QB,m0*B3+3u);
#ifndef RENDER_MODE_DEPTH_STENCIL
M0=U7(a4,H4.xy,r0);
#else
Mc(a4,H4.xy,r0 x5);
#endif
}
#endif
if(T3==Xb){i j=unpackUnorm4x8(o1.y);if(C6){}else{j.xyz*=j.w;}V1=f(j);}
#if defined(ENABLE_CLIPPING)&&!defined(FEATHER_ATLAS_BLIT)
else if(ENABLE_CLIPPING&&T3==a8){d H5=v8(o1.x>>16,l.d6);W1.y=H5;}
#endif
else{d0 ni=I1(K0(QB,m0*B3));f we=K0(QB,m0*B3+1u);V1=Qb(r0,ni,we.xy,float(T3),we.zw,uintBitsToFloat(o1.y));V1.w=-V1.w;}
#ifdef EMULATE_DYNAMIC_COLOR_WRITE_DISABLE
if(EMULATE_DYNAMIC_COLOR_WRITE_DISABLE){V1*=li.ki;}
#endif
#if defined(ENABLE_MODULATED_IMAGE)
if(ENABLE_MODULATED_IMAGE&&(o1.x&eg)!=0u){d0 oi=I1(K0(QB,m0*B3+4u));f xe=K0(QB,m0*B3+5u);c h4=N0(oi,r0)+xe.xy;B2=Q(h4.x,h4.y,1.+xe.z);}else{B2=Q(0.0,0.0,0.0);}
#endif
f W;if(!ve){W=N3(j0);
#ifdef POST_INVERT_Y
W.y=-W.y;
#endif
#ifdef RENDER_MODE_DEPTH_STENCIL
W.z=oa(i9);
#elif defined(RENDER_MODE_CLOCKWISE_ATOMIC)
X R4=K0(PB,m0*4u+3u);g3=R4.xy;p4=j0+uintBitsToFloat(R4.zw);
#endif
}else{W=f(l.T2,l.T2,l.T2,l.T2);}a0(V1);
#if defined(ENABLE_MODULATED_IMAGE)
a0(B2);
#endif
#ifdef FEATHER_ATLAS_BLIT
a0(E2);
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
a0(L3);
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
Q3 R3 e i M7(f N7,
#ifdef ENABLE_MODULATED_IMAGE
Q ub,
#endif
float o M6){i j;if(N7.w>=.0){j=a5(N7);if(C6)j.w*=o;else j*=o;}else{N7.w=-N7.w;c U9=ec(N7);j=i2(ED,O9,U9,.0);j.w*=o;if(C6){}else{j.xyz*=j.w;}}
#if defined(ENABLE_MODULATED_IMAGE)
if(ENABLE_MODULATED_IMAGE&&ub.z>0.0){d pi=ub.z-1.;i j2=V6(HC,V5,ub.xy,pi);if(C6)j2=D0(G6(j2),j2.w);j*=j2;}
#endif
return j;}
#if!defined(DRAW_INTERIOR_TRIANGLES)&&!defined(FEATHER_ATLAS_BLIT)
e d ye(A2 N I3){
#ifdef ENABLE_FEATHER
if(ENABLE_FEATHER&&ac(N))return y4(N d1);else
#endif
return min(N.x,N.y);}e d ze(A2 N I3){
#if defined(ENABLE_FEATHER)
if(ENABLE_FEATHER&&bc(N))return e8(N d1);else
#endif
return N.x;}e d vb(A2 N I3){if(U5(N))return ye(N d1);else return ze(N d1);}e d qi(d S4,A2 N I3){if(U5(N)){d x0=ye(N d1);return max(x0,S4);}else{d x0=ze(N d1);return S4+x0;}}
#endif
#endif
