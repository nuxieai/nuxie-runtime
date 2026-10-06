#pragma once

#include "draw_path_common.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_path_common[] = R"===(#define w7 -2.
#define Dd -1.5
#define Ed .25
#define Q8 1e3
#define Fd (Q8*Q8)
#ifdef BB
k4 W4(l3,Qg,TB);
#ifdef HB
q6(l3,r7,YC);
#endif
l4 Q4 X4(yd,oh,LB);Z5(pc,Gf,WC);a6(qc,Hf,JB);X4(zd,ph,ZC);R4
#endif
#if defined(HB)||defined(EB)
p4(r7,xa)
#endif
#ifdef FB
O3 i3(l3,Ad,ED);
#if defined(HB)||defined(EB)
q6(l3,r7,YC);
#endif
#ifdef EB
D5(l3,Bd,FD);
#endif
i3(w5,m4,CC);
#if defined(CB)&&defined(N)&&!defined(W)
E5(XD);
#endif
P3 p4(Ad,ia)
#ifdef EB
p4(Bd,na)
#endif
x5 n4(r5) y5
#endif
#ifdef FB
f bool f6(e U){return U.y>=.0;}f bool f6(C U){return U.y>=.0;}
#endif
#if defined(FB)&&defined(HB)
f bool zc(e U){return U.x<Dd;}f bool Ac(e U){return U.y<Dd;}
#endif
#ifdef BB
e Gd(float Va,c R8,float M1){c r6=(1.-R8*abs(M1))*.5;float q4,F5;if(abs(Va-i7)<1./Q8){q4=.0;F5=.0;}else{float Wa=tan(Va);q4=sign(i7-Va)/max(abs(Wa),1./Fd);F5=q4>=.0?r6.y-(1.-r6.x)*Wa:r6.y+r6.x*Wa;}e U;U.x=max(r6.x,.0)+Ed;U.y=-r6.y+w7;U.z=q4;U.w=F5;return U;}
#endif
#ifdef HB
f d p8(e U S3){d q4=U.z;d F5=max(U.w,.0);d v6=q4>=.0?A5(F5):.0;if(abs(q4)<Q8){d x=abs(U.x)-Ed;d y=-U.y+w7;d h3=(y-F5)*0.5984134206;i t=F5+h3*I0(0.20888568955,0.62665706865,1.04442844776,1.46219982687);i u=t*-q4+(y*q4+x);i qh=I0(A5(u[0]),A5(u[1]),A5(u[2]),A5(u[3]));i Hd=t*5.09593080173+-2.54796540086;i rh=exp2(-Hd*Hd);v6+=dot(qh,rh)*h3;}return v6*sign(U.x);}f d N4(e U S3){float v6=1.;float sh=(1.-w7)+U.x;v6-=A5(sh);float th=1.-U.y;v6-=A5(th);return v6;}
#endif
#ifdef BB
f e0 r4(int Id){return e0(Id&((1<<kd)-1),Id>>kd);}f float Xa(uint z){return float(z)*(F8/(65536.*65536.));}f float uh(uint z){return float(z&0xffffu)*(1./65535.);}
#endif
#if defined(BB)&&defined(MD)
f float Jd(Y S0,c vh){c r2=M0(S0,vh);return(abs(r2.x)+abs(r2.y))*(1./dot(r2,r2));}f bool L9(e x7,e Ya,int r,i1(uint) m3,i1(c) wh
#ifndef CB
,i1(e) X1
#else
,i1(Q) y7
#endif
w6){int T3=int(x7.x);float M1=x7.y;float Za=x7.z;int Kd=floatBitsToInt(x7.w)>>2;int z7=floatBitsToInt(x7.w)&3;int G5=min(T3,Kd-1);int U3=r*Kd+G5;M C2=p1(TB,r4(U3));uint i0=C2.w;uint x6=max(i0&Na,1u);M H5=p0(ZC,x6-1u);c S8=uintBitsToFloat(H5.xy);m3=H5.z&0xffffu;uint T8=H5.w;Y S0=n1(uintBitsToFloat(p0(LB,m3*4u)));M V3=p0(LB,m3*4u+1u);c m2=uintBitsToFloat(V3.xy);float R2=uintBitsToFloat(V3.z);float S2=uintBitsToFloat(V3.w);uint A7=i0&Q2;if(A7!=0u){T3=int(Ya.x);M1=Ya.y;Za=Ya.z;}if(T3!=G5){int U8=U3+T3-G5;M B7=p1(TB,r4(U8));if((B7.w&(Q2|0xffffu))!=(i0&(Q2|0xffffu))){bool xh=R2==.0||S8.x!=.0;if(xh){U3=int(T8);C2=p1(TB,r4(U3));}}else{U3=U8;C2=B7;}i0=(C2.w&~Q2)|A7;}bool ab=false;float y1;
#ifdef HB
float C7;float E1;if((i0&R3)==M8&&z7==P8){uint Ld=C2.z;float v4=float(Ld&0xffffu);float v2=float(Ld>>16);e0 V8=e0(-v4-1.,v2-v4+1.);if((i0&Q2)!=0u) V8=-V8;M Md=p1(TB,r4(U3+V8.x));M bb=p1(TB,r4(U3+V8.y));if((bb.w&(Q2|0xffffu))!=(Md.w&(Q2|0xffffu))){bb=p1(TB,r4(int(T8)));}C7=Xa(Md.z);float Nd=Xa(bb.z);E1=Nd-C7;if(abs(E1)>j4) E1-=F8*sign(E1);float cb=v2+1.-float(nd);float Od=clamp(round(abs(E1)/j4*cb),1.,cb-1.);float D7=cb-Od;if(v4<=D7){E1=-(j4*sign(E1)-E1);v2=D7;if(v4==D7) M1=-M1;}else if(v4==D7+1.){v4=.0;v2=.0;M1=.0;}else{v4-=D7+2.;v2=Od;}if(v4==v2){y1=Nd;}else{y1=C7+E1*(v4/v2);}}else
#endif
{y1=Xa(C2.z);}c O2=c(sin(y1),-cos(y1));c W8=uintBitsToFloat(C2.xy);c X8=c(0,0);if(S2!=.0){S2=max(S2,(Ma/3.)/length(M0(S0,O2)));}if(R2!=.0){M1*=sign(determinant(S0));if((i0&O8)!=0u) M1=min(M1,.0);if((i0&td)!=0u) M1=max(M1,.0);float Y4=S2!=.0?S2:Jd(S0,O2)*I4;d Pd=1.;if(Y4>R2&&S2==.0){Pd=e4(R2)/e4(Y4);R2=Y4;}c I5=O2*(R2+Y4);
#ifndef CB
float x=M1*(R2+Y4);X1.xy=(1./(Y4*2.))*(c(x,-x)+R2)+.5;X1.zw=Y6(.0);
#endif
uint db=i0&R3;if(db>L8){bool Y8=(i0&rd)!=0u;bool yh=(i0&O8)!=0u;float w4=uh(C2.z);float Z8=sqrt(max(1.-w4*w4,.0));if(Y8==yh) Z8=-Z8;Y zh=Y(w4,Z8,-Z8,w4);c a9=M0(zh,O2);float eb=Jd(S0,a9);float fb;if((db==Gg)||(db==Hg&&w4>=.25)){float Ah=(i0&N8)!=0u?1.:.25;fb=R2*(1./max(w4,Ah));}else{fb=R2*w4+eb*.5;}float gb=fb+eb*I4;if((i0&sd)!=0u){float Qd=R2+Y4;float Bh=Y4*.125;if(Qd<=gb*w4+Bh){float Ch=Qd*(1./w4);I5=a9*Ch;}else{c hb=a9*gb;c Dh=c(dot(I5,I5),dot(hb,hb));I5=M0(Dh,inverse(Y(I5,hb)));}}c Eh=abs(M1)*I5;float Rd=(gb-dot(Eh,a9))/(eb*(I4*2.));
#ifndef CB
if((i0&O8)!=0u) X1.y=Rd;else X1.x=Rd;
#endif
}
#ifndef CB
X1.xy*=Pd;X1.y=max(X1.y,1e-4);if(S2!=.0){X1.x=w7-X1.x;}
#endif
X8=M0(S0,M1*I5);if(z7!=P8) ab=true;}else{
#ifndef CB
X1=e(Za,-1.,.0,.0);
#ifdef HB
if(S2!=.0){X1.y=w7;X1.z=Fd;X1.w=Za;if((i0&R3)==M8&&z7==P8){if(E1<.0){C7+=E1;E1=-E1;}float x4=y1-C7;x4=mod(x4+i7,F8)-i7;x4=clamp(x4,.0,E1);if(x4>E1*.5){x4=E1-x4;}c R8=c(sin(x4),cos(x4));
#if 0
float Y1=1.+.33*log2(i7/(j4-min(E1,j4-j4/16.)));e Fh=Gd(E1,R8,.5*(Y1/3.));float Gh=p8(Fh k1);float Hh=Xc(Gh);float Ih=(.5-Hh)*(Ma*2.);float Jh=Y1/max(Ih,Y1);M1*=Jh;
#endif
X1=Gd(E1,R8,M1);}X8=M0(S0,(M1*S2)*O2);}else
#endif
{X8=sign(M0(M1*O2,inverse(S0)))*I4;}if(bool(i0&Q2)!=bool(i0&Ig)){X1*=e(-1.,+1.,+1.,+1.);}
#endif
if(z7==vd) W8=S8;if((i0&qd)!=0u&&z7!=ud){ab=true;}}wh=M0(S0,W8)+X8+m2;
#ifdef CB
M W3=p0(LB,m3*4u+2u);y7=Q1(W3.x);
#else
X1.xy=mix(X1.xy,c(1.,-1.),hg(j.Kh!=0u));
#endif
return!ab;}
#endif
#if defined(BB)&&defined(DB)
f c mc(O y6,i1(uint) m3
#ifdef CB
,i1(Q) y7
#else
,i1(d) Lh
#endif
w6){m3=floatBitsToUint(y6.z)&0xffffu;
#ifdef CB
M W3=p0(LB,m3*4u+2u);y7=Q1(W3.x);
#else
Lh=za(floatBitsToInt(y6.z)>>16);
#endif
c z6=y6.xy;Y S0=n1(uintBitsToFloat(p0(LB,m3*4u)));M V3=p0(LB,m3*4u+1u);c m2=uintBitsToFloat(V3.xy);z6=M0(S0,z6)+m2;return z6;}
#endif
#if defined(BB)&&defined(EB)
f c lc(O y6,i1(uint) m3,
#ifdef CB
i1(Q) y7,
#endif
i1(c) Mh w6){m3=floatBitsToUint(y6.z)&0xffffu;M W3=p0(LB,m3*4u+2u);
#ifdef CB
y7=Q1(W3.x);
#endif
c z6=y6.xy;O E7=uintBitsToFloat(W3.yzw);Mh=(z6*E7.x+E7.yz)*j.Nh;return z6;}
#endif
f d c9(d i2,d N1,d n3){return(N1-i2)/max(1.-i2*n3,K9);}
#if defined(QB)||defined(ID)
f uint d9(O0 o3,uint Oh){uint ib=(o3.y>>p6)*(Oh<<p6)+((o3.x>>p6)<<(p6<<1));ib+=((o3.x&0x1cu)<<p6)+((o3.y&0x1cu)<<2);ib+=((o3.y&0x3u)<<2)+(o3.x&0x3u);return ib;}
#endif
#ifdef QB
#ifdef W
#define z5 z2
#define o4(J5) L1=J5;A3
#else
#define z5 U1
#define o4(J5) y0(n0,J5);h2;
#endif
f d jb(uint Ph){return za(int((Ph&Sa)-C5))*Qa;}f uint F7(d n){return uint(n*Xg+.5);}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive