#pragma once

#include "bezier_utils.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char bezier_utils[] = R"===(#ifndef lc
#define lc f
#endif
#ifndef Q6
#define Q6 c
#endif
e float Y9(c l,c b){float Bf=dot(l,b);float mc=dot(l,l)*dot(b,b);return(mc==.0)?1.:clamp(Bf*inversesqrt(mc),-1.,1.);}e void Cf(c x0,c B0,c F0,c M0,c1(c)C,c1(c)H,c1(c)l2){l2=B0-x0;c R6=F0-B0;c k8=M0-x0;H=R6-l2;C=-3.*R6+k8;}e e0 Z9(c x0,c B0,c F0,c M0){e0 t;t[0]=(any(notEqual(x0,B0))?B0:any(notEqual(B0,F0))?F0:M0)-x0;t[1]=M0-(any(notEqual(M0,F0))?F0:any(notEqual(F0,B0))?B0:x0);return t;}e float Df(c x0,c B0,c F0,c M0,float w1,float Ef){c C,H,l2;Cf(x0,B0,F0,M0,C,H,l2);c S6=3.*(((C*w1)+2.*H)*w1+l2);float nc=length(S6);if(nc==.0){return.0;}S6*=1./nc;float l8=2.*dot(C,S6);float T6=3.*(l8*w1+4.*dot(H,S6))*w1+6.*dot(l2,S6);float aa=min(w1,1.-w1);float Ff=(l8*aa*aa+T6)*aa;float oc=min(Ef,Ff*.9999);float c3;if(l8==.0){c3=oc/T6;}else{float M=1./l8;float b=T6*M,J1=-oc*M;float U6=(-1./3.)*b,V6=.5*J1;float pc=V6*V6-U6*U6*U6;if(pc<.0){float m8=sqrt(U6);float f1=acos(V6/(m8*m8*m8));c3=-2.*m8*cos(f1*(1./3.)+(-H3*2./3.));}else{float C=pow(abs(V6)+sqrt(pc),1./3.);if(V6<.0)C=-C;c3=C!=.0?C+U6/C:.0;}}c3=abs(c3);f t0011=w1+lc(-c3,-c3,c3,c3);f qc=(C.xyxy*t0011+2.*H.xyxy)*t0011+l2.xyxy;e0 L2=Z9(x0,B0,F0,M0);c Gf=t0011.x<1e-3?L2[0]:qc.xy;c Hf=t0011.z>1.-1e-3?L2[1]:qc.zw;return acos(Y9(Gf,Hf));}e float n8(float l,float b){l=b<.0?-l:l;b=abs(b);return l>.0?(l<b?l/b:1.):.0;}float If(c x0,c B0,c F0,c M0,c1(float)ba){c rc=M0-x0;float sc=length(M0-x0);if(sc==.0){ba=.5;return.0;}c d3=Q6(-rc.y,rc.x)/sc;float tc=dot(d3,F0-x0);float D4=dot(d3,B0-x0);float E4=D4-tc;
#if 0
float l=3.*E4;float uc=E4+D4;float J1=D4;float x2=sqrt(max(E4*E4+tc*D4,.0));if(uc<.0)x2=-x2;x2+=uc;c W6=Q6(n8(x2,l),n8(J1,x2));c c6=3.*(W6*(W6*(W6*E4-(D4+E4))+D4));c6=abs(c6);ba=c6.x>c6.y?W6.x:W6.y;return max(c6.x,c6.y);
#else
float vc=3.*E4;float H=-D4-E4;float l2=D4;float t=.5;for(int H0=0;H0<3;++H0){float wc=vc*t;t=n8(wc*t-l2,2.*(wc+H));}ba=t;return abs(t*(t*(t*vc+3.*H)+3.*l2));
#endif
}
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive