#define w4 3.14159265359
#define B7 6.28318530718
#define C7 1.57079632679
#ifndef CB
#define S4 float(.5)
#else
#define S4 float(.0)
#endif
#define U3(o) i9(o,j.Sg,j.Tg)
#define Ug(a,o,j9) v1(a,g0(o)+g0(-1,0)) j9,v1(a,g0(o)+g0(0,0)) j9,v1(a,g0(o)+g0(0,-1)) j9,v1(a,g0(o)+g0(-1,-1)) j9
#define H5(F) D7(AD,eb,F,Id,float(Id),.0).x
#define Kd(F) D7(AD,eb,F,Jd,float(Jd),.0).x
#ifdef fb
f d T4(float x){return x;}f d x6(uint x){return float(x);}f d Vg(P x){return float(x);}f d gb(int x){return float(x);}f i Y4(e xyzw){return xyzw;}f E G8(c xy){return xy;}f i Ed(O xyzw){return vec4(xyzw);}f P W2(d x){return uint(x);}f P X1(uint x){return x;}
#else
f d T4(float x){return(d) x;}f d x6(uint x){return(d) x;}f d Vg(P x){return(d) x;}f d gb(int x){return(d) x;}f i Y4(e xyzw){return(i) xyzw;}f E G8(c xy){return(E) xy;}f i Ed(O xyzw){return(i) xyzw;}f P W2(d x){return(P) x;}f P X1(uint x){return(P) x;}
#endif
f d C0(d x){return x;}f E S2(E xy){return xy;}f E S2(d x,d y){E Y;Y.x=x,Y.y=y;return Y;}f E S2(d x){E Y;Y.x=x,Y.y=x;return Y;}f c p7(float x){return c(x,x);}f v Z0(d x,d y,d z){v Y;Y.x=x,Y.y=y,Y.z=z;return Y;}f v Z0(d x){v Y;Y.x=x,Y.y=x,Y.z=x;return Y;}f i E0(d x,d y,d z,d w){i Y;Y.x=x,Y.y=y,Y.z=z,Y.w=w;return Y;}f i E0(v xyz,d w){i Y;Y.xyz=xyz;Y.w=w;return Y;}f i E0(d x){i Y;Y.x=x,Y.y=x,Y.z=x,Y.w=x;return Y;}f i E0(i x){return x;}f g5 Wg(bool b){return g5(b,b);}f E7 fk(v l,v b,v W1){E7 Y;Y[0]=l;Y[1]=b;Y[2]=W1;return Y;}f F7 gk(v l,v b){F7 Y;Y[0]=l;Y[1]=b;return Y;}f h5 hk(i l,i b,i W1,i Xg){h5 Y;Y[0]=l;Y[1]=b;Y[2]=W1;Y[3]=Xg;return Y;}f X q1(e x){return X(x.xy,x.zw);}f uint qd(P x){return x;}f c y6(c l,c b,float t){return(b-l)*t+l;}f d k9(uint Ld,uint z6){return Ld==0u?.0:unpackHalf2x16((Ld+Yg)*z6).x;}f float hb(c B2){B2=normalize(B2);float g1=acos(clamp(B2.x,-1.,1.));return B2.y>=.0?g1:-g1;}f i ik(i k){return E0(k.xyz*k.w,k.w);}f v l6(i ib){return ib.xyz*(ib.w!=.0?1./ib.w:.0);}f d C3(E G7){return min(G7.x,G7.y);}f d C3(v Md){return min(C3(Md.xy),Md.z);}f d C3(i Nd){E G7=min(Nd.xy,Nd.zw);d Zg=min(G7.x,G7.y);return Zg;}f d k6(E H7){return max(H7.x,H7.y);}f d k6(v Od){return max(k6(Od.xy),Od.z);}f d k6(i Pd){E H7=max(Pd.xy,Pd.zw);d ah=max(H7.x,H7.y);return ah;}f float Ca(c x){return abs(x.x)+abs(x.y);}f d jb(d x,d kb,d lb){
#if defined(TF)||defined(LD)
#ifdef LD
if(LD)
#endif
{if(x<lb) if(x>kb) return x;else return kb;else return lb;}
#endif
return clamp(x,kb,lb);}f d Qd(c l0,d T2,d J3){d bh=fract(0.06711056*l0.x+0.00583715*l0.y);d ch=fract(52.9829189*bh);return(ch*T2)+J3;}
#if 0
f d jk(c l0,float T2,float J3){int x=int(l0.x);int y=int(l0.y);int Rd=(x^y);int b=(y>>1)&1;b|=(Rd&2);b|=(y&1)<<2;b|=(Rd&1)<<3;float dh=float(b);d eh=T4(dh)/16.0;return(eh*T2)+J3;}f d kk(c l0,float T2,float J3){l0.y*=0.5;l0.x=fract(l0.x*0.5+l0.y);l0.y=fract(l0.y);float l4=(l0.y*0.5+l0.x);return(l4*T2)+J3;}
#endif
#ifdef OB
f d mb(c l0,d T2,d J3){return OB?Qd(l0,T2,J3):.0;}f v J2(v k,d I7,c l0,d T2,d J3){return(OB&&I7!=.0)?(Qd(l0,T2,J3)+k):k;}f v J2(v k,d I7,d Sd){return(OB&&I7!=.0)?(Sd+k):k;}
#else
f d mb(c l0,float T2,float J3){return 0.;}f v J2(v k,d I7,c l0,d T2,d J3){return k;}f v J2(v k,d I7,d Sd){return k;}
#endif
#ifdef BB
f e i9(c Td,float fh,float Ud){return e(Td.x*fh-1.,Td.y*Ud-sign(Ud),0.,1.);}
#ifndef CB
f e I8(X K3,c a4,c nb){c ob=abs(K3[0])+abs(K3[1]);if(ob.x!=.0&&ob.y!=.0){c S=1./ob;c I5=D0(K3,nb)+a4;const float gh=.5;return e(I5,-I5)*S.xyxy+S.xyxy+gh;}else{return a4.xyxy;}}
#else
f float l9(uint hh,uint ih){float Vd=float((hh<<jh)|ih);
#if defined(fb)&&!defined(EC)
return Vd*uintBitsToFloat(0x34000000u)+uintBitsToFloat(0xbf7fffffu);
#else
return Vd*uintBitsToFloat(0x33800000u)+uintBitsToFloat(0x33000000u);
#endif
}
#ifdef AB
f void pb(X K3,c a4,c nb A6){
#ifndef XE
if(any(notEqual(e(K3),e(.0,.0,.0,.0)))){c I5=D0(K3,nb)+a4.xy;gl_ClipDistance[0]=I5.x+1.;gl_ClipDistance[1]=I5.y+1.;gl_ClipDistance[2]=1.-I5.x;gl_ClipDistance[3]=1.-I5.y;}else{gl_ClipDistance[0]=gl_ClipDistance[1]=gl_ClipDistance[2]=gl_ClipDistance[3]=a4.x-.5;}
#endif
}
#endif
#endif
#endif
#if defined(DB)&&defined(CB)&&!defined(R)
f i qb(h5 J7,int m9){if(m9==int(0xf)){return(J7[0]+J7[1]+J7[2]+J7[3])*.25;}else{i kh=e(notEqual(m9&B6(1,2,4,8),B6(0,0,0,0)));i Y=D0(J7,kh);int n9=(m9&5)+((m9>>1)&5);n9=(n9&3)+(n9>>2);Y*=1./float(n9);return Y;}}
#endif
#ifdef DB
f i Pa(i Wd,i z1){return Wd+z1*(1.-Wd.w);}
#endif
