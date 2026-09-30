#pragma once

#include "tessellate.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char tessellate[] = R"===(#define ri 10
#ifdef BB
c1(d0) K(0,e,LD);K(1,e,MD);K(2,e,VC);
#ifdef ta
K(3,uint,IE);K(4,uint,JE);K(5,uint,KE);K(6,uint,LE);
#else
K(3,N,VB);
#endif
d1
#endif
l2 E0 W(0,e,K6);E0 W(1,e,L6);E0 W(2,e,Z4);E0 W(3,P,a5);a3 W(4,uint,S7);d2
#ifdef BB
j4 p6(l3,v7,ZC);k4 o4(v7,wa) P4 W4(xd,dh,LB);W4(yd,eh,AD);Q4 r1(HG,d0,D,G,r){L(r,D,LD,e);L(r,D,MD,e);L(r,D,VC,e);
#ifdef ta
L(r,D,IE,uint);L(r,D,JE,uint);L(r,D,KE,uint);L(r,D,LE,uint);N VB=N(IE,JE,KE,LE);
#else
L(r,D,VB,N);
#endif
T(K6,e);T(L6,e);T(Z4,e);T(a5,P);T(S7,uint);c y0=LD.xy;c C0=LD.zw;c H0=MD.xy;c P0=MD.zw;bool ue=G<4;float y=ue?VC.z:VC.w;int Fb=int(ue?VB.x:VB.y);
#ifdef xa
int ve=Fb<<16;if(VB.z==0xffffffffu){--ve;}float q9=float(ve>>16);
#else
float q9=float(Fb<<16>>16);
#endif
float r9=float(Fb>>16);c z2=c((G&1)==0?q9:r9,(G&2)==0?y+1.:y);if((r9-q9)*j.Td<.0){z2.y=2.*y+1.-z2.y;}uint Z2=VB.z&0x3ffu;uint we=(VB.z>>10)&0x3ffu;uint w2=VB.z>>20;uint i0=VB.w;uint w6=i0&Ma;uint a0=w6>0u?p0(AD,max(w6,1u)-1u).z:0u;N V3=a0!=0u?p0(LB,a0*4u+1u):N(0u,0u,0u,0u);float S2=uintBitsToFloat(V3.z);float T2=uintBitsToFloat(V3.w);if(T2!=.0&&S2==.0){float xe;float si=Tf(y0,C0,H0,P0,xe);float Gb=T2*(1./La);float ti=Of(y0,C0,H0,P0,xe,Gb);float T7=1.-ti*(1./i4);float ui=dot(P0-y0,P0-y0)/(Gb*Gb);float vi=(ui-1.)*.5;T7=min(T7,vi);T7=min(T7,.99);float wi=.5*T7;float x=Wc(wi)*-2.+1.;float ye=A8(x*T2,si);e ze=mix(y0.xyxy,P0.xyxy,e(1./3.,1./3.,2./3.,2./3.));C0=mix(C0,ze.xy,ye);H0=mix(H0,ze.zw,ye);}if((i0&Ag)!=0u){Y j9=n1(uintBitsToFloat(p0(LB,a0*4u)));c Ae=K0(j9,-2.*C0+H0+y0);c Be=K0(j9,-2.*H0+P0+C0);float x1=max(dot(Ae,Ae),dot(Be,Be));float e4=max(ceil(sqrt(.75*4.*sqrt(x1))),1.);Z2=min(uint(e4),Z2);}uint v9=Z2+we+w2-1u;Y r2=qa(y0,C0,H0,P0);float w1=acos(w8(r2[0],r2[1]));float D4=w1/float(we);float Hb=determinant(Y(H0-y0,P0-C0));if(Hb==.0) Hb=determinant(r2);if(Hb<.0) D4=-D4;K6=e(y0,C0);L6=e(H0,P0);Z4=e(float(v9)-abs(r9-z2.x),float(v9),(w2<<10)|Z2,D4);a5.xy=VC.xy;if(w2>1u){Y Ib=Y(r2[1],VC.xy);float xi=acos(w8(Ib[0],Ib[1]));float Ce=float(w2);if((i0&(R3|M8))==(r7|M8)){Ce-=2.;}float Jb=xi/Ce;if(determinant(Ib)<.0) Jb=-Jb;a5.z=Jb;}if(r9<q9){i0|=R2;}S7=i0;e I=E8(z2,2./rg,j.Td);
#ifdef NC
I.y=-I.y;
#endif
Z(K6);Z(L6);Z(Z4);Z(a5);Z(S7);v1(I);}
#endif
#ifdef EB
O3 P3 j3(N,IG){q(K6,e);q(L6,e);q(Z4,e);q(a5,P);q(S7,uint);c y0=K6.xy;c C0=K6.zw;c H0=L6.xy;c P0=L6.zw;Y r2=qa(y0,C0,H0,P0);float yi=max(floor(Z4.x),.0);float v9=Z4.y;uint De=uint(Z4.z);float Z2=float(De&0x3ffu);float w2=float(De>>10);float D4=Z4.w;uint i0=S7;float c5=v9-w2;float a2=yi;if(a2<=c5){i0&=~R3;}else{y0=C0=H0=P0;r2=Y(r2[1],a5.xy);Z2=1.;a2-=c5;c5=w2;D4=a5.z;if((i0&R3)>r7){if(a2<2.5) i0|=qd;if(a2>1.5&&a2<3.5) i0|=rd;}else if((i0&M8)!=0u||(i0&R3)==L8){c5-=2.;--a2;}i0|=D4<.0?N8:sd;}c R5;float w1=.0;if(a2==.0||a2==c5||(i0&R3)>r7){bool X8=a2<c5*.5;R5=X8?y0:P0;w1=Yc(X8?r2[0]:r2[1]);}else if((i0&pd)!=0u){R5=y0;if(a2>=float(K8/2u)) R5=C0;if(a2>=float(K8*3u/4u)) R5=H0;if(a2>=float(K8*7u/8u)) R5=a5.xy;}else{float B1,S5;if(Z2==c5){B1=a2/Z2;S5=.0;}else{c B,J,q2=C0-y0;c Y6=P0-y0;c x8=H0-C0;J=x8-q2;B=-3.*x8+Y6;c zi=J*(Z2*2.);c a7=q2*(Z2*Z2);float w9=.0;float Ai=min(Z2-1.,a2);c Kb=normalize(r2[0]);float Bi=-abs(D4);float Ci=(1.+a2)*abs(D4);for(int Lb=ri-1;Lb>=0;--Lb){float U7=w9+exp2(float(Lb));if(U7<=Ai){c Mb=U7*B+zi;Mb=U7*Mb+a7;float Di=dot(normalize(Mb),Kb);float Nb=U7*Bi+Ci;Nb=min(Nb,i4);if(Di>=cos(Nb)) w9=U7;}}float Ei=w9/Z2;float Ee=a2-w9;float x9=acos(clamp(Kb.x,-1.,1.));x9=Kb.y>=.0?x9:-x9;w1=Ee*D4+x9;c P2=c(sin(w1),-cos(w1));float k=dot(P2,B),y9=dot(P2,J),N1=dot(P2,q2);float Fi=max(y9*y9-k*N1,.0);float C2=sqrt(Fi);if(y9>.0) C2=-C2;C2-=y9;float Fe=-.5*C2*k;c Ob=(abs(C2*C2+Fe)<abs(k*N1+Fe))?c(C2,k):c(N1,C2);S5=(Ob.y!=.0)?Ob.x/Ob.y:.0;S5=clamp(S5,.0,1.);if(Ee==.0) S5=.0;B1=max(Ei,S5);}c Gi=j6(y0,C0,B1);c Ge=j6(C0,H0,B1);c Hi=j6(H0,P0,B1);c He=j6(Gi,Ge,B1);c Ie=j6(Ge,Hi,B1);R5=j6(He,Ie,B1);if(B1!=S5) w1=Yc(Ie-He);}N V7;V7.xy=floatBitsToUint(R5);if((i0&R3)==L8){V7.z=(uint(c5)<<16)|uint(a2);}else{uint Ii=uint(int(round(w1*(65536./F8))))&0xffffu;uint Je=0u;if((i0&R3)>r7){float Ji=clamp(w8(r2[0],r2[1]),-1.,1.);Je=uint(round(sqrt((1.+Ji)*.5)*65535.));}V7.z=(Ii<<16)|Je;}V7.w=i0;Q2(V7);}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive