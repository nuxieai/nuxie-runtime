#pragma once

#include "tessellate.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char tessellate[] = R"===(#define vh 10
#ifdef DB
g1(e0)L(0,g,ID);L(1,g,JD);L(2,g,VC);
#ifdef O3
L(3,uint,FE);L(4,uint,GE);L(5,uint,HE);L(6,uint,IE);
#else
L(3,G,UB);
#endif
h1
#endif
m2 H0 X(0,g,A6);H0 X(1,g,B6);H0 X(2,g,N4);H0 X(3,R,O4);Q2 X(4,uint,I7);g2
#ifdef DB
U3 i6(d3,g7,YC);V3 c4(g7,aa)B4 I4(Oc,ig,QB);I4(Pc,jg,FD);C4 y1(YF,e0,F,B,v){M(v,F,ID,g);M(v,F,JD,g);M(v,F,VC,g);
#ifdef O3
M(v,F,FE,uint);M(v,F,GE,uint);M(v,F,HE,uint);M(v,F,IE,uint);G UB=G(FE,GE,HE,IE);
#else
M(v,F,UB,G);
#endif
V(A6,g);V(B6,g);V(N4,g);V(O4,R);V(I7,uint);d r0=ID.xy;d z0=ID.zw;d D0=JD.xy;d K0=JD.zw;bool Pd=B<4;float y=Pd?VC.z:VC.w;int db=int(Pd?UB.x:UB.y);
#ifdef pc
int Qd=db<<16;if(UB.z==0xffffffffu){--Qd;}float Z8=float(Qd>>16);
#else
float Z8=float(db<<16>>16);
#endif
float a9=float(db>>16);d n2=d((B&1)==0?Z8:a9,(B&2)==0?y+1.:y);if((a9-Z8)*m.rd<.0){n2.y=2.*y+1.-n2.y;}uint P2=UB.z&0x3ffu;uint Rd=(UB.z>>10)&0x3ffu;uint k2=UB.z>>20;uint i0=UB.w;uint F8=i0&Kc;uint l0=F8>0u?J0(FD,max(F8,1u)-1u).z:0u;G K4=l0!=0u?J0(QB,l0*4u+1u):G(0u,0u,0u,0u);float J2=uintBitsToFloat(K4.z);float K2=uintBitsToFloat(K4.w);if(K2!=.0&&J2==.0){float Sd;float wh=ef(r0,z0,D0,K0,Sd);float eb=K2*(1./na);float xh=Ze(r0,z0,D0,K0,Sd,eb);float J7=1.-xh*(1./D3);float yh=dot(K0-r0,K0-r0)/(eb*eb);float zh=(yh-1.)*.5;J7=min(J7,zh);J7=min(J7,.99);float Ah=.5*J7;float x=oc(Ah)*-2.+1.;float Td=k8(x*K2,wh);g Ud=mix(r0.xyxy,K0.xyxy,g(1./3.,1./3.,2./3.,2./3.));z0=mix(z0,Ud.xy,Td);D0=mix(D0,Ud.zw,Td);}if((i0&Df)!=0u){f0 Vd=h2(uintBitsToFloat(J0(QB,l0*4u)));d Wd=R0(Vd,-2.*z0+D0+r0);d Xd=R0(Vd,-2.*D0+K0+z0);float l1=max(dot(Wd,Wd),dot(Xd,Xd));float P3=max(ceil(sqrt(.75*4.*sqrt(l1))),1.);P2=min(uint(P3),P2);}uint c9=P2+Rd+k2-1u;f0 H2=T9(r0,z0,D0,K0);float e1=acos(S9(H2[0],H2[1]));float m4=e1/float(Rd);float fb=determinant(f0(D0-r0,K0-z0));if(fb==.0)fb=determinant(H2);if(fb<.0)m4=-m4;A6=g(r0,z0);B6=g(D0,K0);N4=g(float(c9)-abs(a9-n2.x),float(c9),(k2<<10)|P2,m4);O4.xy=VC.xy;if(k2>1u){f0 gb=f0(H2[1],VC.xy);float Bh=acos(S9(gb[0],gb[1]));float Yd=float(k2);if((i0&(a4|z8))==(x8|z8)){Yd-=2.;}float hb=Bh/Yd;if(determinant(gb)<.0)hb=-hb;O4.z=hb;}if(a9<Z8){i0|=G3;}I7=i0;g W=o8(n2,2./Af,m.rd);
#ifdef SC
W.y=-W.y;
#endif
a0(A6);a0(B6);a0(N4);a0(O4);a0(I7);z1(W);}
#endif
#ifdef GB
E3 F3 a3(D4,ZF){r(A6,g);r(B6,g);r(N4,g);r(O4,R);r(I7,uint);d r0=A6.xy;d z0=A6.zw;d D0=B6.xy;d K0=B6.zw;f0 H2=T9(r0,z0,D0,K0);float Ch=max(floor(N4.x),.0);float c9=N4.y;uint Zd=uint(N4.z);float P2=float(Zd&0x3ffu);float k2=float(Zd>>10);float m4=N4.w;uint i0=I7;float P4=c9-k2;float U1=Ch;if(U1<=P4){i0&=~a4;}else{r0=z0=D0=K0;H2=f0(H2[1],O4.xy);P2=1.;U1-=P4;P4=k2;m4=O4.z;if((i0&a4)>x8){if(U1<2.5)i0|=oa;if(U1>1.5&&U1<3.5)i0|=Ic;}else if((i0&z8)!=0u||(i0&a4)==y8){P4-=2.;--U1;}i0|=m4<.0?A8:Jc;}d E5;float e1=.0;if(U1==.0||U1==P4||(i0&a4)>x8){bool I8=U1<P4*.5;E5=I8?r0:K0;e1=rc(I8?H2[0]:H2[1]);}else if((i0&Hc)!=0u){E5=r0;if(U1>=float(la/2u))E5=z0;if(U1>=float(la*3u/4u))E5=D0;if(U1>=float(la*7u/8u))E5=O4.xy;}else{float r1,F5;if(P2==P4){r1=U1/P2;F5=.0;}else{d C,H,i2=z0-r0;d N6=K0-r0;d h8=D0-z0;H=h8-i2;C=-3.*h8+N6;d Dh=H*(P2*2.);d P6=i2*(P2*P2);float d9=.0;float Eh=min(P2-1.,U1);d ib=normalize(H2[0]);float Fh=-abs(m4);float Gh=(1.+U1)*abs(m4);for(int jb=vh-1;jb>=0;--jb){float K7=d9+exp2(float(jb));if(K7<=Eh){d kb=K7*C+Dh;kb=K7*kb+P6;float Hh=dot(normalize(kb),ib);float lb=K7*Fh+Gh;lb=min(lb,D3);if(Hh>=cos(lb))d9=K7;}}float Ih=d9/P2;float ae=U1-d9;float e9=acos(clamp(ib.x,-1.,1.));e9=ib.y>=.0?e9:-e9;e1=ae*m4+e9;d Y2=d(sin(e1),-cos(e1));float o=dot(Y2,C),f9=dot(Y2,H),G1=dot(Y2,i2);float Jh=max(f9*f9-o*G1,.0);float r2=sqrt(Jh);if(f9>.0)r2=-r2;r2-=f9;float be=-.5*r2*o;d mb=(abs(r2*r2+be)<abs(o*G1+be))?d(r2,o):d(G1,r2);F5=(mb.y!=.0)?mb.x/mb.y:.0;F5=clamp(F5,.0,1.);if(ae==.0)F5=.0;r1=max(Ih,F5);}d Kh=c6(r0,z0,r1);d ce=c6(z0,D0,r1);d Lh=c6(D0,K0,r1);d de=c6(Kh,ce,r1);d ee=c6(ce,Lh,r1);E5=c6(de,ee,r1);if(r1!=F5)e1=rc(ee-de);}D4 L7;L7.xy=Y9(E5);if((i0&a4)==y8){L7.z=Z9((uint(P4)<<16)|uint(U1));}else{L7.z=Y9(mod(e1,p8));}L7.w=Z9(i0);I2(L7);}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive