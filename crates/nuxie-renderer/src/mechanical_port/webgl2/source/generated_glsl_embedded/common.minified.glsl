#define r4 3.14159265359
#define B7 6.28318530718
#define C7 1.57079632679
#ifndef CB
#define Q4 float(.5)
#else
#define Q4 float(.0)
#endif
#define T3(o) h9(o,j.Mg,j.Ng)
#define Og(a,o,i9) v1(a,g0(o)+g0(-1,0)) i9,v1(a,g0(o)+g0(0,0)) i9,v1(a,g0(o)+g0(0,-1)) i9,v1(a,g0(o)+g0(-1,-1)) i9
#define H5(F) D7(AD,db,F,Fd,float(Fd),.0).x
#define Hd(F) D7(AD,db,F,Gd,float(Gd),.0).x
#ifdef eb
f d x5(float x){return x;}f d x6(uint x){return float(x);}f d Pg(P x){return float(x);}f d fb(int x){return float(x);}f i X4(e xyzw){return xyzw;}f E F8(c xy){return xy;}f i Bd(O xyzw){return vec4(xyzw);}f P W2(d x){return uint(x);}f P X1(uint x){return x;}
#else
f d x5(float x){return(d) x;}f d x6(uint x){return(d) x;}f d Pg(P x){return(d) x;}f d fb(int x){return(d) x;}f i X4(e xyzw){return(i) xyzw;}f E F8(c xy){return(E) xy;}f i Bd(O xyzw){return(i) xyzw;}f P W2(d x){return(P) x;}f P X1(uint x){return(P) x;}
#endif
f d J0(d x){return x;}f E S2(E xy){return xy;}f E S2(d x,d y){E Y;Y.x=x,Y.y=y;return Y;}f E S2(d x){E Y;Y.x=x,Y.y=x;return Y;}f c p7(float x){return c(x,x);}f v Z0(d x,d y,d z){v Y;Y.x=x,Y.y=y,Y.z=z;return Y;}f v Z0(d x){v Y;Y.x=x,Y.y=x,Y.z=x;return Y;}f i H0(d x,d y,d z,d w){i Y;Y.x=x,Y.y=y,Y.z=z,Y.w=w;return Y;}f i H0(v xyz,d w){i Y;Y.xyz=xyz;Y.w=w;return Y;}f i H0(d x){i Y;Y.x=x,Y.y=x,Y.z=x,Y.w=x;return Y;}f i H0(i x){return x;}f e5 Qg(bool b){return e5(b,b);}f E7 ek(v l,v b,v W1){E7 Y;Y[0]=l;Y[1]=b;Y[2]=W1;return Y;}f F7 fk(v l,v b){F7 Y;Y[0]=l;Y[1]=b;return Y;}f f5 gk(i l,i b,i W1,i Rg){f5 Y;Y[0]=l;Y[1]=b;Y[2]=W1;Y[3]=Rg;return Y;}f X q1(e x){return X(x.xy,x.zw);}f uint nd(P x){return x;}f c y6(c l,c b,float t){return(b-l)*t+l;}f d j9(uint Id,uint z6){return Id==0u?.0:unpackHalf2x16((Id+Sg)*z6).x;}f float gb(c B2){B2=normalize(B2);float g1=acos(clamp(B2.x,-1.,1.));return B2.y>=.0?g1:-g1;}f i hk(i k){return H0(k.xyz*k.w,k.w);}f v l6(i hb){return hb.xyz*(hb.w!=.0?1./hb.w:.0);}f d C3(E G7){return min(G7.x,G7.y);}f d C3(v Jd){return min(C3(Jd.xy),Jd.z);}f d C3(i Kd){E G7=min(Kd.xy,Kd.zw);d Tg=min(G7.x,G7.y);return Tg;}f d k6(E H7){return max(H7.x,H7.y);}f d k6(v Ld){return max(k6(Ld.xy),Ld.z);}f d k6(i Md){E H7=max(Md.xy,Md.zw);d Ug=max(H7.x,H7.y);return Ug;}f float Ba(c x){return abs(x.x)+abs(x.y);}f d ib(d x,d jb,d kb){
#if defined(TF)||defined(LD)
#ifdef LD
if(LD)
#endif
{if(x<kb) if(x>jb) return x;else return jb;else return kb;}
#endif
return clamp(x,jb,kb);}f d Nd(c l0,d T2,d I3){d Vg=fract(0.06711056*l0.x+0.00583715*l0.y);d Wg=fract(52.9829189*Vg);return(Wg*T2)+I3;}
#if 0
f d ik(c l0,float T2,float I3){int x=int(l0.x);int y=int(l0.y);int Od=(x^y);int b=(y>>1)&1;b|=(Od&2);b|=(y&1)<<2;b|=(Od&1)<<3;float Xg=float(b);d Yg=x5(Xg)/16.0;return(Yg*T2)+I3;}f d jk(c l0,float T2,float I3){l0.y*=0.5;l0.x=fract(l0.x*0.5+l0.y);l0.y=fract(l0.y);float l4=(l0.y*0.5+l0.x);return(l4*T2)+I3;}
#endif
#ifdef OB
f d lb(c l0,d T2,d I3){return OB?Nd(l0,T2,I3):.0;}f v J2(v k,d I7,c l0,d T2,d I3){return(OB&&I7!=.0)?(Nd(l0,T2,I3)+k):k;}f v J2(v k,d I7,d Pd){return(OB&&I7!=.0)?(Pd+k):k;}
#else
f d lb(c l0,float T2,float I3){return 0.;}f v J2(v k,d I7,c l0,d T2,d I3){return k;}f v J2(v k,d I7,d Pd){return k;}
#endif
#ifdef BB
f e h9(c Qd,float Zg,float Rd){return e(Qd.x*Zg-1.,Qd.y*Rd-sign(Rd),0.,1.);}
#ifndef CB
f e H8(X J3,c Z3,c mb){c nb=abs(J3[0])+abs(J3[1]);if(nb.x!=.0&&nb.y!=.0){c S=1./nb;c I5=C0(J3,mb)+Z3;const float ah=.5;return e(I5,-I5)*S.xyxy+S.xyxy+ah;}else{return Z3.xyxy;}}
#else
f float k9(uint bh,uint ch){float Sd=float((bh<<dh)|ch);
#if defined(eb)&&!defined(EC)
return Sd*uintBitsToFloat(0x34000000u)+uintBitsToFloat(0xbf7fffffu);
#else
return Sd*uintBitsToFloat(0x33800000u)+uintBitsToFloat(0x33000000u);
#endif
}
#ifdef AB
f void ob(X J3,c Z3,c mb A6){
#ifndef XE
if(any(notEqual(e(J3),e(.0,.0,.0,.0)))){c I5=C0(J3,mb)+Z3.xy;gl_ClipDistance[0]=I5.x+1.;gl_ClipDistance[1]=I5.y+1.;gl_ClipDistance[2]=1.-I5.x;gl_ClipDistance[3]=1.-I5.y;}else{gl_ClipDistance[0]=gl_ClipDistance[1]=gl_ClipDistance[2]=gl_ClipDistance[3]=Z3.x-.5;}
#endif
}
#endif
#endif
#endif
#if defined(EB)&&defined(CB)&&!defined(R)
f i pb(f5 J7,int l9){if(l9==int(0xf)){return(J7[0]+J7[1]+J7[2]+J7[3])*.25;}else{i eh=e(notEqual(l9&B6(1,2,4,8),B6(0,0,0,0)));i Y=C0(J7,eh);int m9=(l9&5)+((l9>>1)&5);m9=(m9&3)+(m9>>2);Y*=1./float(m9);return Y;}}
#endif
#ifdef EB
f i Oa(i Td,i z1){return Td+z1*(1.-Td.w);}
#endif
