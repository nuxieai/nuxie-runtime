#pragma once

#include "bezier_utils.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char bezier_utils[] = R"===(#ifndef gc
#define gc f
#endif
#ifndef O6
#define O6 c
#endif
e float g8(c l,c b){float vf=dot(l,b);float hc=dot(l,l)*dot(b,b);return(hc==.0)?1.:clamp(vf*inversesqrt(hc),-1.,1.);}e void wf(c x0,c B0,c F0,c M0,c1(c) B,c1(c) H,c1(c) l2){l2=B0-x0;c P6=F0-B0;c h8=M0-x0;H=P6-l2;B=-3.*P6+h8;}e Y W9(c x0,c B0,c F0,c M0){Y t;t[0]=(any(notEqual(x0,B0))?B0:any(notEqual(B0,F0))?F0:M0)-x0;t[1]=M0-(any(notEqual(M0,F0))?F0:any(notEqual(F0,B0))?B0:x0);return t;}e float xf(c x0,c B0,c F0,c M0,float v1,float yf){c B,H,l2;wf(x0,B0,F0,M0,B,H,l2);c Q6=3.*(((B*v1)+2.*H)*v1+l2);float ic=length(Q6);if(ic==.0){return.0;}Q6*=1./ic;float i8=2.*dot(B,Q6);float R6=3.*(i8*v1+4.*dot(H,Q6))*v1+6.*dot(l2,Q6);float X9=min(v1,1.-v1);float zf=(i8*X9*X9+R6)*X9;float jc=min(yf,zf*.9999);float d3;if(i8==.0){d3=jc/R6;}else{float M=1./i8;float b=R6*M,J1=-jc*M;float S6=(-1./3.)*b,T6=.5*J1;float kc=T6*T6-S6*S6*S6;if(kc<.0){float j8=sqrt(S6);float m1=acos(T6/(j8*j8*j8));d3=-2.*j8*cos(m1*(1./3.)+(-X3*2./3.));}else{float B=pow(abs(T6)+sqrt(kc),1./3.);if(T6<.0) B=-B;d3=B!=.0?B+S6/B:.0;}}d3=abs(d3);f t0011=v1+gc(-d3,-d3,d3,d3);f lc=(B.xyxy*t0011+2.*H.xyxy)*t0011+l2.xyxy;Y m2=W9(x0,B0,F0,M0);c Af=t0011.x<1e-3?m2[0]:lc.xy;c Bf=t0011.z>1.-1e-3?m2[1]:lc.zw;return acos(g8(Af,Bf));}e float k8(float l,float b){l=b<.0?-l:l;b=abs(b);return l>.0?(l<b?l/b:1.):.0;}float Cf(c x0,c B0,c F0,c M0,c1(float) Y9){c mc=M0-x0;float nc=length(M0-x0);if(nc==.0){Y9=.5;return.0;}c M2=O6(-mc.y,mc.x)/nc;float oc=dot(M2,F0-x0);float E4=dot(M2,B0-x0);float F4=E4-oc;
#if 0
float l=3.*F4;float pc=F4+E4;float J1=E4;float y2=sqrt(max(F4*F4+oc*E4,.0));if(pc<.0) y2=-y2;y2+=pc;c U6=O6(k8(y2,l),k8(J1,y2));c Y5=3.*(U6*(U6*(U6*F4-(E4+F4))+E4));Y5=abs(Y5);Y9=Y5.x>Y5.y?U6.x:U6.y;return max(Y5.x,Y5.y);
#else
float qc=3.*F4;float H=-E4-F4;float l2=E4;float t=.5;for(int H0=0;H0<3;++H0){float rc=qc*t;t=k8(rc*t-l2,2.*(rc+H));}Y9=t;return abs(t*(t*(t*qc+3.*H)+3.*l2));
#endif
}
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive