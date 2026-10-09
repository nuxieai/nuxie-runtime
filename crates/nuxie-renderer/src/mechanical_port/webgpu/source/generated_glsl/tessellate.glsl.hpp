#pragma once

#include "tessellate.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char tessellate[] = R"===(#define oj 10
#ifdef BB
d1(f0) K(0,f,ND);K(1,f,OD);K(2,f,SC);
#ifdef Xa
K(3,uint,JE);K(4,uint,KE);K(5,uint,LE);K(6,uint,ME);
#else
K(3,O,WB);
#endif
e1
#endif
v2 F0 W(0,f,Y6);F0 W(1,f,Z6);F0 W(2,f,l5);F0 W(3,M,m5);g3 W(4,uint,n8);k2
#ifdef BB
q4 I6(q3,I7,YC);r4 a4(I7,ab) Y4 j5(ae,di,KB);j5(be,ei,BD);Z4 w1(IG,f0,B,F,r){L(r,B,ND,f);L(r,B,OD,f);L(r,B,SC,f);
#ifdef Xa
L(r,B,JE,uint);L(r,B,KE,uint);L(r,B,LE,uint);L(r,B,ME,uint);O WB=O(JE,KE,LE,ME);
#else
L(r,B,WB,O);
#endif
V(Y6,f);V(Z6,f);V(l5,f);V(m5,M);V(n8,uint);c z0=ND.xy;c D0=ND.zw;c K0=OD.xy;c T0=OD.zw;bool cf=F<4;float y=cf?SC.z:SC.w;int vc=int(cf?WB.x:WB.y);
#ifdef bb
int df=vc<<16;if(WB.z==0xffffffffu){--df;}float Z9=float(df>>16);
#else
float Z9=float(vc<<16>>16);
#endif
float aa=float(vc>>16);c E2=c((F&1)==0?Z9:aa,(F&2)==0?y+1.:y);if((aa-Z9)*j.Ce<.0){E2.y=2.*y+1.-E2.y;}uint f3=WB.z&0x3ffu;uint ef=(WB.z>>10)&0x3ffu;uint C2=WB.z>>20;uint a0=WB.w;uint D6=a0&tb;uint c0=D6>0u?p0(BD,max(D6,1u)-1u).z:0u;O L3=c0!=0u?p0(KB,c0*4u+1u):O(0u,0u,0u,0u);float B2=uintBitsToFloat(L3.z);float Y2=uintBitsToFloat(L3.w);if(Y2!=.0&&B2==.0){float ff;float pj=Cg(z0,D0,K0,T0,ff);float wc=Y2*(1./ob);float qj=xg(z0,D0,K0,T0,ff,wc);float o8=1.-qj*(1./p4);float rj=dot(T0-z0,T0-z0)/(wc*wc);float sj=(rj-1.)*.5;o8=min(o8,sj);o8=min(o8,.99);float tj=.5*o8;float x=Bd(tj)*-2.+1.;float gf=Y8(x*Y2,pj);f hf=mix(z0.xyxy,T0.xyxy,f(1./3.,1./3.,2./3.,2./3.));D0=mix(D0,hf.xy,gf);K0=mix(K0,hf.zw,gf);}if((a0&nh)!=0u){X R9=o1(uintBitsToFloat(p0(KB,c0*4u)));c jf=B0(R9,-2.*D0+K0+z0);c kf=B0(R9,-2.*K0+T0+D0);float A1=max(dot(jf,jf),dot(kf,kf));float j4=max(ceil(sqrt(.75*4.*sqrt(A1))),1.);f3=min(uint(j4),f3);}uint ba=f3+ef+C2-1u;X z2=Ua(z0,D0,K0,T0);float f1=acos(U8(z2[0],z2[1]));float K4=f1/float(ef);float xc=determinant(X(K0-z0,T0-D0));if(xc==.0) xc=determinant(z2);if(xc<.0) K4=-K4;Y6=f(z0,D0);Z6=f(K0,T0);l5=f(float(ba)-abs(aa-E2.x),float(ba),(C2<<10)|f3,K4);m5.xy=SC.xy;if(C2>1u){X yc=X(z2[1],SC.xy);float uj=acos(U8(yc[0],yc[1]));float lf=float(C2);if((a0&(J3|n9))==(H7|n9)){lf-=2.;}float zc=uj/lf;if(determinant(yc)<.0) zc=-zc;m5.z=zc;}if(aa<Z9){a0|=X2;}n8=a0;f I=d9(E2,2./ah,j.Ce);
#ifdef MC
I.y=-I.y;
#endif
Z(Y6);Z(Z6);Z(l5);Z(m5);Z(n8);x1(I);}
#endif
#ifdef EB
V3 W3 V2(O,JG){q(Y6,f);q(Z6,f);q(l5,f);q(m5,M);q(n8,uint);c z0=Y6.xy;c D0=Y6.zw;c K0=Z6.xy;c T0=Z6.zw;X z2=Ua(z0,D0,K0,T0);float vj=max(floor(l5.x),.0);float ba=l5.y;uint mf=uint(l5.z);float f3=float(mf&0x3ffu);float C2=float(mf>>10);float K4=l5.w;uint a0=n8;float y3=ba-C2;float v1=vj;if(v1<=y3){a0&=~J3;}else{z0=D0=K0=T0;z2=X(z2[1],m5.xy);f3=1.;v1-=y3;y3=C2;K4=m5.z;bool nf=(a0&n9)!=0u;if(nf||(a0&J3)==m9){y3-=2.;--v1;}bool wj=nf&&(v1==0.||v1==y3);if(wj){a0&=~J3;}else{a0|=K4<.0?y6:sb;}if((a0&J3)>H7){float ca=y3*.5;if(v1<ca) a0|=qb;if(y3>3.&&v1>ca-1.&&v1<ca+1.) a0|=rb;v1=v1<ca?.0:y3;}}c a6;float f1=.0;if(v1==.0||v1==y3){bool H6=v1<y3*.5;a6=H6?z0:T0;f1=Dd(H6?z2[0]:z2[1]);}else if((a0&Vd)!=0u){a6=z0;if(v1>=float(k9/2u)) a6=D0;if(v1>=float(k9*3u/4u)) a6=K0;if(v1>=float(k9*7u/8u)) a6=m5.xy;}else{float E1,c6;if(f3==y3){E1=v1/f3;c6=.0;}else{c A,J,y2=D0-z0;c m7=T0-z0;c V8=K0-D0;J=V8-y2;A=-3.*V8+m7;c xj=J*(f3*2.);c o7=y2*(f3*f3);float da=.0;float yj=min(f3-1.,v1);c Ac=normalize(z2[0]);float zj=-abs(K4);float Aj=(1.+v1)*abs(K4);for(int Bc=oj-1;Bc>=0;--Bc){float p8=da+exp2(float(Bc));if(p8<=yj){c Cc=p8*A+xj;Cc=p8*Cc+o7;float Bj=dot(normalize(Cc),Ac);float Dc=p8*zj+Aj;Dc=min(Dc,p4);if(Bj>=cos(Dc)) da=p8;}}float Cj=da/f3;float of=v1-da;float ea=acos(clamp(Ac.x,-1.,1.));ea=Ac.y>=.0?ea:-ea;f1=of*K4+ea;c N1=c(sin(f1),-cos(f1));float m=dot(N1,A),fa=dot(N1,J),R1=dot(N1,y2);float Dj=max(fa*fa-m*R1,.0);float J2=sqrt(Dj);if(fa>.0) J2=-J2;J2-=fa;float pf=-.5*J2*m;c Ec=(abs(J2*J2+pf)<abs(m*R1+pf))?c(J2,m):c(R1,J2);c6=(Ec.y!=.0)?Ec.x/Ec.y:.0;c6=clamp(c6,.0,1.);if(of==.0) c6=.0;E1=max(Cj,c6);}c Ej=v6(z0,D0,E1);c qf=v6(D0,K0,E1);c Fj=v6(K0,T0,E1);c rf=v6(Ej,qf,E1);c sf=v6(qf,Fj,E1);a6=v6(rf,sf,E1);if(E1!=c6) f1=Dd(sf-rf);}O q8;q8.xy=floatBitsToUint(a6);if((a0&J3)==m9){q8.z=(uint(y3)<<16)|uint(v1);}else{uint Gj=uint(int(round(f1*(65536./e9))))&0xffffu;uint tf=0u;if((a0&J3)>H7){float Hj=clamp(U8(z2[0],z2[1]),-1.,1.);tf=uint(round(sqrt((1.+Hj)*.5)*65535.));}q8.z=(Gj<<16)|tf;}q8.w=a0;K2(q8);}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive