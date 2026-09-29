#define D3 3.14159265359
#define q8 6.28318530718
#define Y6 1.57079632679
#ifndef RENDER_MODE_DEPTH_STENCIL
#define q4 float(.5)
#else
#define q4 float(.0)
#endif
#define M3(l) p8(l,m.of,m.pf)
#ifdef TESS_TEXTURE_FLOATING_POINT
#define nc(U,f,a) i5(U,f,a)
#define E4 g
#define Z9(q) q
#define Z5(q) q
#define aa(q) uintBitsToFloat(q)
#define j5(q) floatBitsToUint(q)
#else
#define nc(U,f,a) F4(U,f,a)
#define E4 H
#define Z9(q) floatBitsToUint(q)
#define Z5(q) uintBitsToFloat(q)
#define aa(q) q
#define j5(q) q
#endif
#define qf(a,l,r8) q1(a,Y(l)+Y(-1,0))r8,q1(a,Y(l)+Y(0,0))r8,q1(a,Y(l)+Y(0,-1))r8,q1(a,Y(l)+Y(-1,-1))r8
#define k5(q) Z6(YC,ba,q,oc,float(oc),.0).x
#define qc(q) Z6(YC,ba,q,pc,float(pc),.0).x
#ifdef rc
e c X4(float x){return x;}e c a6(uint x){return float(x);}e c rf(N x){return float(x);}e c ca(int x){return float(x);}e i d5(g xyzw){return xyzw;}e E S7(d xy){return xy;}e i ic(H xyzw){return vec4(xyzw);}e N c6(c x){return uint(x);}e N X1(uint x){return x;}
#else
e c X4(float x){return(c)x;}e c a6(uint x){return(c)x;}e c rf(N x){return(c)x;}e c ca(int x){return(c)x;}e i d5(g xyzw){return(i)xyzw;}e E S7(d xy){return(E)xy;}e i ic(H xyzw){return(i)xyzw;}e N c6(c x){return(N)x;}e N X1(uint x){return(N)x;}
#endif
e c G0(c x){return x;}e E B2(E xy){return xy;}e E B2(c x,c y){E T;T.x=x,T.y=y;return T;}e E B2(c x){E T;T.x=x,T.y=x;return T;}e d O6(float x){return d(x,x);}e A Q0(c x,c y,c z){A T;T.x=x,T.y=y,T.z=z;return T;}e A Q0(c x){A T;T.x=x,T.y=x,T.z=x;return T;}e i C0(c x,c y,c z,c w){i T;T.x=x,T.y=y,T.z=z,T.w=w;return T;}e i C0(A xyz,c w){i T;T.xyz=xyz;T.w=w;return T;}e i C0(c x){i T;T.x=x,T.y=x,T.z=x,T.w=x;return T;}e i C0(i x){return x;}e G4 sf(bool b){return G4(b,b);}e a7 fi(A o,A b,A H1){a7 T;T[0]=o;T[1]=b;T[2]=H1;return T;}e c7 gi(A o,A b){c7 T;T[0]=o;T[1]=b;return T;}e H4 hi(i o,i b,i H1,i tf){H4 T;T[0]=o;T[1]=b;T[2]=H1;T[3]=tf;return T;}e f0 h2(g x){return f0(x.xy,x.zw);}e uint Vb(N x){return x;}e d d6(d o,d b,float t){return(b-o)*t+o;}e c v8(uint sc,uint e6){return sc==0u?.0:unpackHalf2x16((sc+uf)*e6).x;}e float tc(d j2){j2=normalize(j2);float e1=acos(clamp(j2.x,-1.,1.));return j2.y>=.0?e1:-e1;}e i ii(i j){return C0(j.xyz*j.w,j.w);}e A H6(i da){return da.xyz*(da.w!=.0?1./da.w:.0);}e c h3(E d7){return min(d7.x,d7.y);}e c h3(A uc){return min(h3(uc.xy),uc.z);}e c h3(i vc){E d7=min(vc.xy,vc.zw);c vf=min(d7.x,d7.y);return vf;}e c N5(E e7){return max(e7.x,e7.y);}e c N5(A wc){return max(N5(wc.xy),wc.z);}e c N5(i xc){E e7=max(xc.xy,xc.zw);c wf=max(e7.x,e7.y);return wf;}e float F9(d x){return abs(x.x)+abs(x.y);}e c ea(c x,c fa,c ga){
#if defined(GL_RENDERER_MALI)||defined(VULKAN_VENDOR_ARM)
#ifdef VULKAN_VENDOR_ARM
if(VULKAN_VENDOR_ARM)
#endif
{if(x<ga)if(x>fa)return x;else return fa;else return ga;}
#endif
return clamp(x,fa,ga);}e c yc(d L0,c C2,c o3){c xf=fract(0.06711056*L0.x+0.00583715*L0.y);c yf=fract(52.9829189*xf);return(yf*C2)+o3;}
#if 0
e c ji(d L0,float C2,float o3){int x=int(L0.x);int y=int(L0.y);int zc=(x^y);int b=(y>>1)&1;b|=(zc&2);b|=(y&1)<<2;b|=(zc&1)<<3;float zf=float(b);c Af=X4(zf)/16.0;return(Af*C2)+o3;}e c ki(d L0,float C2,float o3){L0.y*=0.5;L0.x=fract(L0.x*0.5+L0.y);L0.y=fract(L0.y);float P3=(L0.y*0.5+L0.x);return(P3*C2)+o3;}
#endif
#ifdef ENABLE_DITHER
e c ha(d L0,c C2,c o3){return ENABLE_DITHER?yc(L0,C2,o3):.0;}e A F2(A j,c f7,d L0,c C2,c o3){return(ENABLE_DITHER&&f7!=.0)?(yc(L0,C2,o3)+j):j;}e A F2(A j,c f7,c Ac){return(ENABLE_DITHER&&f7!=.0)?(Ac+j):j;}
#else
e c ha(d L0,float C2,float o3){return 0.;}e A F2(A j,c f7,d L0,c C2,c o3){return j;}e A F2(A j,c f7,c Ac){return j;}
#endif
#ifdef VERTEX
e g p8(d Bc,float Bf,float Cc){return g(Bc.x*Bf-1.,Bc.y*Cc-sign(Cc),0.,1.);}
#ifndef RENDER_MODE_DEPTH_STENCIL
e g U7(f0 Z3,d I4,d ia){d ja=abs(Z3[0])+abs(Z3[1]);if(ja.x!=.0&&ja.y!=.0){d K=1./ja;d l5=R0(Z3,ia)+I4;const float Cf=.5;return g(l5,-l5)*K.xyxy+K.xyxy+Cf;}else{return I4.xyxy;}}
#else
e float ka(uint la){return 1.-float(la)*(2./32768.);}
#ifdef ENABLE_CLIP_RECT
e void Dc(f0 Z3,d I4,d ia g7){
#ifndef DISABLE_CLIP_DISTANCE_FOR_UBERSHADERS
if(any(notEqual(g(Z3),g(.0,.0,.0,.0)))){d l5=R0(Z3,ia)+I4.xy;gl_ClipDistance[0]=l5.x+1.;gl_ClipDistance[1]=l5.y+1.;gl_ClipDistance[2]=1.-l5.x;gl_ClipDistance[3]=1.-l5.y;}else{gl_ClipDistance[0]=gl_ClipDistance[1]=gl_ClipDistance[2]=gl_ClipDistance[3]=I4.x-.5;}
#endif
}
#endif
#endif
#endif
#ifdef FRAGMENT
#ifdef NEEDS_GAMMA_CORRECTION
e c m3(c j){return(j<=0.04045)?j/12.92:pow(abs((j+0.055)/1.055),2.4);}e A m3(A j){return Q0(m3(j.x),m3(j.y),m3(j.z));}e i m3(i j){return C0(m3(j.xyz),j.w);}
#endif
#endif
#if defined(FRAGMENT)&&defined(RENDER_MODE_DEPTH_STENCIL)&&!defined(FIXED_FUNCTION_COLOR_OUTPUT)
e i ma(H4 h7,int w8){if(w8==0xf){return(h7[0]+h7[1]+h7[2]+h7[3])*.25;}else{i Df=g(notEqual(w8&f6(1,2,4,8),f6(0,0,0,0)));i T=R0(h7,Df);int x8=(w8&5)+((w8>>1)&5);x8=(x8&3)+(x8>>2);T*=1./float(x8);return T;}}
#endif
