#define j4 3.14159265359
#define F8 6.28318530718
#define i7 1.57079632679
#ifndef CB
#define I4 float(.5)
#else
#define I4 float(.0)
#endif
#define I3(l) E8(l,j.dg,j.eg)
#define fg(a,l,G8) p1(a,e0(l)+e0(-1,0)) G8,p1(a,e0(l)+e0(0,0)) G8,p1(a,e0(l)+e0(0,-1)) G8,p1(a,e0(l)+e0(-1,-1)) G8
#define A5(F) j7(YC,xa,F,Vc,float(Vc),.0).x
#define Xc(F) j7(YC,xa,F,Wc,float(Wc),.0).x
#ifdef ya
f d e4(float x){return x;}f d j6(uint x){return float(x);}f d gg(Q x){return float(x);}f d za(int x){return float(x);}f i v5(e xyzw){return xyzw;}f C f8(c xy){return xy;}f i Qc(M xyzw){return vec4(xyzw);}f Q k3(d x){return uint(x);}f Q Q1(uint x){return x;}
#else
f d e4(float x){return(d) x;}f d j6(uint x){return(d) x;}f d gg(Q x){return(d) x;}f d za(int x){return(d) x;}f i v5(e xyzw){return(i) xyzw;}f C f8(c xy){return(C) xy;}f i Qc(M xyzw){return(i) xyzw;}f Q k3(d x){return(Q) x;}f Q Q1(uint x){return(Q) x;}
#endif
f d H0(d x){return x;}f C H2(C xy){return xy;}f C H2(d x,d y){C X;X.x=x,X.y=y;return X;}f C H2(d x){C X;X.x=x,X.y=x;return X;}f c Y6(float x){return c(x,x);}f v W0(d x,d y,d z){v X;X.x=x,X.y=y,X.z=z;return X;}f v W0(d x){v X;X.x=x,X.y=x,X.z=x;return X;}f i I0(d x,d y,d z,d w){i X;X.x=x,X.y=y,X.z=z,X.w=w;return X;}f i I0(v xyz,d w){i X;X.xyz=xyz;X.w=w;return X;}f i I0(d x){i X;X.x=x,X.y=x,X.z=x,X.w=x;return X;}f i I0(i x){return x;}f S4 hg(bool b){return S4(b,b);}f k7 kj(v k,v b,v P1){k7 X;X[0]=k;X[1]=b;X[2]=P1;return X;}f l7 lj(v k,v b){l7 X;X[0]=k;X[1]=b;return X;}f T4 mj(i k,i b,i P1,i ig){T4 X;X[0]=k;X[1]=b;X[2]=P1;X[3]=ig;return X;}f Y n1(e x){return Y(x.xy,x.zw);}f uint Cc(Q x){return x;}f c k6(c k,c b,float t){return(b-k)*t+k;}f d l6(uint Yc,uint U4){return Yc==0u?.0:unpackHalf2x16((Yc+jg)*U4).x;}f float Zc(c r2){r2=normalize(r2);float y1=acos(clamp(r2.x,-1.,1.));return r2.y>=.0?y1:-y1;}f i nj(i p){return I0(p.xyz*p.w,p.w);}f v Q6(i Aa){return Aa.xyz*(Aa.w!=.0?1./Aa.w:.0);}f d w3(C m7){return min(m7.x,m7.y);}f d w3(v ad){return min(w3(ad.xy),ad.z);}f d w3(i bd){C m7=min(bd.xy,bd.zw);d kg=min(m7.x,m7.y);return kg;}f d Y5(C n7){return max(n7.x,n7.y);}f d Y5(v cd){return max(Y5(cd.xy),cd.z);}f d Y5(i dd){C n7=max(dd.xy,dd.zw);d lg=max(n7.x,n7.y);return lg;}f float V9(c x){return abs(x.x)+abs(x.y);}f d Ba(d x,d Ca,d Da){
#if defined(OF)||defined(HD)
#ifdef HD
if(HD)
#endif
{if(x<Da) if(x>Ca) return x;else return Ca;else return Da;}
#endif
return clamp(x,Ca,Da);}f d ed(c l0,d I2,d B3){d mg=fract(0.06711056*l0.x+0.00583715*l0.y);d ng=fract(52.9829189*mg);return(ng*I2)+B3;}
#if 0
f d oj(c l0,float I2,float B3){int x=int(l0.x);int y=int(l0.y);int fd=(x^y);int b=(y>>1)&1;b|=(fd&2);b|=(y&1)<<2;b|=(fd&1)<<3;float og=float(b);d pg=e4(og)/16.0;return(pg*I2)+B3;}f d pj(c l0,float I2,float B3){l0.y*=0.5;l0.x=fract(l0.x*0.5+l0.y);l0.y=fract(l0.y);float f4=(l0.y*0.5+l0.x);return(f4*I2)+B3;}
#endif
#ifdef OB
f d Ea(c l0,d I2,d B3){return OB?ed(l0,I2,B3):.0;}f v M2(v p,d o7,c l0,d I2,d B3){return(OB&&o7!=.0)?(ed(l0,I2,B3)+p):p;}f v M2(v p,d o7,d gd){return(OB&&o7!=.0)?(gd+p):p;}
#else
f d Ea(c l0,float I2,float B3){return 0.;}f v M2(v p,d o7,c l0,d I2,d B3){return p;}f v M2(v p,d o7,d gd){return p;}
#endif
#ifdef BB
f e E8(c hd,float qg,float id){return e(hd.x*qg-1.,hd.y*id-sign(id),0.,1.);}
#ifndef CB
f e h8(Y C3,c Q3,c Fa){c Ga=abs(C3[0])+abs(C3[1]);if(Ga.x!=.0&&Ga.y!=.0){c R=1./Ga;c B5=M0(C3,Fa)+Q3;const float rg=.5;return e(B5,-B5)*R.xyxy+R.xyxy+rg;}else{return Q3.xyxy;}}
#else
f float H8(uint sg,uint tg){float jd=float((sg<<ug)|tg);
#if defined(ya)&&!defined(DC)
return jd*uintBitsToFloat(0x34000000u)+uintBitsToFloat(0xbf7fffffu);
#else
return jd*uintBitsToFloat(0x33800000u)+uintBitsToFloat(0x33000000u);
#endif
}
#ifdef AB
f void Ha(Y C3,c Q3,c Fa p7){
#ifndef QE
if(any(notEqual(e(C3),e(.0,.0,.0,.0)))){c B5=M0(C3,Fa)+Q3.xy;gl_ClipDistance[0]=B5.x+1.;gl_ClipDistance[1]=B5.y+1.;gl_ClipDistance[2]=1.-B5.x;gl_ClipDistance[3]=1.-B5.y;}else{gl_ClipDistance[0]=gl_ClipDistance[1]=gl_ClipDistance[2]=gl_ClipDistance[3]=Q3.x-.5;}
#endif
}
#endif
#endif
#endif
#if defined(FB)&&defined(CB)&&!defined(W)
f i Ia(T4 q7,int I8){if(I8==0xf){return(q7[0]+q7[1]+q7[2]+q7[3])*.25;}else{i vg=e(notEqual(I8&m6(1,2,4,8),m6(0,0,0,0)));i X=M0(q7,vg);int J8=(I8&5)+((I8>>1)&5);J8=(J8&3)+(J8>>2);X*=1./float(J8);return X;}}
#endif
