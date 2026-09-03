#define D3 3.14159265359
#define p8 6.28318530718
#define W6 1.57079632679
#ifndef RENDER_MODE_MSAA
#define p4 float(.5)
#else
#define p4 float(.0)
#endif
#define M3(l) o8(l,m.kf,m.lf)
#ifdef TESS_TEXTURE_FLOATING_POINT
#define lc(U,f,a) g5(U,f,a)
#define D4 g
#define Y9(q) q
#define Y5(q) q
#define Z9(q) uintBitsToFloat(q)
#define h5(q) floatBitsToUint(q)
#else
#define lc(U,f,a) E4(U,f,a)
#define D4 G
#define Y9(q) floatBitsToUint(q)
#define Y5(q) uintBitsToFloat(q)
#define Z9(q) q
#define h5(q) q
#endif
#define mf(a,l,q8) q1(a,Y(l)+Y(-1,0))q8,q1(a,Y(l)+Y(0,0))q8,q1(a,Y(l)+Y(0,-1))q8,q1(a,Y(l)+Y(-1,-1))q8
#define i5(q) X6(YC,aa,q,mc,float(mc),.0).x
#define oc(q) X6(YC,aa,q,nc,float(nc),.0).x
#ifdef pc
e c V4(float x){return x;}e c Z5(uint x){return float(x);}e c nf(N x){return float(x);}e c ba(int x){return float(x);}e i a5(g xyzw){return xyzw;}e E R7(d xy){return xy;}e i gc(G xyzw){return vec4(xyzw);}e N a6(c x){return uint(x);}e N X1(uint x){return x;}
#else
e c V4(float x){return(c)x;}e c Z5(uint x){return(c)x;}e c nf(N x){return(c)x;}e c ba(int x){return(c)x;}e i a5(g xyzw){return(i)xyzw;}e E R7(d xy){return(E)xy;}e i gc(G xyzw){return(i)xyzw;}e N a6(c x){return(N)x;}e N X1(uint x){return(N)x;}
#endif
e c G0(c x){return x;}e E B2(E xy){return xy;}e E B2(c x,c y){E T;T.x=x,T.y=y;return T;}e E B2(c x){E T;T.x=x,T.y=x;return T;}e d M6(float x){return d(x,x);}e A Q0(c x,c y,c z){A T;T.x=x,T.y=y,T.z=z;return T;}e A Q0(c x){A T;T.x=x,T.y=x,T.z=x;return T;}e i C0(c x,c y,c z,c w){i T;T.x=x,T.y=y,T.z=z,T.w=w;return T;}e i C0(A xyz,c w){i T;T.xyz=xyz;T.w=w;return T;}e i C0(c x){i T;T.x=x,T.y=x,T.z=x,T.w=x;return T;}e i C0(i x){return x;}e F4 of(bool b){return F4(b,b);}e Y6 ai(A o,A b,A G1){Y6 T;T[0]=o;T[1]=b;T[2]=G1;return T;}e Z6 bi(A o,A b){Z6 T;T[0]=o;T[1]=b;return T;}e j5 ci(i o,i b,i G1,i pf){j5 T;T[0]=o;T[1]=b;T[2]=G1;T[3]=pf;return T;}e f0 h2(g x){return f0(x.xy,x.zw);}e uint Tb(N x){return x;}e d c6(d o,d b,float t){return(b-o)*t+o;}e c r8(uint qc,uint d6){return qc==0u?.0:unpackHalf2x16((qc+qf)*d6).x;}e float rc(d j2){j2=normalize(j2);float e1=acos(clamp(j2.x,-1.,1.));return j2.y>=.0?e1:-e1;}e i di(i j){return C0(j.xyz*j.w,j.w);}e A F6(i ca){return ca.xyz*(ca.w!=.0?1./ca.w:.0);}e c h3(E a7){return min(a7.x,a7.y);}e c h3(A sc){return min(h3(sc.xy),sc.z);}e c h3(i tc){E a7=min(tc.xy,tc.zw);c rf=min(a7.x,a7.y);return rf;}e c M5(E c7){return max(c7.x,c7.y);}e c M5(A uc){return max(M5(uc.xy),uc.z);}e c M5(i vc){E c7=max(vc.xy,vc.zw);c sf=max(c7.x,c7.y);return sf;}e float E9(d x){return abs(x.x)+abs(x.y);}e c da(c x,c ea,c fa){
#if defined(GL_RENDERER_MALI)||defined(VULKAN_VENDOR_ARM)
#ifdef VULKAN_VENDOR_ARM
if(VULKAN_VENDOR_ARM)
#endif
{if(x<fa)if(x>ea)return x;else return ea;else return fa;}
#endif
return clamp(x,ea,fa);}e c wc(d L0,c C2,c o3){c tf=fract(0.06711056*L0.x+0.00583715*L0.y);c uf=fract(52.9829189*tf);return(uf*C2)+o3;}
#if 0
e c ei(d L0,float C2,float o3){int x=int(L0.x);int y=int(L0.y);int xc=(x^y);int b=(y>>1)&1;b|=(xc&2);b|=(y&1)<<2;b|=(xc&1)<<3;float vf=float(b);c wf=V4(vf)/16.0;return(wf*C2)+o3;}e c fi(d L0,float C2,float o3){L0.y*=0.5;L0.x=fract(L0.x*0.5+L0.y);L0.y=fract(L0.y);float P3=(L0.y*0.5+L0.x);return(P3*C2)+o3;}
#endif
#ifdef ENABLE_DITHER
e c ga(d L0,c C2,c o3){return ENABLE_DITHER?wc(L0,C2,o3):.0;}e A F2(A j,c d7,d L0,c C2,c o3){return(ENABLE_DITHER&&d7!=.0)?(wc(L0,C2,o3)+j):j;}e A F2(A j,c d7,c yc){return(ENABLE_DITHER&&d7!=.0)?(yc+j):j;}
#else
e c ga(d L0,float C2,float o3){return 0.;}e A F2(A j,c d7,d L0,c C2,c o3){return j;}e A F2(A j,c d7,c yc){return j;}
#endif
#ifdef VERTEX
e g o8(d zc,float xf,float Ac){return g(zc.x*xf-1.,zc.y*Ac-sign(Ac),0.,1.);}
#ifndef RENDER_MODE_MSAA
e g T7(f0 Z3,d G4,d ha){d ia=abs(Z3[0])+abs(Z3[1]);if(ia.x!=.0&&ia.y!=.0){d K=1./ia;d k5=R0(Z3,ha)+G4;const float yf=.5;return g(k5,-k5)*K.xyxy+K.xyxy+yf;}else{return G4.xyxy;}}
#else
e float ja(uint ka){return 1.-float(ka)*(2./32768.);}
#ifdef ENABLE_CLIP_RECT
e void Bc(f0 Z3,d G4,d ha e7){
#ifndef DISABLE_CLIP_DISTANCE_FOR_UBERSHADERS
if(any(notEqual(g(Z3),g(.0,.0,.0,.0)))){d k5=R0(Z3,ha)+G4.xy;gl_ClipDistance[0]=k5.x+1.;gl_ClipDistance[1]=k5.y+1.;gl_ClipDistance[2]=1.-k5.x;gl_ClipDistance[3]=1.-k5.y;}else{gl_ClipDistance[0]=gl_ClipDistance[1]=gl_ClipDistance[2]=gl_ClipDistance[3]=G4.x-.5;}
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
#if defined(FRAGMENT)&&defined(RENDER_MODE_MSAA)&&!defined(FIXED_FUNCTION_COLOR_OUTPUT)
e i Cc(j5 f7,int v8){if(v8==0xf){return(f7[0]+f7[1]+f7[2]+f7[3])*.25;}else{i zf=g(notEqual(v8&e6(1,2,4,8),e6(0,0,0,0)));i T=R0(f7,zf);int w8=(v8&5)+((v8>>1)&5);w8=(w8&3)+(w8>>2);T*=1./float(w8);return T;}}
#endif
