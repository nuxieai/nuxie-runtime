#define j4 3.14159265359
#define F8 6.28318530718
#define i7 1.57079632679
#ifndef RENDER_MODE_DEPTH_STENCIL
#define I4 float(.5)
#else
#define I4 float(.0)
#endif
#define I3(l) E8(l,j.eg,j.fg)
#define gg(a,l,G8) p1(a,e0(l)+e0(-1,0)) G8,p1(a,e0(l)+e0(0,0)) G8,p1(a,e0(l)+e0(0,-1)) G8,p1(a,e0(l)+e0(-1,-1)) G8
#define A5(F) j7(YC,xa,F,Vc,float(Vc),.0).x
#define Xc(F) j7(YC,xa,F,Wc,float(Wc),.0).x
#ifdef ya
f d e4(float x){return x;}f d j6(uint x){return float(x);}f d hg(Q x){return float(x);}f d za(int x){return float(x);}f i v5(e xyzw){return xyzw;}f C f8(c xy){return xy;}f i Qc(M xyzw){return vec4(xyzw);}f Q k3(d x){return uint(x);}f Q Q1(uint x){return x;}
#else
f d e4(float x){return(d) x;}f d j6(uint x){return(d) x;}f d hg(Q x){return(d) x;}f d za(int x){return(d) x;}f i v5(e xyzw){return(i) xyzw;}f C f8(c xy){return(C) xy;}f i Qc(M xyzw){return(i) xyzw;}f Q k3(d x){return(Q) x;}f Q Q1(uint x){return(Q) x;}
#endif
f d H0(d x){return x;}f C H2(C xy){return xy;}f C H2(d x,d y){C X;X.x=x,X.y=y;return X;}f C H2(d x){C X;X.x=x,X.y=x;return X;}f c Y6(float x){return c(x,x);}f v W0(d x,d y,d z){v X;X.x=x,X.y=y,X.z=z;return X;}f v W0(d x){v X;X.x=x,X.y=x,X.z=x;return X;}f i I0(d x,d y,d z,d w){i X;X.x=x,X.y=y,X.z=z,X.w=w;return X;}f i I0(v xyz,d w){i X;X.xyz=xyz;X.w=w;return X;}f i I0(d x){i X;X.x=x,X.y=x,X.z=x,X.w=x;return X;}f i I0(i x){return x;}f S4 ig(bool b){return S4(b,b);}f k7 lj(v k,v b,v P1){k7 X;X[0]=k;X[1]=b;X[2]=P1;return X;}f l7 mj(v k,v b){l7 X;X[0]=k;X[1]=b;return X;}f T4 nj(i k,i b,i P1,i jg){T4 X;X[0]=k;X[1]=b;X[2]=P1;X[3]=jg;return X;}f Y n1(e x){return Y(x.xy,x.zw);}f uint Cc(Q x){return x;}f c k6(c k,c b,float t){return(b-k)*t+k;}f d l6(uint Yc,uint U4){return Yc==0u?.0:unpackHalf2x16((Yc+kg)*U4).x;}f float Zc(c r2){r2=normalize(r2);float y1=acos(clamp(r2.x,-1.,1.));return r2.y>=.0?y1:-y1;}f i oj(i p){return I0(p.xyz*p.w,p.w);}f v Q6(i Aa){return Aa.xyz*(Aa.w!=.0?1./Aa.w:.0);}f d w3(C m7){return min(m7.x,m7.y);}f d w3(v ad){return min(w3(ad.xy),ad.z);}f d w3(i bd){C m7=min(bd.xy,bd.zw);d lg=min(m7.x,m7.y);return lg;}f d Y5(C n7){return max(n7.x,n7.y);}f d Y5(v cd){return max(Y5(cd.xy),cd.z);}f d Y5(i dd){C n7=max(dd.xy,dd.zw);d mg=max(n7.x,n7.y);return mg;}f float V9(c x){return abs(x.x)+abs(x.y);}f d Ba(d x,d Ca,d Da){
#if defined(GL_RENDERER_MALI)||defined(VULKAN_VENDOR_ARM)
#ifdef VULKAN_VENDOR_ARM
if(VULKAN_VENDOR_ARM)
#endif
{if(x<Da) if(x>Ca) return x;else return Ca;else return Da;}
#endif
return clamp(x,Ca,Da);}f d ed(c l0,d I2,d B3){d ng=fract(0.06711056*l0.x+0.00583715*l0.y);d og=fract(52.9829189*ng);return(og*I2)+B3;}
#if 0
f d pj(c l0,float I2,float B3){int x=int(l0.x);int y=int(l0.y);int fd=(x^y);int b=(y>>1)&1;b|=(fd&2);b|=(y&1)<<2;b|=(fd&1)<<3;float pg=float(b);d qg=e4(pg)/16.0;return(qg*I2)+B3;}f d qj(c l0,float I2,float B3){l0.y*=0.5;l0.x=fract(l0.x*0.5+l0.y);l0.y=fract(l0.y);float f4=(l0.y*0.5+l0.x);return(f4*I2)+B3;}
#endif
#ifdef ENABLE_DITHER
f d Ea(c l0,d I2,d B3){return ENABLE_DITHER?ed(l0,I2,B3):.0;}f v M2(v p,d o7,c l0,d I2,d B3){return(ENABLE_DITHER&&o7!=.0)?(ed(l0,I2,B3)+p):p;}f v M2(v p,d o7,d gd){return(ENABLE_DITHER&&o7!=.0)?(gd+p):p;}
#else
f d Ea(c l0,float I2,float B3){return 0.;}f v M2(v p,d o7,c l0,d I2,d B3){return p;}f v M2(v p,d o7,d gd){return p;}
#endif
#ifdef VERTEX
f e E8(c hd,float rg,float id){return e(hd.x*rg-1.,hd.y*id-sign(id),0.,1.);}
#ifndef RENDER_MODE_DEPTH_STENCIL
f e h8(Y C3,c Q3,c Fa){c Ga=abs(C3[0])+abs(C3[1]);if(Ga.x!=.0&&Ga.y!=.0){c R=1./Ga;c B5=M0(C3,Fa)+Q3;const float sg=.5;return e(B5,-B5)*R.xyxy+R.xyxy+sg;}else{return Q3.xyxy;}}
#else
f float H8(uint tg,uint ug){float jd=float((tg<<vg)|ug);
#if defined(ya)&&!defined(TARGET_SPIRV)
return jd*uintBitsToFloat(0x34000000u)+uintBitsToFloat(0xbf7fffffu);
#else
return jd*uintBitsToFloat(0x33800000u)+uintBitsToFloat(0x33000000u);
#endif
}
#ifdef ENABLE_CLIP_RECT
f void Ha(Y C3,c Q3,c Fa p7){
#ifndef DISABLE_CLIP_DISTANCE_FOR_UBERSHADERS
if(any(notEqual(e(C3),e(.0,.0,.0,.0)))){c B5=M0(C3,Fa)+Q3.xy;gl_ClipDistance[0]=B5.x+1.;gl_ClipDistance[1]=B5.y+1.;gl_ClipDistance[2]=1.-B5.x;gl_ClipDistance[3]=1.-B5.y;}else{gl_ClipDistance[0]=gl_ClipDistance[1]=gl_ClipDistance[2]=gl_ClipDistance[3]=Q3.x-.5;}
#endif
}
#endif
#endif
#endif
#if defined(FRAGMENT)&&defined(RENDER_MODE_DEPTH_STENCIL)&&!defined(FIXED_FUNCTION_COLOR_OUTPUT)
f i Ia(T4 q7,int I8){if(I8==0xf){return(q7[0]+q7[1]+q7[2]+q7[3])*.25;}else{i wg=e(notEqual(I8&m6(1,2,4,8),m6(0,0,0,0)));i X=M0(q7,wg);int J8=(I8&5)+((I8>>1)&5);J8=(J8&3)+(J8>>2);X*=1./float(J8);return X;}}
#endif
