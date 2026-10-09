#define p4 3.14159265359
#define e9 6.28318530718
#define x7 1.57079632679
#ifndef CB
#define O4 float(.5)
#else
#define O4 float(.0)
#endif
#define R3(o) d9(o,j.Hg,j.Ig)
#define Jg(a,o,f9) q1(a,g0(o)+g0(-1,0)) f9,q1(a,g0(o)+g0(0,0)) f9,q1(a,g0(o)+g0(0,-1)) f9,q1(a,g0(o)+g0(-1,-1)) f9
#define F5(E) y7(YC,ab,E,zd,float(zd),.0).x
#define Bd(E) y7(YC,ab,E,Ad,float(Ad),.0).x
#ifdef bb
e d v5(float x){return x;}e d r6(uint x){return float(x);}e d Kg(P x){return float(x);}e d cb(int x){return float(x);}e i V4(f xyzw){return xyzw;}e D B8(c xy){return xy;}e i vd(O xyzw){return vec4(xyzw);}e P W2(d x){return uint(x);}e P S1(uint x){return x;}
#else
e d v5(float x){return(d) x;}e d r6(uint x){return(d) x;}e d Kg(P x){return(d) x;}e d cb(int x){return(d) x;}e i V4(f xyzw){return(i) xyzw;}e D B8(c xy){return(D) xy;}e i vd(O xyzw){return(i) xyzw;}e P W2(d x){return(P) x;}e P S1(uint x){return(P) x;}
#endif
e d J0(d x){return x;}e D Q2(D xy){return xy;}e D Q2(d x,d y){D Y;Y.x=x,Y.y=y;return Y;}e D Q2(d x){D Y;Y.x=x,Y.y=x;return Y;}e c l7(float x){return c(x,x);}e v Z0(d x,d y,d z){v Y;Y.x=x,Y.y=y,Y.z=z;return Y;}e v Z0(d x){v Y;Y.x=x,Y.y=x,Y.z=x;return Y;}e i H0(d x,d y,d z,d w){i Y;Y.x=x,Y.y=y,Y.z=z,Y.w=w;return Y;}e i H0(v xyz,d w){i Y;Y.xyz=xyz;Y.w=w;return Y;}e i H0(d x){i Y;Y.x=x,Y.y=x,Y.z=x,Y.w=x;return Y;}e i H0(i x){return x;}e c5 Lg(bool b){return c5(b,b);}e z7 Uj(v m,v b,v R1){z7 Y;Y[0]=m;Y[1]=b;Y[2]=R1;return Y;}e A7 Vj(v m,v b){A7 Y;Y[0]=m;Y[1]=b;return Y;}e d5 Wj(i m,i b,i R1,i Mg){d5 Y;Y[0]=m;Y[1]=b;Y[2]=R1;Y[3]=Mg;return Y;}e X o1(f x){return X(x.xy,x.zw);}e uint id(P x){return x;}e c v6(c m,c b,float t){return(b-m)*t+m;}e d g9(uint Cd,uint w6){return Cd==0u?.0:unpackHalf2x16((Cd+Ng)*w6).x;}e float Dd(c A2){A2=normalize(A2);float f1=acos(clamp(A2.x,-1.,1.));return A2.y>=.0?f1:-f1;}e i Xj(i l){return H0(l.xyz*l.w,l.w);}e v i6(i db){return db.xyz*(db.w!=.0?1./db.w:.0);}e d B3(D B7){return min(B7.x,B7.y);}e d B3(v Ed){return min(B3(Ed.xy),Ed.z);}e d B3(i Fd){D B7=min(Fd.xy,Fd.zw);d Og=min(B7.x,B7.y);return Og;}e d h6(D C7){return max(C7.x,C7.y);}e d h6(v Gd){return max(h6(Gd.xy),Gd.z);}e d h6(i Hd){D C7=max(Hd.xy,Hd.zw);d Pg=max(C7.x,C7.y);return Pg;}e float za(c x){return abs(x.x)+abs(x.y);}e d eb(d x,d fb,d gb){
#if defined(QF)||defined(JD)
#ifdef JD
if(JD)
#endif
{if(x<gb) if(x>fb) return x;else return fb;else return gb;}
#endif
return clamp(x,fb,gb);}e d Id(c l0,d R2,d H3){d Qg=fract(0.06711056*l0.x+0.00583715*l0.y);d Rg=fract(52.9829189*Qg);return(Rg*R2)+H3;}
#if 0
e d Yj(c l0,float R2,float H3){int x=int(l0.x);int y=int(l0.y);int Jd=(x^y);int b=(y>>1)&1;b|=(Jd&2);b|=(y&1)<<2;b|=(Jd&1)<<3;float Sg=float(b);d Tg=v5(Sg)/16.0;return(Tg*R2)+H3;}e d Zj(c l0,float R2,float H3){l0.y*=0.5;l0.x=fract(l0.x*0.5+l0.y);l0.y=fract(l0.y);float j4=(l0.y*0.5+l0.x);return(j4*R2)+H3;}
#endif
#ifdef OB
e d hb(c l0,d R2,d H3){return OB?Id(l0,R2,H3):.0;}e v I2(v l,d D7,c l0,d R2,d H3){return(OB&&D7!=.0)?(Id(l0,R2,H3)+l):l;}e v I2(v l,d D7,d Kd){return(OB&&D7!=.0)?(Kd+l):l;}
#else
e d hb(c l0,float R2,float H3){return 0.;}e v I2(v l,d D7,c l0,d R2,d H3){return l;}e v I2(v l,d D7,d Kd){return l;}
#endif
#ifdef BB
e f d9(c Ld,float Ug,float Md){return f(Ld.x*Ug-1.,Ld.y*Md-sign(Md),0.,1.);}
#ifndef CB
e f D8(X I3,c X3,c ib){c jb=abs(I3[0])+abs(I3[1]);if(jb.x!=.0&&jb.y!=.0){c R=1./jb;c G5=B0(I3,ib)+X3;const float Vg=.5;return f(G5,-G5)*R.xyxy+R.xyxy+Vg;}else{return X3.xyxy;}}
#else
e float h9(uint Wg,uint Xg){float Nd=float((Wg<<Yg)|Xg);
#if defined(bb)&&!defined(DC)
return Nd*uintBitsToFloat(0x34000000u)+uintBitsToFloat(0xbf7fffffu);
#else
return Nd*uintBitsToFloat(0x33800000u)+uintBitsToFloat(0x33000000u);
#endif
}
#ifdef AB
e void kb(X I3,c X3,c ib E7){
#ifndef SE
if(any(notEqual(f(I3),f(.0,.0,.0,.0)))){c G5=B0(I3,ib)+X3.xy;gl_ClipDistance[0]=G5.x+1.;gl_ClipDistance[1]=G5.y+1.;gl_ClipDistance[2]=1.-G5.x;gl_ClipDistance[3]=1.-G5.y;}else{gl_ClipDistance[0]=gl_ClipDistance[1]=gl_ClipDistance[2]=gl_ClipDistance[3]=X3.x-.5;}
#endif
}
#endif
#endif
#endif
#if defined(EB)&&defined(CB)&&!defined(U)
e i lb(d5 F7,int i9){if(i9==0xf){return(F7[0]+F7[1]+F7[2]+F7[3])*.25;}else{i Zg=f(notEqual(i9&x6(1,2,4,8),x6(0,0,0,0)));i Y=B0(F7,Zg);int j9=(i9&5)+((i9>>1)&5);j9=(j9&3)+(j9>>2);Y*=1./float(j9);return Y;}}
#endif
