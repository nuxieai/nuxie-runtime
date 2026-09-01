#pragma once

#include "tessellate.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char tessellate[] = R"===(#define uh 10
#ifdef DB
g1(e0)L(0,g,ID);L(1,g,JD);L(2,g,VC);
#ifdef O3
L(3,uint,FE);L(4,uint,GE);L(5,uint,HE);L(6,uint,IE);
#else
L(3,G,UB);
#endif
h1
#endif
l2 H0 X(0,g,z6);H0 X(1,g,A6);H0 X(2,g,N4);H0 X(3,R,D5);Q2 X(4,uint,H7);f2
#ifdef DB
U3 h6(d3,f7,YC);V3 c4(f7,aa)B4 I4(Nc,hg,QB);I4(Oc,ig,FD);C4 y1(YF,e0,F,B,v){M(v,F,ID,g);M(v,F,JD,g);M(v,F,VC,g);
#ifdef O3
M(v,F,FE,uint);M(v,F,GE,uint);M(v,F,HE,uint);M(v,F,IE,uint);G UB=G(FE,GE,HE,IE);
#else
M(v,F,UB,G);
#endif
V(z6,g);V(A6,g);V(N4,g);V(D5,R);V(H7,uint);d y0=ID.xy;d z0=ID.zw;d F0=JD.xy;d K0=JD.zw;bool Od=B<4;float y=Od?VC.z:VC.w;int cb=int(Od?UB.x:UB.y);
#ifdef oc
int Pd=cb<<16;if(UB.z==0xffffffffu){--Pd;}float Y8=float(Pd>>16);
#else
float Y8=float(cb<<16>>16);
#endif
float Z8=float(cb>>16);d m2=d((B&1)==0?Y8:Z8,(B&2)==0?y+1.:y);if((Z8-Y8)*m.qd<.0){m2.y=2.*y+1.-m2.y;}uint P2=UB.z&0x3ffu;uint Qd=(UB.z>>10)&0x3ffu;uint j2=UB.z>>20;uint i0=UB.w;uint E8=i0&Jc;uint l0=E8>0u?J0(FD,max(E8,1u)-1u).z:0u;G K4=l0!=0u?J0(QB,l0*4u+1u):G(0u,0u,0u,0u);float J2=uintBitsToFloat(K4.z);float K2=uintBitsToFloat(K4.w);if(K2!=.0&&J2==.0){float Rd;float vh=df(y0,z0,F0,K0,Rd);float db=K2*(1./ma);float wh=Ye(y0,z0,F0,K0,Rd,db);float I7=1.-wh*(1./D3);float xh=dot(K0-y0,K0-y0)/(db*db);float yh=(xh-1.)*.5;I7=min(I7,yh);I7=min(I7,.99);float zh=.5*I7;float x=nc(zh)*-2.+1.;float Sd=j8(x*K2,vh);g Td=mix(y0.xyxy,K0.xyxy,g(1./3.,1./3.,2./3.,2./3.));z0=mix(z0,Td.xy,Sd);F0=mix(F0,Td.zw,Sd);}if((i0&Cf)!=0u){f0 Ud=g2(uintBitsToFloat(J0(QB,l0*4u)));d Vd=R0(Ud,-2.*z0+F0+y0);d Wd=R0(Ud,-2.*F0+K0+z0);float l1=max(dot(Vd,Vd),dot(Wd,Wd));float P3=max(ceil(sqrt(.75*4.*sqrt(l1))),1.);P2=min(uint(P3),P2);}uint a9=P2+Qd+j2-1u;f0 H2=T9(y0,z0,F0,K0);float e1=acos(S9(H2[0],H2[1]));float m4=e1/float(Qd);float eb=determinant(f0(F0-y0,K0-z0));if(eb==.0)eb=determinant(H2);if(eb<.0)m4=-m4;z6=g(y0,z0);A6=g(F0,K0);N4=g(float(a9)-abs(Z8-m2.x),float(a9),(j2<<10)|P2,m4);if(j2>1u){f0 fb=f0(H2[1],VC.xy);float Ah=acos(S9(fb[0],fb[1]));float Xd=float(j2);if((i0&(a4|y8))==(w8|y8)){Xd-=2.;}float gb=Ah/Xd;if(determinant(fb)<.0)gb=-gb;D5.xy=VC.xy;D5.z=gb;}if(Z8<Y8){i0|=G3;}H7=i0;g W=n8(m2,2./zf,m.qd);
#ifdef SC
W.y=-W.y;
#endif
a0(z6);a0(A6);a0(N4);a0(D5);a0(H7);z1(W);}
#endif
#ifdef GB
E3 F3 a3(D4,ZF){r(z6,g);r(A6,g);r(N4,g);r(D5,R);r(H7,uint);d y0=z6.xy;d z0=z6.zw;d F0=A6.xy;d K0=A6.zw;f0 H2=T9(y0,z0,F0,K0);float Bh=max(floor(N4.x),.0);float a9=N4.y;uint Yd=uint(N4.z);float P2=float(Yd&0x3ffu);float j2=float(Yd>>10);float m4=N4.w;uint i0=H7;float O4=a9-j2;float y2=Bh;if(y2<=O4){i0&=~a4;}else{y0=z0=F0=K0;H2=f0(H2[1],D5.xy);P2=1.;y2-=O4;O4=j2;m4=D5.z;if((i0&a4)>w8){if(y2<2.5)i0|=na;if(y2>1.5&&y2<3.5)i0|=Hc;}else if((i0&y8)!=0u||(i0&a4)==x8){O4-=2.;--y2;}i0|=m4<.0?z8:Ic;}d c9;float e1=.0;if(y2==.0||y2==O4||(i0&a4)>w8){bool H8=y2<O4*.5;c9=H8?y0:K0;e1=qc(H8?H2[0]:H2[1]);}else if((i0&Gc)!=0u){c9=z0;}else{float r1,E5;if(P2==O4){r1=y2/P2;E5=.0;}else{d C,H,h2=z0-y0;d M6=K0-y0;d g8=F0-z0;H=g8-h2;C=-3.*g8+M6;d Ch=H*(P2*2.);d O6=h2*(P2*P2);float d9=.0;float Dh=min(P2-1.,y2);d hb=normalize(H2[0]);float Eh=-abs(m4);float Fh=(1.+y2)*abs(m4);for(int ib=uh-1;ib>=0;--ib){float J7=d9+exp2(float(ib));if(J7<=Dh){d jb=J7*C+Ch;jb=J7*jb+O6;float Gh=dot(normalize(jb),hb);float kb=J7*Eh+Fh;kb=min(kb,D3);if(Gh>=cos(kb))d9=J7;}}float Hh=d9/P2;float Zd=y2-d9;float e9=acos(clamp(hb.x,-1.,1.));e9=hb.y>=.0?e9:-e9;e1=Zd*m4+e9;d Y2=d(sin(e1),-cos(e1));float o=dot(Y2,C),f9=dot(Y2,H),G1=dot(Y2,h2);float Ih=max(f9*f9-o*G1,.0);float q2=sqrt(Ih);if(f9>.0)q2=-q2;q2-=f9;float ae=-.5*q2*o;d lb=(abs(q2*q2+ae)<abs(o*G1+ae))?d(q2,o):d(G1,q2);E5=(lb.y!=.0)?lb.x/lb.y:.0;E5=clamp(E5,.0,1.);if(Zd==.0)E5=.0;r1=max(Hh,E5);}d Jh=a6(y0,z0,r1);d be=a6(z0,F0,r1);d Kh=a6(F0,K0,r1);d ce=a6(Jh,be,r1);d de=a6(be,Kh,r1);c9=a6(ce,de,r1);if(r1!=E5)e1=qc(de-ce);}D4 K7;K7.xy=Y9(c9);if((i0&a4)==x8){K7.z=Z9((uint(O4)<<16)|uint(y2));}else{K7.z=Y9(mod(e1,o8));}K7.w=Z9(i0);I2(K7);}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive