#define i4 3.14159265359
#define F8 6.28318530718
#define i7 1.57079632679
#ifndef RENDER_MODE_DEPTH_STENCIL
#define H4 float(.5)
#else
#define H4 float(.0)
#endif
#define I3(m) E8(m,j.Yf,j.Zf)
#define ag(a,m,G8) p1(a,e0(m)+e0(-1,0)) G8,p1(a,e0(m)+e0(0,0)) G8,p1(a,e0(m)+e0(0,-1)) G8,p1(a,e0(m)+e0(-1,-1)) G8
#define y5(F) j7(ZC,wa,F,Uc,float(Uc),.0).x
#define Wc(F) j7(ZC,wa,F,Vc,float(Vc),.0).x
#ifdef xa
f d d4(float x){return x;}f d i6(uint x){return float(x);}f d bg(R x){return float(x);}f d ya(int x){return float(x);}f i q5(e xyzw){return xyzw;}f C f8(c xy){return xy;}f i Pc(N xyzw){return vec4(xyzw);}f R k3(d x){return uint(x);}f R O1(uint x){return x;}
#else
f d d4(float x){return(d) x;}f d i6(uint x){return(d) x;}f d bg(R x){return(d) x;}f d ya(int x){return(d) x;}f i q5(e xyzw){return(i) xyzw;}f C f8(c xy){return(C) xy;}f i Pc(N xyzw){return(i) xyzw;}f R k3(d x){return(R) x;}f R O1(uint x){return(R) x;}
#endif
f d M0(d x){return x;}f C I2(C xy){return xy;}f C I2(d x,d y){C X;X.x=x,X.y=y;return X;}f C I2(d x){C X;X.x=x,X.y=x;return X;}f c X6(float x){return c(x,x);}f v W0(d x,d y,d z){v X;X.x=x,X.y=y,X.z=z;return X;}f v W0(d x){v X;X.x=x,X.y=x,X.z=x;return X;}f i G0(d x,d y,d z,d w){i X;X.x=x,X.y=y,X.z=z,X.w=w;return X;}f i G0(v xyz,d w){i X;X.xyz=xyz;X.w=w;return X;}f i G0(d x){i X;X.x=x,X.y=x,X.z=x,X.w=x;return X;}f i G0(i x){return x;}f R4 cg(bool b){return R4(b,b);}f k7 Vi(v k,v b,v N1){k7 X;X[0]=k;X[1]=b;X[2]=N1;return X;}f l7 Wi(v k,v b){l7 X;X[0]=k;X[1]=b;return X;}f S4 Xi(i k,i b,i N1,i dg){S4 X;X[0]=k;X[1]=b;X[2]=N1;X[3]=dg;return X;}f Y n1(e x){return Y(x.xy,x.zw);}f uint Bc(R x){return x;}f c j6(c k,c b,float t){return(b-k)*t+k;}f d k6(uint Xc,uint T4){return Xc==0u?.0:unpackHalf2x16((Xc+eg)*T4).x;}f float Yc(c v2){v2=normalize(v2);float w1=acos(clamp(v2.x,-1.,1.));return v2.y>=.0?w1:-w1;}f i Yi(i l){return G0(l.xyz*l.w,l.w);}f v P6(i za){return za.xyz*(za.w!=.0?1./za.w:.0);}f d v3(C m7){return min(m7.x,m7.y);}f d v3(v Zc){return min(v3(Zc.xy),Zc.z);}f d v3(i ad){C m7=min(ad.xy,ad.zw);d fg=min(m7.x,m7.y);return fg;}f d W5(C n7){return max(n7.x,n7.y);}f d W5(v bd){return max(W5(bd.xy),bd.z);}f d W5(i cd){C n7=max(cd.xy,cd.zw);d gg=max(n7.x,n7.y);return gg;}f float U9(c x){return abs(x.x)+abs(x.y);}f d Aa(d x,d Ba,d Ca){
#if defined(GL_RENDERER_MALI)||defined(VULKAN_VENDOR_ARM)
#ifdef VULKAN_VENDOR_ARM
if(VULKAN_VENDOR_ARM)
#endif
{if(x<Ca) if(x>Ba) return x;else return Ba;else return Ca;}
#endif
return clamp(x,Ba,Ca);}f d dd(c l0,d J2,d B3){d hg=fract(0.06711056*l0.x+0.00583715*l0.y);d ig=fract(52.9829189*hg);return(ig*J2)+B3;}
#if 0
f d Zi(c l0,float J2,float B3){int x=int(l0.x);int y=int(l0.y);int ed=(x^y);int b=(y>>1)&1;b|=(ed&2);b|=(y&1)<<2;b|=(ed&1)<<3;float jg=float(b);d kg=d4(jg)/16.0;return(kg*J2)+B3;}f d aj(c l0,float J2,float B3){l0.y*=0.5;l0.x=fract(l0.x*0.5+l0.y);l0.y=fract(l0.y);float e4=(l0.y*0.5+l0.x);return(e4*J2)+B3;}
#endif
#ifdef ENABLE_DITHER
f d Da(c l0,d J2,d B3){return ENABLE_DITHER?dd(l0,J2,B3):.0;}f v O2(v l,d o7,c l0,d J2,d B3){return(ENABLE_DITHER&&o7!=.0)?(dd(l0,J2,B3)+l):l;}f v O2(v l,d o7,d fd){return(ENABLE_DITHER&&o7!=.0)?(fd+l):l;}
#else
f d Da(c l0,float J2,float B3){return 0.;}f v O2(v l,d o7,c l0,d J2,d B3){return l;}f v O2(v l,d o7,d fd){return l;}
#endif
#ifdef VERTEX
f e E8(c gd,float lg,float hd){return e(gd.x*lg-1.,gd.y*hd-sign(hd),0.,1.);}
#ifndef RENDER_MODE_DEPTH_STENCIL
f e h8(Y C3,c Q3,c Ea){c Fa=abs(C3[0])+abs(C3[1]);if(Fa.x!=.0&&Fa.y!=.0){c Q=1./Fa;c z5=K0(C3,Ea)+Q3;const float mg=.5;return e(z5,-z5)*Q.xyxy+Q.xyxy+mg;}else{return Q3.xyxy;}}
#else
f float H8(uint ng,uint og){float id=float((ng<<pg)|og);
#if defined(xa)&&!defined(TARGET_SPIRV)
return id*uintBitsToFloat(0x34000000u)+uintBitsToFloat(0xbf7fffffu);
#else
return id*uintBitsToFloat(0x33800000u)+uintBitsToFloat(0x33000000u);
#endif
}
#ifdef ENABLE_CLIP_RECT
f void Ga(Y C3,c Q3,c Ea p7){
#ifndef DISABLE_CLIP_DISTANCE_FOR_UBERSHADERS
if(any(notEqual(e(C3),e(.0,.0,.0,.0)))){c z5=K0(C3,Ea)+Q3.xy;gl_ClipDistance[0]=z5.x+1.;gl_ClipDistance[1]=z5.y+1.;gl_ClipDistance[2]=1.-z5.x;gl_ClipDistance[3]=1.-z5.y;}else{gl_ClipDistance[0]=gl_ClipDistance[1]=gl_ClipDistance[2]=gl_ClipDistance[3]=Q3.x-.5;}
#endif
}
#endif
#endif
#endif
#ifdef FRAGMENT
#ifdef NEEDS_GAMMA_CORRECTION
f d z3(d l){return(l<=0.04045)?l/12.92:pow(abs((l+0.055)/1.055),2.4);}f v z3(v l){return W0(z3(l.x),z3(l.y),z3(l.z));}f i z3(i l){return G0(z3(l.xyz),l.w);}
#endif
#endif
#if defined(FRAGMENT)&&defined(RENDER_MODE_DEPTH_STENCIL)&&!defined(FIXED_FUNCTION_COLOR_OUTPUT)
f i Ha(S4 q7,int I8){if(I8==0xf){return(q7[0]+q7[1]+q7[2]+q7[3])*.25;}else{i qg=e(notEqual(I8&l6(1,2,4,8),l6(0,0,0,0)));i X=K0(q7,qg);int J8=(I8&5)+((I8>>1)&5);J8=(J8&3)+(J8>>2);X*=1./float(J8);return X;}}
#endif
