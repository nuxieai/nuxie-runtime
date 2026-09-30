#define j4 3.14159265359
#define H8 6.28318530718
#define j7 1.57079632679
#ifndef RENDER_MODE_DEPTH_STENCIL
#define I4 float(.5)
#else
#define I4 float(.0)
#endif
#define I3(m) G8(m,j.cg,j.dg)
#define eg(a,m,I8) p1(a,e0(m)+e0(-1,0)) I8,p1(a,e0(m)+e0(0,0)) I8,p1(a,e0(m)+e0(0,-1)) I8,p1(a,e0(m)+e0(-1,-1)) I8
#define B5(F) k7(ZC,xa,F,Vc,float(Vc),.0).x
#define Xc(F) k7(ZC,xa,F,Wc,float(Wc),.0).x
#ifdef ya
e d e4(float x){return x;}e d k6(uint x){return float(x);}e d fg(R x){return float(x);}e d za(int x){return float(x);}e i w5(f xyzw){return xyzw;}e C h8(c xy){return xy;}e i Qc(N xyzw){return vec4(xyzw);}e R k3(d x){return uint(x);}e R P1(uint x){return x;}
#else
e d e4(float x){return(d) x;}e d k6(uint x){return(d) x;}e d fg(R x){return(d) x;}e d za(int x){return(d) x;}e i w5(f xyzw){return(i) xyzw;}e C h8(c xy){return(C) xy;}e i Qc(N xyzw){return(i) xyzw;}e R k3(d x){return(R) x;}e R P1(uint x){return(R) x;}
#endif
e d I0(d x){return x;}e C I2(C xy){return xy;}e C I2(d x,d y){C X;X.x=x,X.y=y;return X;}e C I2(d x){C X;X.x=x,X.y=x;return X;}e c Z6(float x){return c(x,x);}e v R0(d x,d y,d z){v X;X.x=x,X.y=y,X.z=z;return X;}e v R0(d x){v X;X.x=x,X.y=x,X.z=x;return X;}e i G0(d x,d y,d z,d w){i X;X.x=x,X.y=y,X.z=z,X.w=w;return X;}e i G0(v xyz,d w){i X;X.xyz=xyz;X.w=w;return X;}e i G0(d x){i X;X.x=x,X.y=x,X.z=x,X.w=x;return X;}e i G0(i x){return x;}e S4 gg(bool b){return S4(b,b);}e l7 ij(v k,v b,v O1){l7 X;X[0]=k;X[1]=b;X[2]=O1;return X;}e m7 jj(v k,v b){m7 X;X[0]=k;X[1]=b;return X;}e T4 kj(i k,i b,i O1,i hg){T4 X;X[0]=k;X[1]=b;X[2]=O1;X[3]=hg;return X;}e Y n1(f x){return Y(x.xy,x.zw);}e uint Cc(R x){return x;}e c l6(c k,c b,float t){return(b-k)*t+k;}e d m6(uint Yc,uint U4){return Yc==0u?.0:unpackHalf2x16((Yc+ig)*U4).x;}e float Zc(c v2){v2=normalize(v2);float x1=acos(clamp(v2.x,-1.,1.));return v2.y>=.0?x1:-x1;}e i lj(i l){return G0(l.xyz*l.w,l.w);}e v R6(i Aa){return Aa.xyz*(Aa.w!=.0?1./Aa.w:.0);}e d v3(C n7){return min(n7.x,n7.y);}e d v3(v ad){return min(v3(ad.xy),ad.z);}e d v3(i bd){C n7=min(bd.xy,bd.zw);d jg=min(n7.x,n7.y);return jg;}e d Z5(C o7){return max(o7.x,o7.y);}e d Z5(v cd){return max(Z5(cd.xy),cd.z);}e d Z5(i dd){C o7=max(dd.xy,dd.zw);d kg=max(o7.x,o7.y);return kg;}e float V9(c x){return abs(x.x)+abs(x.y);}e d Ba(d x,d Ca,d Da){
#if defined(GL_RENDERER_MALI)||defined(VULKAN_VENDOR_ARM)
#ifdef VULKAN_VENDOR_ARM
if(VULKAN_VENDOR_ARM)
#endif
{if(x<Da) if(x>Ca) return x;else return Ca;else return Da;}
#endif
return clamp(x,Ca,Da);}e d ed(c l0,d J2,d B3){d lg=fract(0.06711056*l0.x+0.00583715*l0.y);d mg=fract(52.9829189*lg);return(mg*J2)+B3;}
#if 0
e d mj(c l0,float J2,float B3){int x=int(l0.x);int y=int(l0.y);int fd=(x^y);int b=(y>>1)&1;b|=(fd&2);b|=(y&1)<<2;b|=(fd&1)<<3;float ng=float(b);d og=e4(ng)/16.0;return(og*J2)+B3;}e d nj(c l0,float J2,float B3){l0.y*=0.5;l0.x=fract(l0.x*0.5+l0.y);l0.y=fract(l0.y);float f4=(l0.y*0.5+l0.x);return(f4*J2)+B3;}
#endif
#ifdef ENABLE_DITHER
e d Ea(c l0,d J2,d B3){return ENABLE_DITHER?ed(l0,J2,B3):.0;}e v O2(v l,d p7,c l0,d J2,d B3){return(ENABLE_DITHER&&p7!=.0)?(ed(l0,J2,B3)+l):l;}e v O2(v l,d p7,d gd){return(ENABLE_DITHER&&p7!=.0)?(gd+l):l;}
#else
e d Ea(c l0,float J2,float B3){return 0.;}e v O2(v l,d p7,c l0,d J2,d B3){return l;}e v O2(v l,d p7,d gd){return l;}
#endif
#ifdef VERTEX
e f G8(c hd,float pg,float id){return f(hd.x*pg-1.,hd.y*id-sign(id),0.,1.);}
#ifndef RENDER_MODE_DEPTH_STENCIL
e f j8(Y C3,c Q3,c Fa){c Ga=abs(C3[0])+abs(C3[1]);if(Ga.x!=.0&&Ga.y!=.0){c Q=1./Ga;c C5=M0(C3,Fa)+Q3;const float qg=.5;return f(C5,-C5)*Q.xyxy+Q.xyxy+qg;}else{return Q3.xyxy;}}
#else
e float J8(uint rg,uint sg){float jd=float((rg<<tg)|sg);
#if defined(ya)&&!defined(TARGET_SPIRV)
return jd*uintBitsToFloat(0x34000000u)+uintBitsToFloat(0xbf7fffffu);
#else
return jd*uintBitsToFloat(0x33800000u)+uintBitsToFloat(0x33000000u);
#endif
}
#ifdef ENABLE_CLIP_RECT
e void Ha(Y C3,c Q3,c Fa q7){
#ifndef DISABLE_CLIP_DISTANCE_FOR_UBERSHADERS
if(any(notEqual(f(C3),f(.0,.0,.0,.0)))){c C5=M0(C3,Fa)+Q3.xy;gl_ClipDistance[0]=C5.x+1.;gl_ClipDistance[1]=C5.y+1.;gl_ClipDistance[2]=1.-C5.x;gl_ClipDistance[3]=1.-C5.y;}else{gl_ClipDistance[0]=gl_ClipDistance[1]=gl_ClipDistance[2]=gl_ClipDistance[3]=Q3.x-.5;}
#endif
}
#endif
#endif
#endif
#ifdef FRAGMENT
#ifdef NEEDS_GAMMA_CORRECTION
e d z3(d l){return(l<=0.04045)?l/12.92:pow(abs((l+0.055)/1.055),2.4);}e v z3(v l){return R0(z3(l.x),z3(l.y),z3(l.z));}e i z3(i l){return G0(z3(l.xyz),l.w);}
#endif
#endif
#if defined(FRAGMENT)&&defined(RENDER_MODE_DEPTH_STENCIL)&&!defined(FIXED_FUNCTION_COLOR_OUTPUT)
e i Ia(T4 r7,int K8){if(K8==0xf){return(r7[0]+r7[1]+r7[2]+r7[3])*.25;}else{i ug=f(notEqual(K8&n6(1,2,4,8),n6(0,0,0,0)));i X=M0(r7,ug);int L8=(K8&5)+((K8>>1)&5);L8=(L8&3)+(L8>>2);X*=1./float(L8);return X;}}
#endif
