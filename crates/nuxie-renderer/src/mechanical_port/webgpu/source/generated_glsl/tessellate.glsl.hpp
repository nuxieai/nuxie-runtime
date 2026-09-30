#pragma once

#include "tessellate.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char tessellate[] = R"===(#define Zh 10
#ifdef CB
h1(g0)K(0,f,LD);K(1,f,MD);K(2,f,UC);
#ifdef ca
K(3,uint,HE);K(4,uint,IE);K(5,uint,JE);K(6,uint,KE);
#else
K(3,Y,TB);
#endif
i1
#endif
q2 I0 W(0,f,D6);I0 W(1,f,E6);I0 W(2,f,S4);I0 W(3,R,T4);V2 W(4,uint,K7);i2
#ifdef CB
Y3 l6(h3,k7,XC);Z3 g4(k7,ha)F4 N4(dd,Jg,OB);N4(ed,Kg,HD);G4 B1(FG,g0,F,B,v){L(v,F,LD,f);L(v,F,MD,f);L(v,F,UC,f);
#ifdef ca
L(v,F,HE,uint);L(v,F,IE,uint);L(v,F,JE,uint);L(v,F,KE,uint);Y TB=Y(HE,IE,JE,KE);
#else
L(v,F,TB,Y);
#endif
U(D6,f);U(E6,f);U(S4,f);U(T4,R);U(K7,uint);c x0=LD.xy;c B0=LD.zw;c F0=MD.xy;c M0=MD.zw;bool le=B<4;float y=le?UC.z:UC.w;int mb=int(le?TB.x:TB.y);
#ifdef Fc
int me=mb<<16;if(TB.z==0xffffffffu){--me;}float e9=float(me>>16);
#else
float e9=float(mb<<16>>16);
#endif
float f9=float(mb>>16);c r2=c((B&1)==0?e9:f9,(B&2)==0?y+1.:y);if((f9-e9)*j.Hd<.0){r2.y=2.*y+1.-r2.y;}uint U2=TB.z&0x3ffu;uint ne=(TB.z>>10)&0x3ffu;uint n2=TB.z>>20;uint j0=TB.w;uint I8=j0&Zc;uint o0=I8>0u?L0(HD,max(I8,1u)-1u).z:0u;Y P4=o0!=0u?L0(OB,o0*4u+1u):Y(0u,0u,0u,0u);float N2=uintBitsToFloat(P4.z);float O2=uintBitsToFloat(P4.w);if(O2!=.0&&N2==.0){float oe;float ai=Hf(x0,B0,F0,M0,oe);float nb=O2*(1./va);float bi=Cf(x0,B0,F0,M0,oe,nb);float L7=1.-bi*(1./H3);float ci=dot(M0-x0,M0-x0)/(nb*nb);float di=(ci-1.)*.5;L7=min(L7,di);L7=min(L7,.99);float ei=.5*L7;float x=Ec(ei)*-2.+1.;float pe=n8(x*O2,ai);f qe=mix(x0.xyxy,M0.xyxy,f(1./3.,1./3.,2./3.,2./3.));B0=mix(B0,qe.xy,pe);F0=mix(F0,qe.zw,pe);}if((j0&fg)!=0u){e0 W8=L1(uintBitsToFloat(L0(OB,o0*4u)));c re=P0(W8,-2.*B0+F0+x0);c se=P0(W8,-2.*F0+M0+B0);float n1=max(dot(re,re),dot(se,se));float T3=max(ceil(sqrt(.75*4.*sqrt(n1))),1.);U2=min(uint(T3),U2);}uint g9=U2+ne+n2-1u;e0 L2=Z9(x0,B0,F0,M0);float f1=acos(Y9(L2[0],L2[1]));float r4=f1/float(ne);float ob=determinant(e0(F0-x0,M0-B0));if(ob==.0)ob=determinant(L2);if(ob<.0)r4=-r4;D6=f(x0,B0);E6=f(F0,M0);S4=f(float(g9)-abs(f9-r2.x),float(g9),(n2<<10)|U2,r4);T4.xy=UC.xy;if(n2>1u){e0 pb=e0(L2[1],UC.xy);float fi=acos(Y9(pb[0],pb[1]));float te=float(n2);if((j0&(f4|C8))==(A8|C8)){te-=2.;}float qb=fi/te;if(determinant(pb)<.0)qb=-qb;T4.z=qb;}if(f9<e9){j0|=K3;}K7=j0;f X=r8(r2,2./cg,j.Hd);
#ifdef RC
X.y=-X.y;
#endif
c0(D6);c0(E6);c0(S4);c0(T4);c0(K7);C1(X);}
#endif
#ifdef EB
I3 J3 f3(H4,GG){r(D6,f);r(E6,f);r(S4,f);r(T4,R);r(K7,uint);c x0=D6.xy;c B0=D6.zw;c F0=E6.xy;c M0=E6.zw;e0 L2=Z9(x0,B0,F0,M0);float gi=max(floor(S4.x),.0);float g9=S4.y;uint ue=uint(S4.z);float U2=float(ue&0x3ffu);float n2=float(ue>>10);float r4=S4.w;uint j0=K7;float U4=g9-n2;float W1=gi;if(W1<=U4){j0&=~f4;}else{x0=B0=F0=M0;L2=e0(L2[1],T4.xy);U2=1.;W1-=U4;U4=n2;r4=T4.z;if((j0&f4)>A8){if(W1<2.5)j0|=wa;if(W1>1.5&&W1<3.5)j0|=Xc;}else if((j0&C8)!=0u||(j0&f4)==B8){U4-=2.;--W1;}j0|=r4<.0?D8:Yc;}c J5;float f1=.0;if(W1==.0||W1==U4||(j0&f4)>A8){bool L8=W1<U4*.5;J5=L8?x0:M0;f1=Hc(L8?L2[0]:L2[1]);}else if((j0&Wc)!=0u){J5=x0;if(W1>=float(ta/2u))J5=B0;if(W1>=float(ta*3u/4u))J5=F0;if(W1>=float(ta*7u/8u))J5=T4.xy;}else{float w1,K5;if(U2==U4){w1=W1/U2;K5=.0;}else{c C,H,l2=B0-x0;c R6=M0-x0;c k8=F0-B0;H=k8-l2;C=-3.*k8+R6;c hi=H*(U2*2.);c T6=l2*(U2*U2);float h9=.0;float ii=min(U2-1.,W1);c rb=normalize(L2[0]);float ji=-abs(r4);float ki=(1.+W1)*abs(r4);for(int sb=Zh-1;sb>=0;--sb){float M7=h9+exp2(float(sb));if(M7<=ii){c tb=M7*C+hi;tb=M7*tb+T6;float li=dot(normalize(tb),rb);float ub=M7*ji+ki;ub=min(ub,H3);if(li>=cos(ub))h9=M7;}}float mi=h9/U2;float ve=W1-h9;float i9=acos(clamp(rb.x,-1.,1.));i9=rb.y>=.0?i9:-i9;f1=ve*r4+i9;c d3=c(sin(f1),-cos(f1));float l=dot(d3,C),j9=dot(d3,H),J1=dot(d3,l2);float ni=max(j9*j9-l*J1,.0);float x2=sqrt(ni);if(j9>.0)x2=-x2;x2-=j9;float we=-.5*x2*l;c vb=(abs(x2*x2+we)<abs(l*J1+we))?c(x2,l):c(J1,x2);K5=(vb.y!=.0)?vb.x/vb.y:.0;K5=clamp(K5,.0,1.);if(ve==.0)K5=.0;w1=max(mi,K5);}c oi=f6(x0,B0,w1);c xe=f6(B0,F0,w1);c pi=f6(F0,M0,w1);c ye=f6(oi,xe,w1);c ze=f6(xe,pi,w1);J5=f6(ye,ze,w1);if(w1!=K5)f1=Hc(ze-ye);}H4 N7;N7.xy=fa(J5);if((j0&f4)==B8){N7.z=ga((uint(U4)<<16)|uint(W1));}else{N7.z=fa(mod(f1,v8));}N7.w=ga(j0);M2(N7);}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive