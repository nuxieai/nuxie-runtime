#pragma once

#include "tessellate.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char tessellate[] = R"===(#define qj 10
#ifdef BB
f1(f0) K(0,e,ND);K(1,e,OD);K(2,e,TC);
#ifdef Sa
K(3,uint,JE);K(4,uint,KE);K(5,uint,LE);K(6,uint,ME);
#else
K(3,O,WB);
#endif
g1
#endif
w2 F0 X(0,e,U6);F0 X(1,e,V6);F0 X(2,e,i5);F0 X(3,M,j5);g3 X(4,uint,l8);l2
#ifdef BB
o4 F6(q3,F7,ZC);p4 y4(F7,Va) W4 g5(Zd,gi,KB);g5(ae,hi,CD);X4 x1(IG,f0,B,F,r){L(r,B,ND,e);L(r,B,OD,e);L(r,B,TC,e);
#ifdef Sa
L(r,B,JE,uint);L(r,B,KE,uint);L(r,B,LE,uint);L(r,B,ME,uint);O WB=O(JE,KE,LE,ME);
#else
L(r,B,WB,O);
#endif
V(U6,e);V(V6,e);V(i5,e);V(j5,M);V(l8,uint);c A0=ND.xy;c D0=ND.zw;c K0=OD.xy;c U0=OD.zw;bool cf=F<4;float y=cf?TC.z:TC.w;int pc=int(cf?WB.x:WB.y);
#ifdef Wa
int df=pc<<16;if(WB.z==0xffffffffu){--df;}float V9=float(df>>16);
#else
float V9=float(pc<<16>>16);
#endif
float W9=float(pc>>16);c E2=c((F&1)==0?V9:W9,(F&2)==0?y+1.:y);if((W9-V9)*j.Ce<.0){E2.y=2.*y+1.-E2.y;}uint f3=WB.z&0x3ffu;uint ef=(WB.z>>10)&0x3ffu;uint C2=WB.z>>20;uint a0=WB.w;uint A6=a0&ob;uint c0=A6>0u?p0(CD,max(A6,1u)-1u).z:0u;O K3=c0!=0u?p0(KB,c0*4u+1u):O(0u,0u,0u,0u);float B2=uintBitsToFloat(K3.z);float Z2=uintBitsToFloat(K3.w);if(Z2!=.0&&B2==.0){float ff;float rj=Bg(A0,D0,K0,U0,ff);float qc=Z2*(1./jb);float sj=wg(A0,D0,K0,U0,ff,qc);float m8=1.-sj*(1./n4);float tj=dot(U0-A0,U0-A0)/(qc*qc);float uj=(tj-1.)*.5;m8=min(m8,uj);m8=min(m8,.99);float vj=.5*m8;float x=Ad(vj)*-2.+1.;float gf=T8(x*Z2,rj);e hf=mix(A0.xyxy,U0.xyxy,e(1./3.,1./3.,2./3.,2./3.));D0=mix(D0,hf.xy,gf);K0=mix(K0,hf.zw,gf);}if((a0&mh)!=0u){W y6=p1(uintBitsToFloat(p0(KB,c0*4u)));c jf=y0(y6,-2.*D0+K0+A0);c kf=y0(y6,-2.*K0+U0+D0);float B1=max(dot(jf,jf),dot(kf,kf));float j4=max(ceil(sqrt(.75*4.*sqrt(B1))),1.);f3=min(uint(j4),f3);}uint X9=f3+ef+C2-1u;W z2=Pa(A0,D0,K0,U0);float h1=acos(P8(z2[0],z2[1]));float I4=h1/float(ef);float rc=determinant(W(K0-A0,U0-D0));if(rc==.0) rc=determinant(z2);if(rc<.0) I4=-I4;U6=e(A0,D0);V6=e(K0,U0);i5=e(float(X9)-abs(W9-E2.x),float(X9),(C2<<10)|f3,I4);j5.xy=TC.xy;if(C2>1u){W sc=W(z2[1],TC.xy);float wj=acos(P8(sc[0],sc[1]));float lf=float(C2);if((a0&(I3|k9))==(E7|k9)){lf-=2.;}float tc=wj/lf;if(determinant(sc)<.0) tc=-tc;j5.z=tc;}if(W9<V9){a0|=Y2;}l8=a0;e I=Y8(E2,2./Zg,j.Ce);
#ifdef MC
I.y=-I.y;
#endif
Z(U6);Z(V6);Z(i5);Z(j5);Z(l8);y1(I);}
#endif
#ifdef EB
U3 V3 W2(O,JG){q(U6,e);q(V6,e);q(i5,e);q(j5,M);q(l8,uint);c A0=U6.xy;c D0=U6.zw;c K0=V6.xy;c U0=V6.zw;W z2=Pa(A0,D0,K0,U0);float xj=max(floor(i5.x),.0);float X9=i5.y;uint mf=uint(i5.z);float f3=float(mf&0x3ffu);float C2=float(mf>>10);float I4=i5.w;uint a0=l8;float x3=X9-C2;float w1=xj;if(w1<=x3){a0&=~I3;}else{A0=D0=K0=U0;z2=W(z2[1],j5.xy);f3=1.;w1-=x3;x3=C2;I4=j5.z;bool nf=(a0&k9)!=0u;if(nf||(a0&I3)==j9){x3-=2.;--w1;}bool yj=nf&&(w1==0.||w1==x3);if(yj){a0&=~I3;}else{a0|=I4<.0?r6:nb;}if((a0&I3)>E7){float Y9=x3*.5;if(w1<Y9) a0|=lb;if(x3>3.&&w1>Y9-1.&&w1<Y9+1.) a0|=mb;w1=w1<Y9?.0:x3;}}c X5;float h1=.0;if(w1==.0||w1==x3){bool E6=w1<x3*.5;X5=E6?A0:U0;h1=Cd(E6?z2[0]:z2[1]);}else if((a0&Ud)!=0u){X5=A0;if(w1>=float(g9/2u)) X5=D0;if(w1>=float(g9*3u/4u)) X5=K0;if(w1>=float(g9*7u/8u)) X5=j5.xy;}else{float F1,Y5;if(f3==x3){F1=w1/f3;Y5=.0;}else{c A,J,y2=D0-A0;c j7=U0-A0;c Q8=K0-D0;J=Q8-y2;A=-3.*Q8+j7;c zj=J*(f3*2.);c l7=y2*(f3*f3);float Z9=.0;float Aj=min(f3-1.,w1);c uc=normalize(z2[0]);float Bj=-abs(I4);float Cj=(1.+w1)*abs(I4);for(int vc=qj-1;vc>=0;--vc){float n8=Z9+exp2(float(vc));if(n8<=Aj){c wc=n8*A+zj;wc=n8*wc+l7;float Dj=dot(normalize(wc),uc);float xc=n8*Bj+Cj;xc=min(xc,n4);if(Dj>=cos(xc)) Z9=n8;}}float Ej=Z9/f3;float of=w1-Z9;float aa=acos(clamp(uc.x,-1.,1.));aa=uc.y>=.0?aa:-aa;h1=of*I4+aa;c P1=c(sin(h1),-cos(h1));float k=dot(P1,A),ba=dot(P1,J),S1=dot(P1,y2);float Fj=max(ba*ba-k*S1,.0);float J2=sqrt(Fj);if(ba>.0) J2=-J2;J2-=ba;float pf=-.5*J2*k;c yc=(abs(J2*J2+pf)<abs(k*S1+pf))?c(J2,k):c(S1,J2);Y5=(yc.y!=.0)?yc.x/yc.y:.0;Y5=clamp(Y5,.0,1.);if(of==.0) Y5=.0;F1=max(Ej,Y5);}c Gj=o6(A0,D0,F1);c qf=o6(D0,K0,F1);c Hj=o6(K0,U0,F1);c rf=o6(Gj,qf,F1);c sf=o6(qf,Hj,F1);X5=o6(rf,sf,F1);if(F1!=Y5) h1=Cd(sf-rf);}O o8;o8.xy=floatBitsToUint(X5);if((a0&I3)==j9){o8.z=(uint(x3)<<16)|uint(w1);}else{uint Ij=uint(int(round(h1*(65536./Z8))))&0xffffu;uint tf=0u;if((a0&I3)>E7){float Jj=clamp(P8(z2[0],z2[1]),-1.,1.);tf=uint(round(sqrt((1.+Jj)*.5)*65535.));}o8.z=(Ij<<16)|tf;}o8.w=a0;K2(o8);}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive