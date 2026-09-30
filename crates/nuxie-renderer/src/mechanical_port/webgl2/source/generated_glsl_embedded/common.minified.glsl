#define X3 3.14159265359
#define p8 6.28318530718
#define Y6 1.57079632679
#ifndef BB
#define y4 float(.5)
#else
#define y4 float(.0)
#endif
#define P3(m) o8(m,j.Hf,j.If)
#define Jf(a,m,q8) F1(a,d0(m)+d0(-1,0)) q8,F1(a,d0(m)+d0(0,0)) q8,F1(a,d0(m)+d0(0,-1)) q8,F1(a,d0(m)+d0(-1,-1)) q8
#define m5(E) Z6(YC,ca,E,xc,float(xc),.0).x
#define zc(E) Z6(YC,ca,E,yc,float(yc),.0).x
#ifdef da
e d R3(float x){return x;}e d Z5(uint x){return float(x);}e d Kf(N x){return float(x);}e d ea(int x){return float(x);}e i h5(f xyzw){return xyzw;}e D Q7(c xy){return xy;}e i sc(R xyzw){return vec4(xyzw);}e N g3(d x){return uint(x);}e N a2(uint x){return x;}
#else
e d R3(float x){return(d) x;}e d Z5(uint x){return(d) x;}e d Kf(N x){return(d) x;}e d ea(int x){return(d) x;}e i h5(f xyzw){return(i) xyzw;}e D Q7(c xy){return(D) xy;}e i sc(R xyzw){return(i) xyzw;}e N g3(d x){return(N) x;}e N a2(uint x){return(N) x;}
#endif
e d J0(d x){return x;}e D E2(D xy){return xy;}e D E2(d x,d y){D U;U.x=x,U.y=y;return U;}e D E2(d x){D U;U.x=x,U.y=x;return U;}e c O6(float x){return c(x,x);}e v T0(d x,d y,d z){v U;U.x=x,U.y=y,U.z=z;return U;}e v T0(d x){v U;U.x=x,U.y=x,U.z=x;return U;}e i E0(d x,d y,d z,d w){i U;U.x=x,U.y=y,U.z=z,U.w=w;return U;}e i E0(v xyz,d w){i U;U.xyz=xyz;U.w=w;return U;}e i E0(d x){i U;U.x=x,U.y=x,U.z=x,U.w=x;return U;}e i E0(i x){return x;}e I4 Lf(bool b){return I4(b,b);}e a7 Fi(v l,v b,v J1){a7 U;U[0]=l;U[1]=b;U[2]=J1;return U;}e c7 Gi(v l,v b){c7 U;U[0]=l;U[1]=b;return U;}e J4 Hi(i l,i b,i J1,i Mf){J4 U;U[0]=l;U[1]=b;U[2]=J1;U[3]=Mf;return U;}e Y L1(f x){return Y(x.xy,x.zw);}e uint ec(N x){return x;}e c a6(c l,c b,float t){return(b-l)*t+l;}e d r8(uint Ac,uint c6){return Ac==0u?.0:unpackHalf2x16((Ac+Nf)*c6).x;}e float Bc(c n2){n2=normalize(n2);float m1=acos(clamp(n2.x,-1.,1.));return n2.y>=.0?m1:-m1;}e i Ii(i k){return E0(k.xyz*k.w,k.w);}e v G6(i fa){return fa.xyz*(fa.w!=.0?1./fa.w:.0);}e d m3(D d7){return min(d7.x,d7.y);}e d m3(v Cc){return min(m3(Cc.xy),Cc.z);}e d m3(i Dc){D d7=min(Dc.xy,Dc.zw);d Of=min(d7.x,d7.y);return Of;}e d N5(D e7){return max(e7.x,e7.y);}e d N5(v Ec){return max(N5(Ec.xy),Ec.z);}e d N5(i Fc){D e7=max(Fc.xy,Fc.zw);d Pf=max(e7.x,e7.y);return Pf;}e float E9(c x){return abs(x.x)+abs(x.y);}e d ga(d x,d ha,d ia){
#if defined(OF)||defined(HD)
#ifdef HD
if(HD)
#endif
{if(x<ia) if(x>ha) return x;else return ha;else return ia;}
#endif
return clamp(x,ha,ia);}e d Gc(c v0,d F2,d v3){d Qf=fract(0.06711056*v0.x+0.00583715*v0.y);d Rf=fract(52.9829189*Qf);return(Rf*F2)+v3;}
#if 0
e d Ji(c v0,float F2,float v3){int x=int(v0.x);int y=int(v0.y);int Hc=(x^y);int b=(y>>1)&1;b|=(Hc&2);b|=(y&1)<<2;b|=(Hc&1)<<3;float Sf=float(b);d Tf=R3(Sf)/16.0;return(Tf*F2)+v3;}e d Ki(c v0,float F2,float v3){v0.y*=0.5;v0.x=fract(v0.x*0.5+v0.y);v0.y=fract(v0.y);float S3=(v0.y*0.5+v0.x);return(S3*F2)+v3;}
#endif
#ifdef LB
e d ja(c v0,d F2,d v3){return LB?Gc(v0,F2,v3):.0;}e v L2(v k,d f7,c v0,d F2,d v3){return(LB&&f7!=.0)?(Gc(v0,F2,v3)+k):k;}e v L2(v k,d f7,d Ic){return(LB&&f7!=.0)?(Ic+k):k;}
#else
e d ja(c v0,float F2,float v3){return 0.;}e v L2(v k,d f7,c v0,d F2,d v3){return k;}e v L2(v k,d f7,d Ic){return k;}
#endif
#ifdef CB
e f o8(c Jc,float Uf,float Kc){return f(Jc.x*Uf-1.,Jc.y*Kc-sign(Kc),0.,1.);}
#ifndef BB
e f S7(Y e4,c K4,c ka){c la=abs(e4[0])+abs(e4[1]);if(la.x!=.0&&la.y!=.0){c M=1./la;c n5=N0(e4,ka)+K4;const float Vf=.5;return f(n5,-n5)*M.xyxy+M.xyxy+Vf;}else{return K4.xyxy;}}
#else
e float ma(uint Wf,uint Xf){float Lc=float((Wf<<Yf)|Xf);
#if defined(da)&&!defined(SB)
return Lc*uintBitsToFloat(0x34000000u)+uintBitsToFloat(0xbf7fffffu);
#else
return Lc*uintBitsToFloat(0x33800000u)+uintBitsToFloat(0x33000000u);
#endif
}
#ifdef AB
e void Mc(Y e4,c K4,c ka g7){
#ifndef SE
if(any(notEqual(f(e4),f(.0,.0,.0,.0)))){c n5=N0(e4,ka)+K4.xy;gl_ClipDistance[0]=n5.x+1.;gl_ClipDistance[1]=n5.y+1.;gl_ClipDistance[2]=1.-n5.x;gl_ClipDistance[3]=1.-n5.y;}else{gl_ClipDistance[0]=gl_ClipDistance[1]=gl_ClipDistance[2]=gl_ClipDistance[3]=K4.x-.5;}
#endif
}
#endif
#endif
#endif
#ifdef EB
#ifdef BC
e d q3(d k){return(k<=0.04045)?k/12.92:pow(abs((k+0.055)/1.055),2.4);}e v q3(v k){return T0(q3(k.x),q3(k.y),q3(k.z));}e i q3(i k){return E0(q3(k.xyz),k.w);}
#endif
#endif
#if defined(EB)&&defined(BB)&&!defined(Q)
e i na(J4 h7,int v8){if(v8==0xf){return(h7[0]+h7[1]+h7[2]+h7[3])*.25;}else{i Zf=f(notEqual(v8&d6(1,2,4,8),d6(0,0,0,0)));i U=N0(h7,Zf);int w8=(v8&5)+((v8>>1)&5);w8=(w8&3)+(w8>>2);U*=1./float(w8);return U;}}
#endif
