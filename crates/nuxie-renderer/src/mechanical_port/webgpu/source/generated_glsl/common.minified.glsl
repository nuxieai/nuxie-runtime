#define B3 3.14159265359
#define m8 6.28318530718
#define T6 1.57079632679
#ifndef RENDER_MODE_MSAA
#define n4 float(.5)
#else
#define n4 float(.0)
#endif
#define K3(l) l8(l,n.ff,n.gf)
#ifdef TESS_TEXTURE_FLOATING_POINT
#define ic(T,f,a) e5(T,f,a)
#define B4 g
#define Y9(q) q
#define V5(q) q
#define Z9(q) uintBitsToFloat(q)
#define f5(q) floatBitsToUint(q)
#else
#define ic(T,f,a) C4(T,f,a)
#define B4 G
#define Y9(q) floatBitsToUint(q)
#define V5(q) uintBitsToFloat(q)
#define Z9(q) q
#define f5(q) q
#endif
#define hf(a,l,n8) q1(a,X(l)+X(-1,0))n8,q1(a,X(l)+X(0,0))n8,q1(a,X(l)+X(0,-1))n8,q1(a,X(l)+X(-1,-1))n8
#define g5(q) U6(XC,aa,q,jc,float(jc),.0).x
#define lc(q) U6(XC,aa,q,kc,float(kc),.0).x
#ifdef mc
e c S4(float x){return x;}e c W5(uint x){return float(x);}e c jf(N x){return float(x);}e c ba(int x){return float(x);}e i Y4(g xyzw){return xyzw;}e E O7(d xy){return xy;}e i dc(G xyzw){return vec4(xyzw);}e N X5(c x){return uint(x);}e N W1(uint x){return x;}
#else
e c S4(float x){return(c)x;}e c W5(uint x){return(c)x;}e c jf(N x){return(c)x;}e c ba(int x){return(c)x;}e i Y4(g xyzw){return(i)xyzw;}e E O7(d xy){return(E)xy;}e i dc(G xyzw){return(i)xyzw;}e N X5(c x){return(N)x;}e N W1(uint x){return(N)x;}
#endif
e c G0(c x){return x;}e E A2(E xy){return xy;}e E A2(c x,c y){E S;S.x=x,S.y=y;return S;}e E A2(c x){E S;S.x=x,S.y=x;return S;}e d J6(float x){return d(x,x);}e v Q0(c x,c y,c z){v S;S.x=x,S.y=y,S.z=z;return S;}e v Q0(c x){v S;S.x=x,S.y=x,S.z=x;return S;}e i C0(c x,c y,c z,c w){i S;S.x=x,S.y=y,S.z=z,S.w=w;return S;}e i C0(v xyz,c w){i S;S.xyz=xyz;S.w=w;return S;}e i C0(c x){i S;S.x=x,S.y=x,S.z=x,S.w=x;return S;}e i C0(i x){return x;}e D4 kf(bool b){return D4(b,b);}e V6 Ph(v m,v b,v G1){V6 S;S[0]=m;S[1]=b;S[2]=G1;return S;}e W6 Qh(v m,v b){W6 S;S[0]=m;S[1]=b;return S;}e h5 Rh(i m,i b,i G1,i lf){h5 S;S[0]=m;S[1]=b;S[2]=G1;S[3]=lf;return S;}e g0 l2(g x){return g0(x.xy,x.zw);}e uint Qb(N x){return x;}e d Y5(d m,d b,float t){return(b-m)*t+m;}e c o8(uint nc,uint Z5){return nc==0u?.0:unpackHalf2x16((nc+mf)*Z5).x;}e float oc(d h2){h2=normalize(h2);float e1=acos(clamp(h2.x,-1.,1.));return h2.y>=.0?e1:-e1;}e i Sh(i j){return C0(j.xyz*j.w,j.w);}e v C6(i ca){return ca.xyz*(ca.w!=.0?1./ca.w:.0);}e c f3(E X6){return min(X6.x,X6.y);}e c f3(v pc){return min(f3(pc.xy),pc.z);}e c f3(i qc){E X6=min(qc.xy,qc.zw);c nf=min(X6.x,X6.y);return nf;}e c J5(E Y6){return max(Y6.x,Y6.y);}e c J5(v rc){return max(J5(rc.xy),rc.z);}e c J5(i sc){E Y6=max(sc.xy,sc.zw);c of=max(Y6.x,Y6.y);return of;}e float E9(d x){return abs(x.x)+abs(x.y);}e c da(c x,c ea,c fa){
#if defined(GL_RENDERER_MALI)||defined(VULKAN_VENDOR_ARM)
#ifdef VULKAN_VENDOR_ARM
if(VULKAN_VENDOR_ARM)
#endif
{if(x<fa)if(x>ea)return x;else return ea;else return fa;}
#endif
return clamp(x,ea,fa);}e c tc(d K0,c B2,c m3){c pf=fract(0.06711056*K0.x+0.00583715*K0.y);c qf=fract(52.9829189*pf);return(qf*B2)+m3;}
#if 0
e c Th(d K0,float B2,float m3){int x=int(K0.x);int y=int(K0.y);int uc=(x^y);int b=(y>>1)&1;b|=(uc&2);b|=(y&1)<<2;b|=(uc&1)<<3;float rf=float(b);c sf=S4(rf)/16.0;return(sf*B2)+m3;}e c Uh(d K0,float B2,float m3){K0.y*=0.5;K0.x=fract(K0.x*0.5+K0.y);K0.y=fract(K0.y);float N3=(K0.y*0.5+K0.x);return(N3*B2)+m3;}
#endif
#ifdef ENABLE_DITHER
e c ga(d K0,c B2,c m3){return ENABLE_DITHER?tc(K0,B2,m3):.0;}e v E2(v j,c Z6,d K0,c B2,c m3){return(ENABLE_DITHER&&Z6!=.0)?(tc(K0,B2,m3)+j):j;}e v E2(v j,c Z6,c vc){return(ENABLE_DITHER&&Z6!=.0)?(vc+j):j;}
#else
e c ga(d K0,float B2,float m3){return 0.;}e v E2(v j,c Z6,d K0,c B2,c m3){return j;}e v E2(v j,c Z6,c vc){return j;}
#endif
#ifdef VERTEX
e g l8(d wc,float tf,float xc){return g(wc.x*tf-1.,wc.y*xc-sign(xc),0.,1.);}
#ifndef RENDER_MODE_MSAA
e g Q7(g0 X3,d E4,d ha){d ia=abs(X3[0])+abs(X3[1]);if(ia.x!=.0&&ia.y!=.0){d K=1./ia;d i5=U0(X3,ha)+E4;const float uf=.5;return g(i5,-i5)*K.xyxy+K.xyxy+uf;}else{return E4.xyxy;}}
#else
e float ja(uint ka){return 1.-float(ka)*(2./32768.);}
#ifdef ENABLE_CLIP_RECT
e void yc(g0 X3,d E4,d ha a7){
#ifndef DISABLE_CLIP_DISTANCE_FOR_UBERSHADERS
if(any(notEqual(g(X3),g(.0,.0,.0,.0)))){d i5=U0(X3,ha)+E4.xy;gl_ClipDistance[0]=i5.x+1.;gl_ClipDistance[1]=i5.y+1.;gl_ClipDistance[2]=1.-i5.x;gl_ClipDistance[3]=1.-i5.y;}else{gl_ClipDistance[0]=gl_ClipDistance[1]=gl_ClipDistance[2]=gl_ClipDistance[3]=E4.x-.5;}
#endif
}
#endif
#endif
#endif
#ifdef FRAGMENT
#ifdef NEEDS_GAMMA_CORRECTION
e c k3(c j){return(j<=0.04045)?j/12.92:pow(abs((j+0.055)/1.055),2.4);}e v k3(v j){return Q0(k3(j.x),k3(j.y),k3(j.z));}e i k3(i j){return C0(k3(j.xyz),j.w);}
#endif
#endif
#if defined(FRAGMENT)&&defined(RENDER_MODE_MSAA)&&!defined(FIXED_FUNCTION_COLOR_OUTPUT)
e i zc(h5 c7,int p8){if(p8==0xf){return(c7[0]+c7[1]+c7[2]+c7[3])*.25;}else{i vf=g(notEqual(p8&a6(1,2,4,8),a6(0,0,0,0)));i S=U0(c7,vf);int q8=(p8&5)+((p8>>1)&5);q8=(q8&3)+(q8>>2);S*=1./float(q8);return S;}}
#endif
