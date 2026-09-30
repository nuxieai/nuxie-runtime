#define H3 3.14159265359
#define v8 6.28318530718
#define a7 1.57079632679
#ifndef RENDER_MODE_DEPTH_STENCIL
#define x4 float(.5)
#else
#define x4 float(.0)
#endif
#define Q3(m) r8(m,j.Nf,j.Of)
#ifdef TESS_TEXTURE_FLOATING_POINT
#define Cc(V,g,a) l5(V,g,a)
#define H4 f
#define fa(q) q
#define d6(q) q
#define ga(q) uintBitsToFloat(q)
#define m5(q) floatBitsToUint(q)
#else
#define Cc(V,g,a) I4(V,g,a)
#define H4 Y
#define fa(q) floatBitsToUint(q)
#define d6(q) uintBitsToFloat(q)
#define ga(q) q
#define m5(q) q
#endif
#define Pf(a,m,w8) v1(a,Z(m)+Z(-1,0))w8,v1(a,Z(m)+Z(0,0))w8,v1(a,Z(m)+Z(0,-1))w8,v1(a,Z(m)+Z(-1,-1))w8
#define n5(q) c7(XC,ha,q,Dc,float(Dc),.0).x
#define Fc(q) c7(XC,ha,q,Ec,float(Ec),.0).x
#ifdef Gc
e d S3(float x){return x;}e d e6(uint x){return float(x);}e d Qf(N x){return float(x);}e d ia(int x){return float(x);}e i g5(f xyzw){return xyzw;}e E U7(c xy){return xy;}e i xc(Y xyzw){return vec4(xyzw);}e N g3(d x){return uint(x);}e N a2(uint x){return x;}
#else
e d S3(float x){return(d)x;}e d e6(uint x){return(d)x;}e d Qf(N x){return(d)x;}e d ia(int x){return(d)x;}e i g5(f xyzw){return(i)xyzw;}e E U7(c xy){return(E)xy;}e i xc(Y xyzw){return(i)xyzw;}e N g3(d x){return(N)x;}e N a2(uint x){return(N)x;}
#endif
e d J0(d x){return x;}e E D2(E xy){return xy;}e E D2(d x,d y){E T;T.x=x,T.y=y;return T;}e E D2(d x){E T;T.x=x,T.y=x;return T;}e c Q6(float x){return c(x,x);}e A T0(d x,d y,d z){A T;T.x=x,T.y=y,T.z=z;return T;}e A T0(d x){A T;T.x=x,T.y=x,T.z=x;return T;}e i E0(d x,d y,d z,d w){i T;T.x=x,T.y=y,T.z=z,T.w=w;return T;}e i E0(A xyz,d w){i T;T.xyz=xyz;T.w=w;return T;}e i E0(d x){i T;T.x=x,T.y=x,T.z=x,T.w=x;return T;}e i E0(i x){return x;}e J4 Rf(bool b){return J4(b,b);}e d7 Fi(A l,A b,A J1){d7 T;T[0]=l;T[1]=b;T[2]=J1;return T;}e e7 Gi(A l,A b){e7 T;T[0]=l;T[1]=b;return T;}e K4 Hi(i l,i b,i J1,i Sf){K4 T;T[0]=l;T[1]=b;T[2]=J1;T[3]=Sf;return T;}e e0 L1(f x){return e0(x.xy,x.zw);}e uint jc(N x){return x;}e c f6(c l,c b,float t){return(b-l)*t+l;}e d x8(uint Hc,uint g6){return Hc==0u?.0:unpackHalf2x16((Hc+Tf)*g6).x;}e float Ic(c m2){m2=normalize(m2);float f1=acos(clamp(m2.x,-1.,1.));return m2.y>=.0?f1:-f1;}e i Ii(i k){return E0(k.xyz*k.w,k.w);}e A I6(i ja){return ja.xyz*(ja.w!=.0?1./ja.w:.0);}e d m3(E f7){return min(f7.x,f7.y);}e d m3(A Jc){return min(m3(Jc.xy),Jc.z);}e d m3(i Kc){E f7=min(Kc.xy,Kc.zw);d Uf=min(f7.x,f7.y);return Uf;}e d P5(E g7){return max(g7.x,g7.y);}e d P5(A Lc){return max(P5(Lc.xy),Lc.z);}e d P5(i Mc){E g7=max(Mc.xy,Mc.zw);d Vf=max(g7.x,g7.y);return Vf;}e float H9(c x){return abs(x.x)+abs(x.y);}e d ka(d x,d la,d ma){
#if defined(GL_RENDERER_MALI)||defined(VULKAN_VENDOR_ARM)
#ifdef VULKAN_VENDOR_ARM
if(VULKAN_VENDOR_ARM)
#endif
{if(x<ma)if(x>la)return x;else return la;else return ma;}
#endif
return clamp(x,la,ma);}e d Nc(c v0,d E2,d v3){d Wf=fract(0.06711056*v0.x+0.00583715*v0.y);d Xf=fract(52.9829189*Wf);return(Xf*E2)+v3;}
#if 0
e d Ji(c v0,float E2,float v3){int x=int(v0.x);int y=int(v0.y);int Oc=(x^y);int b=(y>>1)&1;b|=(Oc&2);b|=(y&1)<<2;b|=(Oc&1)<<3;float Yf=float(b);d Zf=S3(Yf)/16.0;return(Zf*E2)+v3;}e d Ki(c v0,float E2,float v3){v0.y*=0.5;v0.x=fract(v0.x*0.5+v0.y);v0.y=fract(v0.y);float T3=(v0.y*0.5+v0.x);return(T3*E2)+v3;}
#endif
#ifdef ENABLE_DITHER
e d na(c v0,d E2,d v3){return ENABLE_DITHER?Nc(v0,E2,v3):.0;}e A K2(A k,d h7,c v0,d E2,d v3){return(ENABLE_DITHER&&h7!=.0)?(Nc(v0,E2,v3)+k):k;}e A K2(A k,d h7,d Pc){return(ENABLE_DITHER&&h7!=.0)?(Pc+k):k;}
#else
e d na(c v0,float E2,float v3){return 0.;}e A K2(A k,d h7,c v0,d E2,d v3){return k;}e A K2(A k,d h7,d Pc){return k;}
#endif
#ifdef VERTEX
e f r8(c Qc,float ag,float Rc){return f(Qc.x*ag-1.,Qc.y*Rc-sign(Rc),0.,1.);}
#ifndef RENDER_MODE_DEPTH_STENCIL
e f W7(e0 e4,c L4,c oa){c pa=abs(e4[0])+abs(e4[1]);if(pa.x!=.0&&pa.y!=.0){c M=1./pa;c o5=P0(e4,oa)+L4;const float bg=.5;return f(o5,-o5)*M.xyxy+M.xyxy+bg;}else{return L4.xyxy;}}
#else
e float qa(uint ra){return 1.-float(ra)*(2./32768.);}
#ifdef ENABLE_CLIP_RECT
e void Sc(e0 e4,c L4,c oa i7){
#ifndef DISABLE_CLIP_DISTANCE_FOR_UBERSHADERS
if(any(notEqual(f(e4),f(.0,.0,.0,.0)))){c o5=P0(e4,oa)+L4.xy;gl_ClipDistance[0]=o5.x+1.;gl_ClipDistance[1]=o5.y+1.;gl_ClipDistance[2]=1.-o5.x;gl_ClipDistance[3]=1.-o5.y;}else{gl_ClipDistance[0]=gl_ClipDistance[1]=gl_ClipDistance[2]=gl_ClipDistance[3]=L4.x-.5;}
#endif
}
#endif
#endif
#endif
#ifdef FRAGMENT
#ifdef NEEDS_GAMMA_CORRECTION
e d q3(d k){return(k<=0.04045)?k/12.92:pow(abs((k+0.055)/1.055),2.4);}e A q3(A k){return T0(q3(k.x),q3(k.y),q3(k.z));}e i q3(i k){return E0(q3(k.xyz),k.w);}
#endif
#endif
#if defined(FRAGMENT)&&defined(RENDER_MODE_DEPTH_STENCIL)&&!defined(FIXED_FUNCTION_COLOR_OUTPUT)
e i sa(K4 j7,int y8){if(y8==0xf){return(j7[0]+j7[1]+j7[2]+j7[3])*.25;}else{i cg=f(notEqual(y8&h6(1,2,4,8),h6(0,0,0,0)));i T=P0(j7,cg);int z8=(y8&5)+((y8>>1)&5);z8=(z8&3)+(z8>>2);T*=1./float(z8);return T;}}
#endif
