#define z4 3.14159265359
#define A7 6.28318530718
#define B7 1.57079632679
#ifndef RENDER_MODE_DEPTH_STENCIL
#define Y4 float(.5)
#else
#define Y4 float(.0)
#endif
#define U3(o) l9(o,j.ah,j.bh)
#define ch(a,o,m9) q1(a,f0(o)+f0(-1,0)) m9,q1(a,f0(o)+f0(0,0)) m9,q1(a,f0(o)+f0(0,-1)) m9,q1(a,f0(o)+f0(-1,-1)) m9
#define M5(G) C7(ZC,ib,G,Nd,float(Nd),.0).x
#define Pd(G) C7(ZC,ib,G,Od,float(Od),.0).x
#ifdef jb
f d Z4(float x){return x;}f d x6(uint x){return float(x);}f d dh(O x){return float(x);}f d kb(int x){return float(x);}f i y4(e xyzw){return xyzw;}f F J8(c xy){return xy;}f i Jd(R xyzw){return vec4(xyzw);}f O Z2(d x){return uint(x);}f O Z1(uint x){return x;}
#else
f d Z4(float x){return(d) x;}f d x6(uint x){return(d) x;}f d dh(O x){return(d) x;}f d kb(int x){return(d) x;}f i y4(e xyzw){return(i) xyzw;}f F J8(c xy){return(F) xy;}f i Jd(R xyzw){return(i) xyzw;}f O Z2(d x){return(O) x;}f O Z1(uint x){return(O) x;}
#endif
f d A0(d x){return x;}f F U2(F xy){return xy;}f F U2(d x,d y){F Z;Z.x=x,Z.y=y;return Z;}f F U2(d x){F Z;Z.x=x,Z.y=x;return Z;}f c o7(float x){return c(x,x);}f v d1(d x,d y,d z){v Z;Z.x=x,Z.y=y,Z.z=z;return Z;}f v d1(d x){v Z;Z.x=x,Z.y=x,Z.z=x;return Z;}f i z0(d x,d y,d z,d w){i Z;Z.x=x,Z.y=y,Z.z=z,Z.w=w;return Z;}f i z0(v xyz,d w){i Z;Z.xyz=xyz;Z.w=w;return Z;}f i z0(d x){i Z;Z.x=x,Z.y=x,Z.z=x,Z.w=x;return Z;}f i z0(i x){return x;}f l5 eh(bool b){return l5(b,b);}f D7 pk(v l,v b,v Y1){D7 Z;Z[0]=l;Z[1]=b;Z[2]=Y1;return Z;}f E7 qk(v l,v b){E7 Z;Z[0]=l;Z[1]=b;return Z;}f m5 rk(i l,i b,i Y1,i fh){m5 Z;Z[0]=l;Z[1]=b;Z[2]=Y1;Z[3]=fh;return Z;}f X v1(e x){return X(x.xy,x.zw);}f uint vd(O x){return x;}f c y6(c l,c b,float t){return(b-l)*t+l;}f d n9(uint Qd,uint z6){return Qd==0u?.0:unpackHalf2x16((Qd+gh)*z6).x;}f float lb(c F2){F2=normalize(F2);float h1=acos(clamp(F2.x,-1.,1.));return F2.y>=.0?h1:-h1;}f i sk(i k){return z0(k.xyz*k.w,k.w);}f v m6(i mb){return mb.xyz*(mb.w!=.0?1./mb.w:.0);}f d F3(F F7){return min(F7.x,F7.y);}f d F3(v Rd){return min(F3(Rd.xy),Rd.z);}f d F3(i Sd){F F7=min(Sd.xy,Sd.zw);d hh=min(F7.x,F7.y);return hh;}f d l6(F G7){return max(G7.x,G7.y);}f d l6(v Td){return max(l6(Td.xy),Td.z);}f d l6(i Ud){F G7=max(Ud.xy,Ud.zw);d ih=max(G7.x,G7.y);return ih;}f float Fa(c x){return abs(x.x)+abs(x.y);}f d nb(d x,d ob,d pb){
#if defined(GL_RENDERER_MALI)||defined(VULKAN_VENDOR_ARM)
#ifdef VULKAN_VENDOR_ARM
if(VULKAN_VENDOR_ARM)
#endif
{if(x<pb) if(x>ob) return x;else return ob;else return pb;}
#endif
return clamp(x,ob,pb);}f d Vd(c m0,d V2,d M3){d jh=fract(0.06711056*m0.x+0.00583715*m0.y);d kh=fract(52.9829189*jh);return(kh*V2)+M3;}
#if 0
f d tk(c m0,float V2,float M3){int x=int(m0.x);int y=int(m0.y);int Wd=(x^y);int b=(y>>1)&1;b|=(Wd&2);b|=(y&1)<<2;b|=(Wd&1)<<3;float lh=float(b);d mh=Z4(lh)/16.0;return(mh*V2)+M3;}f d uk(c m0,float V2,float M3){m0.y*=0.5;m0.x=fract(m0.x*0.5+m0.y);m0.y=fract(m0.y);float m4=(m0.y*0.5+m0.x);return(m4*V2)+M3;}
#endif
#ifdef ENABLE_DITHER
f d qb(c m0,d V2,d M3){return ENABLE_DITHER?Vd(m0,V2,M3):.0;}f v M2(v k,d H7,c m0,d V2,d M3){return(ENABLE_DITHER&&H7!=.0)?(Vd(m0,V2,M3)+k):k;}f v M2(v k,d H7,d Xd){return(ENABLE_DITHER&&H7!=.0)?(Xd+k):k;}
#else
f d qb(c m0,float V2,float M3){return 0.;}f v M2(v k,d H7,c m0,d V2,d M3){return k;}f v M2(v k,d H7,d Xd){return k;}
#endif
#ifdef VERTEX
f e l9(c Yd,float nh,float Zd){return e(Yd.x*nh-1.,Yd.y*Zd-sign(Zd),0.,1.);}
#ifndef RENDER_MODE_DEPTH_STENCIL
f e L8(X N3,c a4,c rb){c sb=abs(N3[0])+abs(N3[1]);if(sb.x!=.0&&sb.y!=.0){c U=1./sb;c N5=F0(N3,rb)+a4;const float oh=.5;return e(N5,-N5)*U.xyxy+U.xyxy+oh;}else{return a4.xyxy;}}
#else
f float o9(uint ph,uint qh){float ae=float((ph<<rh)|qh);
#if defined(jb)&&!defined(TARGET_SPIRV)
return ae*uintBitsToFloat(0x34000000u)+uintBitsToFloat(0xbf7fffffu);
#else
return ae*uintBitsToFloat(0x33800000u)+uintBitsToFloat(0x33000000u);
#endif
}
#ifdef ENABLE_CLIP_RECT
f void tb(X N3,c a4,c rb A6){
#ifndef DISABLE_CLIP_DISTANCE_FOR_UBERSHADERS
if(any(notEqual(e(N3),e(.0,.0,.0,.0)))){c N5=F0(N3,rb)+a4.xy;gl_ClipDistance[0]=N5.x+1.;gl_ClipDistance[1]=N5.y+1.;gl_ClipDistance[2]=1.-N5.x;gl_ClipDistance[3]=1.-N5.y;}else{gl_ClipDistance[0]=gl_ClipDistance[1]=gl_ClipDistance[2]=gl_ClipDistance[3]=a4.x-.5;}
#endif
}
#endif
#endif
#endif
#if defined(FRAGMENT)&&defined(RENDER_MODE_DEPTH_STENCIL)&&!defined(FIXED_FUNCTION_COLOR_OUTPUT)
f i ub(m5 I7,int p9){if(p9==int(0xf)){return(I7[0]+I7[1]+I7[2]+I7[3])*.25;}else{i sh=y4(e(notEqual(p9&B6(1,2,4,8),B6(0,0,0,0))));i Z=F0(I7,sh);int q9=(p9&5)+((p9>>1)&5);q9=(q9&3)+(q9>>2);Z*=A0(1./float(q9));return Z;}}
#endif
#ifdef FRAGMENT
f i Ta(i be,i A1){return be+A1*(1.-be.w);}
#endif
