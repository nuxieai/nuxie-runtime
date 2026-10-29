#define w4 3.14159265359
#define y7 6.28318530718
#define z7 1.57079632679
#ifndef RENDER_MODE_DEPTH_STENCIL
#define S4 float(.5)
#else
#define S4 float(.0)
#endif
#define U3(o) g9(o,j.Rg,j.Sg)
#define Tg(a,o,h9) v1(a,f0(o)+f0(-1,0)) h9,v1(a,f0(o)+f0(0,0)) h9,v1(a,f0(o)+f0(0,-1)) h9,v1(a,f0(o)+f0(-1,-1)) h9
#define F5(F) A7(ZC,fb,F,Jd,float(Jd),.0).x
#define Ld(F) A7(ZC,fb,F,Kd,float(Kd),.0).x
#ifdef gb
f d T4(float x){return x;}f d r6(uint x){return float(x);}f d Ug(O x){return float(x);}f d hb(int x){return float(x);}f i Y4(e xyzw){return xyzw;}f E E8(c xy){return xy;}f i Fd(R xyzw){return vec4(xyzw);}f O Y2(d x){return uint(x);}f O X1(uint x){return x;}
#else
f d T4(float x){return(d) x;}f d r6(uint x){return(d) x;}f d Ug(O x){return(d) x;}f d hb(int x){return(d) x;}f i Y4(e xyzw){return(i) xyzw;}f E E8(c xy){return(E) xy;}f i Fd(R xyzw){return(i) xyzw;}f O Y2(d x){return(O) x;}f O X1(uint x){return(O) x;}
#endif
f d C0(d x){return x;}f E U2(E xy){return xy;}f E U2(d x,d y){E Y;Y.x=x,Y.y=y;return Y;}f E U2(d x){E Y;Y.x=x,Y.y=x;return Y;}f c m7(float x){return c(x,x);}f v Y0(d x,d y,d z){v Y;Y.x=x,Y.y=y,Y.z=z;return Y;}f v Y0(d x){v Y;Y.x=x,Y.y=x,Y.z=x;return Y;}f i E0(d x,d y,d z,d w){i Y;Y.x=x,Y.y=y,Y.z=z,Y.w=w;return Y;}f i E0(v xyz,d w){i Y;Y.xyz=xyz;Y.w=w;return Y;}f i E0(d x){i Y;Y.x=x,Y.y=x,Y.z=x,Y.w=x;return Y;}f i E0(i x){return x;}f g5 Vg(bool b){return g5(b,b);}f B7 fk(v l,v b,v W1){B7 Y;Y[0]=l;Y[1]=b;Y[2]=W1;return Y;}f C7 gk(v l,v b){C7 Y;Y[0]=l;Y[1]=b;return Y;}f h5 hk(i l,i b,i W1,i Wg){h5 Y;Y[0]=l;Y[1]=b;Y[2]=W1;Y[3]=Wg;return Y;}f X p1(e x){return X(x.xy,x.zw);}f uint rd(O x){return x;}f c v6(c l,c b,float t){return(b-l)*t+l;}f d i9(uint Md,uint w6){return Md==0u?.0:unpackHalf2x16((Md+Xg)*w6).x;}f float ib(c D2){D2=normalize(D2);float f1=acos(clamp(D2.x,-1.,1.));return D2.y>=.0?f1:-f1;}f i ik(i k){return E0(k.xyz*k.w,k.w);}f v j6(i jb){return jb.xyz*(jb.w!=.0?1./jb.w:.0);}f d D3(E D7){return min(D7.x,D7.y);}f d D3(v Nd){return min(D3(Nd.xy),Nd.z);}f d D3(i Od){E D7=min(Od.xy,Od.zw);d Yg=min(D7.x,D7.y);return Yg;}f d i6(E E7){return max(E7.x,E7.y);}f d i6(v Pd){return max(i6(Pd.xy),Pd.z);}f d i6(i Qd){E E7=max(Qd.xy,Qd.zw);d Zg=max(E7.x,E7.y);return Zg;}f float Ca(c x){return abs(x.x)+abs(x.y);}f d kb(d x,d lb,d mb){
#if defined(GL_RENDERER_MALI)||defined(VULKAN_VENDOR_ARM)
#ifdef VULKAN_VENDOR_ARM
if(VULKAN_VENDOR_ARM)
#endif
{if(x<mb) if(x>lb) return x;else return lb;else return mb;}
#endif
return clamp(x,lb,mb);}f d Rd(c l0,d V2,d K3){d ah=fract(0.06711056*l0.x+0.00583715*l0.y);d bh=fract(52.9829189*ah);return(bh*V2)+K3;}
#if 0
f d jk(c l0,float V2,float K3){int x=int(l0.x);int y=int(l0.y);int Sd=(x^y);int b=(y>>1)&1;b|=(Sd&2);b|=(y&1)<<2;b|=(Sd&1)<<3;float ch=float(b);d dh=T4(ch)/16.0;return(dh*V2)+K3;}f d kk(c l0,float V2,float K3){l0.y*=0.5;l0.x=fract(l0.x*0.5+l0.y);l0.y=fract(l0.y);float k4=(l0.y*0.5+l0.x);return(k4*V2)+K3;}
#endif
#ifdef ENABLE_DITHER
f d nb(c l0,d V2,d K3){return ENABLE_DITHER?Rd(l0,V2,K3):.0;}f v K2(v k,d F7,c l0,d V2,d K3){return(ENABLE_DITHER&&F7!=.0)?(Rd(l0,V2,K3)+k):k;}f v K2(v k,d F7,d Td){return(ENABLE_DITHER&&F7!=.0)?(Td+k):k;}
#else
f d nb(c l0,float V2,float K3){return 0.;}f v K2(v k,d F7,c l0,d V2,d K3){return k;}f v K2(v k,d F7,d Td){return k;}
#endif
#ifdef VERTEX
f e g9(c Ud,float eh,float Vd){return e(Ud.x*eh-1.,Ud.y*Vd-sign(Vd),0.,1.);}
#ifndef RENDER_MODE_DEPTH_STENCIL
f e G8(X L3,c a4,c ob){c pb=abs(L3[0])+abs(L3[1]);if(pb.x!=.0&&pb.y!=.0){c U=1./pb;c G5=D0(L3,ob)+a4;const float fh=.5;return e(G5,-G5)*U.xyxy+U.xyxy+fh;}else{return a4.xyxy;}}
#else
f float j9(uint gh,uint hh){float Wd=float((gh<<ih)|hh);
#if defined(gb)&&!defined(TARGET_SPIRV)
return Wd*uintBitsToFloat(0x34000000u)+uintBitsToFloat(0xbf7fffffu);
#else
return Wd*uintBitsToFloat(0x33800000u)+uintBitsToFloat(0x33000000u);
#endif
}
#ifdef ENABLE_CLIP_RECT
f void qb(X L3,c a4,c ob x6){
#ifndef DISABLE_CLIP_DISTANCE_FOR_UBERSHADERS
if(any(notEqual(e(L3),e(.0,.0,.0,.0)))){c G5=D0(L3,ob)+a4.xy;gl_ClipDistance[0]=G5.x+1.;gl_ClipDistance[1]=G5.y+1.;gl_ClipDistance[2]=1.-G5.x;gl_ClipDistance[3]=1.-G5.y;}else{gl_ClipDistance[0]=gl_ClipDistance[1]=gl_ClipDistance[2]=gl_ClipDistance[3]=a4.x-.5;}
#endif
}
#endif
#endif
#endif
#if defined(FRAGMENT)&&defined(RENDER_MODE_DEPTH_STENCIL)&&!defined(FIXED_FUNCTION_COLOR_OUTPUT)
f i rb(h5 G7,int k9){if(k9==int(0xf)){return(G7[0]+G7[1]+G7[2]+G7[3])*.25;}else{i jh=e(notEqual(k9&y6(1,2,4,8),y6(0,0,0,0)));i Y=D0(G7,jh);int l9=(k9&5)+((k9>>1)&5);l9=(l9&3)+(l9>>2);Y*=1./float(l9);return Y;}}
#endif
#ifdef FRAGMENT
f i Qa(i Xd,i z1){return Xd+z1*(1.-Xd.w);}
#endif
