#pragma once

#include "bezier_utils.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char bezier_utils[] = R"===(#ifndef sd
#define sd e
#endif
#ifndef p7
#define p7 c
#endif
f float Z8(c l,c b){float Gg=dot(l,b);float td=dot(l,l)*dot(b,b);return(td==.0)?1.:clamp(Gg*inversesqrt(td),-1.,1.);}f void Hg(c A0,c F0,c M0,c U0,l1(c) B,l1(c) J,l1(c) z2){z2=F0-A0;c q7=M0-F0;c a9=U0-A0;J=q7-z2;B=-3.*q7+a9;}f X Ya(c A0,c F0,c M0,c U0){X t;t[0]=(any(notEqual(A0,F0))?F0:any(notEqual(F0,M0))?M0:U0)-A0;t[1]=U0-(any(notEqual(U0,M0))?M0:any(notEqual(M0,F0))?F0:A0);return t;}f float Ig(c A0,c F0,c M0,c U0,float J1,float Jg){c B,J,z2;Hg(A0,F0,M0,U0,B,J,z2);c r7=3.*(((B*J1)+2.*J)*J1+z2);float ud=length(r7);if(ud==.0){return.0;}r7*=1./ud;float c9=2.*dot(B,r7);float v7=3.*(c9*J1+4.*dot(J,r7))*J1+6.*dot(z2,r7);float Za=min(J1,1.-J1);float Kg=(c9*Za*Za+v7)*Za;float vd=min(Jg,Kg*.9999);float p3;if(c9==.0){p3=vd/v7;}else{float S=1./c9;float b=v7*S,W1=-vd*S;float w7=(-1./3.)*b,x7=.5*W1;float wd=x7*x7-w7*w7*w7;if(wd<.0){float d9=sqrt(w7);float g1=acos(x7/(d9*d9*d9));p3=-2.*d9*cos(g1*(1./3.)+(-w4*2./3.));}else{float B=pow(abs(x7)+sqrt(wd),1./3.);if(x7<.0) B=-B;p3=B!=.0?B+w7/B:.0;}}p3=abs(p3);e t0011=J1+sd(-p3,-p3,p3,p3);e xd=(B.xyxy*t0011+2.*J.xyxy)*t0011+z2.xyxy;X A2=Ya(A0,F0,M0,U0);c Lg=t0011.x<1e-3?A2[0]:xd.xy;c Mg=t0011.z>1.-1e-3?A2[1]:xd.zw;return acos(Z8(Lg,Mg));}f float e9(float l,float b){l=b<.0?-l:l;b=abs(b);return l>.0?(l<b?l/b:1.):.0;}float Ng(c A0,c F0,c M0,c U0,l1(float) ab){c yd=U0-A0;float zd=length(U0-A0);if(zd==.0){ab=.5;return.0;}c S1=p7(-yd.y,yd.x)/zd;float Ad=dot(S1,M0-A0);float Z4=dot(S1,F0-A0);float a5=Z4-Ad;
#if 0
float l=3.*a5;float Bd=a5+Z4;float W1=Z4;float K2=sqrt(max(a5*a5+Ad*Z4,.0));if(Bd<.0) K2=-K2;K2+=Bd;c y7=p7(e9(K2,l),e9(W1,K2));c w6=3.*(y7*(y7*(y7*a5-(Z4+a5))+Z4));w6=abs(w6);ab=w6.x>w6.y?y7.x:y7.y;return max(w6.x,w6.y);
#else
float Cd=3.*a5;float J=-Z4-a5;float z2=Z4;float t=.5;for(int Q0=0;Q0<3;++Q0){float Dd=Cd*t;t=e9(Dd*t-z2,2.*(Dd+J));}ab=t;return abs(t*(t*(t*Cd+3.*J)+3.*z2));
#endif
}
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive