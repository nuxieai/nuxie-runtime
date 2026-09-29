#pragma once

#include "bezier_utils.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char bezier_utils[] = R"===(#ifndef Wb
#define Wb g
#endif
#ifndef O6
#define O6 d
#endif
e float T9(d o,d b){float bf=dot(o,b);float Xb=dot(o,o)*dot(b,b);return(Xb==.0)?1.:clamp(bf*inversesqrt(Xb),-1.,1.);}e void cf(d r0,d z0,d D0,d K0,Z0(d)C,Z0(d)I,Z0(d)i2){i2=z0-r0;d P6=D0-z0;d i8=K0-r0;I=P6-i2;C=-3.*P6+i8;}e f0 U9(d r0,d z0,d D0,d K0){f0 t;t[0]=(any(notEqual(r0,z0))?z0:any(notEqual(z0,D0))?D0:K0)-r0;t[1]=K0-(any(notEqual(K0,D0))?D0:any(notEqual(D0,z0))?z0:r0);return t;}e float df(d r0,d z0,d D0,d K0,float r1,float ef){d C,I,i2;cf(r0,z0,D0,K0,C,I,i2);d Q6=3.*(((C*r1)+2.*I)*r1+i2);float Yb=length(Q6);if(Yb==.0){return.0;}Q6*=1./Yb;float j8=2.*dot(C,Q6);float R6=3.*(j8*r1+4.*dot(I,Q6))*r1+6.*dot(i2,Q6);float V9=min(r1,1.-r1);float ff=(j8*V9*V9+R6)*V9;float Zb=min(ef,ff*.9999);float X2;if(j8==.0){X2=Zb/R6;}else{float K=1./j8;float b=R6*K,H1=-Zb*K;float S6=(-1./3.)*b,T6=.5*H1;float ac=T6*T6-S6*S6*S6;if(ac<.0){float k8=sqrt(S6);float e1=acos(T6/(k8*k8*k8));X2=-2.*k8*cos(e1*(1./3.)+(-D3*2./3.));}else{float C=pow(abs(T6)+sqrt(ac),1./3.);if(T6<.0)C=-C;X2=C!=.0?C+S6/C:.0;}}X2=abs(X2);g t0011=r1+Wb(-X2,-X2,X2,X2);g bc=(C.xyxy*t0011+2.*I.xyxy)*t0011+i2.xyxy;f0 H2=U9(r0,z0,D0,K0);d gf=t0011.x<1e-3?H2[0]:bc.xy;d hf=t0011.z>1.-1e-3?H2[1]:bc.zw;return acos(T9(gf,hf));}e float l8(float o,float b){o=b<.0?-o:o;b=abs(b);return o>.0?(o<b?o/b:1.):.0;}float jf(d r0,d z0,d D0,d K0,Z0(float)W9){d cc=K0-r0;float dc=length(K0-r0);if(dc==.0){W9=.5;return.0;}d Y2=O6(-cc.y,cc.x)/dc;float ec=dot(Y2,D0-r0);float A4=dot(Y2,z0-r0);float B4=A4-ec;
#if 0
float o=3.*B4;float fc=B4+A4;float H1=A4;float r2=sqrt(max(B4*B4+ec*A4,.0));if(fc<.0)r2=-r2;r2+=fc;d U6=O6(l8(r2,o),l8(H1,r2));d Y5=3.*(U6*(U6*(U6*B4-(A4+B4))+A4));Y5=abs(Y5);W9=Y5.x>Y5.y?U6.x:U6.y;return max(Y5.x,Y5.y);
#else
float gc=3.*B4;float I=-A4-B4;float i2=A4;float t=.5;for(int F0=0;F0<3;++F0){float hc=gc*t;t=l8(hc*t-i2,2.*(hc+I));}W9=t;return abs(t*(t*(t*gc+3.*I)+3.*i2));
#endif
}
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive