#pragma once

#include "tessellate.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char tessellate[] = R"===(#define Ah 10
#ifdef DB
g1(e0)L(0,g,JD);L(1,g,KD);L(2,g,VC);
#ifdef O3
L(3,uint,FE);L(4,uint,GE);L(5,uint,HE);L(6,uint,IE);
#else
L(3,H,UB);
#endif
h1
#endif
m2 H0 X(0,g,C6);H0 X(1,g,D6);H0 X(2,g,P4);H0 X(3,R,Q4);Q2 X(4,uint,J7);g2
#ifdef DB
U3 j6(d3,i7,YC);V3 c4(i7,ba)C4 K4(Pc,mg,QB);K4(Qc,ng,FD);D4 z1(BG,e0,F,B,v){M(v,F,JD,g);M(v,F,KD,g);M(v,F,VC,g);
#ifdef O3
M(v,F,FE,uint);M(v,F,GE,uint);M(v,F,HE,uint);M(v,F,IE,uint);H UB=H(FE,GE,HE,IE);
#else
M(v,F,UB,H);
#endif
V(C6,g);V(D6,g);V(P4,g);V(Q4,R);V(J7,uint);d r0=JD.xy;d z0=JD.zw;d D0=KD.xy;d K0=KD.zw;bool Td=B<4;float y=Td?VC.z:VC.w;int fb=int(Td?UB.x:UB.y);
#ifdef rc
int Ud=fb<<16;if(UB.z==0xffffffffu){--Ud;}float a9=float(Ud>>16);
#else
float a9=float(fb<<16>>16);
#endif
float c9=float(fb>>16);d n2=d((B&1)==0?a9:c9,(B&2)==0?y+1.:y);if((c9-a9)*m.sd<.0){n2.y=2.*y+1.-n2.y;}uint P2=UB.z&0x3ffu;uint Vd=(UB.z>>10)&0x3ffu;uint k2=UB.z>>20;uint i0=UB.w;uint G8=i0&Lc;uint l0=G8>0u?J0(FD,max(G8,1u)-1u).z:0u;H M4=l0!=0u?J0(QB,l0*4u+1u):H(0u,0u,0u,0u);float J2=uintBitsToFloat(M4.z);float K2=uintBitsToFloat(M4.w);if(K2!=.0&&J2==.0){float Wd;float Bh=jf(r0,z0,D0,K0,Wd);float gb=K2*(1./pa);float Ch=df(r0,z0,D0,K0,Wd,gb);float K7=1.-Ch*(1./D3);float Dh=dot(K0-r0,K0-r0)/(gb*gb);float Eh=(Dh-1.)*.5;K7=min(K7,Eh);K7=min(K7,.99);float Fh=.5*K7;float x=qc(Fh)*-2.+1.;float Xd=l8(x*K2,Bh);g Yd=mix(r0.xyxy,K0.xyxy,g(1./3.,1./3.,2./3.,2./3.));z0=mix(z0,Yd.xy,Xd);D0=mix(D0,Yd.zw,Xd);}if((i0&Hf)!=0u){f0 Zd=h2(uintBitsToFloat(J0(QB,l0*4u)));d ae=R0(Zd,-2.*z0+D0+r0);d be=R0(Zd,-2.*D0+K0+z0);float l1=max(dot(ae,ae),dot(be,be));float P3=max(ceil(sqrt(.75*4.*sqrt(l1))),1.);P2=min(uint(P3),P2);}uint d9=P2+Vd+k2-1u;f0 H2=U9(r0,z0,D0,K0);float e1=acos(T9(H2[0],H2[1]));float n4=e1/float(Vd);float hb=determinant(f0(D0-r0,K0-z0));if(hb==.0)hb=determinant(H2);if(hb<.0)n4=-n4;C6=g(r0,z0);D6=g(D0,K0);P4=g(float(d9)-abs(c9-n2.x),float(d9),(k2<<10)|P2,n4);Q4.xy=VC.xy;if(k2>1u){f0 ib=f0(H2[1],VC.xy);float Gh=acos(T9(ib[0],ib[1]));float ce=float(k2);if((i0&(a4|A8))==(y8|A8)){ce-=2.;}float jb=Gh/ce;if(determinant(ib)<.0)jb=-jb;Q4.z=jb;}if(c9<a9){i0|=G3;}J7=i0;g W=p8(n2,2./Ef,m.sd);
#ifdef SC
W.y=-W.y;
#endif
c0(C6);c0(D6);c0(P4);c0(Q4);c0(J7);A1(W);}
#endif
#ifdef GB
E3 F3 a3(E4,CG){r(C6,g);r(D6,g);r(P4,g);r(Q4,R);r(J7,uint);d r0=C6.xy;d z0=C6.zw;d D0=D6.xy;d K0=D6.zw;f0 H2=U9(r0,z0,D0,K0);float Hh=max(floor(P4.x),.0);float d9=P4.y;uint de=uint(P4.z);float P2=float(de&0x3ffu);float k2=float(de>>10);float n4=P4.w;uint i0=J7;float R4=d9-k2;float U1=Hh;if(U1<=R4){i0&=~a4;}else{r0=z0=D0=K0;H2=f0(H2[1],Q4.xy);P2=1.;U1-=R4;R4=k2;n4=Q4.z;if((i0&a4)>y8){if(U1<2.5)i0|=qa;if(U1>1.5&&U1<3.5)i0|=Jc;}else if((i0&A8)!=0u||(i0&a4)==z8){R4-=2.;--U1;}i0|=n4<.0?B8:Kc;}d F5;float e1=.0;if(U1==.0||U1==R4||(i0&a4)>y8){bool J8=U1<R4*.5;F5=J8?r0:K0;e1=tc(J8?H2[0]:H2[1]);}else if((i0&Ic)!=0u){F5=r0;if(U1>=float(na/2u))F5=z0;if(U1>=float(na*3u/4u))F5=D0;if(U1>=float(na*7u/8u))F5=Q4.xy;}else{float r1,G5;if(P2==R4){r1=U1/P2;G5=.0;}else{d C,I,i2=z0-r0;d P6=K0-r0;d i8=D0-z0;I=i8-i2;C=-3.*i8+P6;d Ih=I*(P2*2.);d R6=i2*(P2*P2);float e9=.0;float Jh=min(P2-1.,U1);d kb=normalize(H2[0]);float Kh=-abs(n4);float Lh=(1.+U1)*abs(n4);for(int lb=Ah-1;lb>=0;--lb){float L7=e9+exp2(float(lb));if(L7<=Jh){d mb=L7*C+Ih;mb=L7*mb+R6;float Mh=dot(normalize(mb),kb);float nb=L7*Kh+Lh;nb=min(nb,D3);if(Mh>=cos(nb))e9=L7;}}float Nh=e9/P2;float ee=U1-e9;float f9=acos(clamp(kb.x,-1.,1.));f9=kb.y>=.0?f9:-f9;e1=ee*n4+f9;d Y2=d(sin(e1),-cos(e1));float o=dot(Y2,C),g9=dot(Y2,I),H1=dot(Y2,i2);float Oh=max(g9*g9-o*H1,.0);float r2=sqrt(Oh);if(g9>.0)r2=-r2;r2-=g9;float fe=-.5*r2*o;d ob=(abs(r2*r2+fe)<abs(o*H1+fe))?d(r2,o):d(H1,r2);G5=(ob.y!=.0)?ob.x/ob.y:.0;G5=clamp(G5,.0,1.);if(ee==.0)G5=.0;r1=max(Nh,G5);}d Ph=d6(r0,z0,r1);d ge=d6(z0,D0,r1);d Qh=d6(D0,K0,r1);d he=d6(Ph,ge,r1);d ie=d6(ge,Qh,r1);F5=d6(he,ie,r1);if(r1!=G5)e1=tc(ie-he);}E4 M7;M7.xy=Z9(F5);if((i0&a4)==z8){M7.z=aa((uint(R4)<<16)|uint(U1));}else{M7.z=Z9(mod(e1,q8));}M7.w=aa(i0);I2(M7);}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive