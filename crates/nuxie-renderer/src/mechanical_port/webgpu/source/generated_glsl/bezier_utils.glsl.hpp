#pragma once

#include "bezier_utils.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char bezier_utils[] = R"===(#ifndef pd
#define pd e
#endif
#ifndef p7
#define p7 c
#endif
f float X8(c l,c b){float Ag=dot(l,b);float qd=dot(l,l)*dot(b,b);return(qd==.0)?1.:clamp(Ag*inversesqrt(qd),-1.,1.);}f void Bg(c A0,c D0,c L0,c T0,l1(c) B,l1(c) J,l1(c) z2){z2=D0-A0;c q7=L0-D0;c Y8=T0-A0;J=q7-z2;B=-3.*q7+Y8;}f X Xa(c A0,c D0,c L0,c T0){X t;t[0]=(any(notEqual(A0,D0))?D0:any(notEqual(D0,L0))?L0:T0)-A0;t[1]=T0-(any(notEqual(T0,L0))?L0:any(notEqual(L0,D0))?D0:A0);return t;}f float Cg(c A0,c D0,c L0,c T0,float J1,float Dg){c B,J,z2;Bg(A0,D0,L0,T0,B,J,z2);c r7=3.*(((B*J1)+2.*J)*J1+z2);float rd=length(r7);if(rd==.0){return.0;}r7*=1./rd;float Z8=2.*dot(B,r7);float v7=3.*(Z8*J1+4.*dot(J,r7))*J1+6.*dot(z2,r7);float Ya=min(J1,1.-J1);float Eg=(Z8*Ya*Ya+v7)*Ya;float sd=min(Dg,Eg*.9999);float p3;if(Z8==.0){p3=sd/v7;}else{float S=1./Z8;float b=v7*S,W1=-sd*S;float w7=(-1./3.)*b,x7=.5*W1;float td=x7*x7-w7*w7*w7;if(td<.0){float a9=sqrt(w7);float g1=acos(x7/(a9*a9*a9));p3=-2.*a9*cos(g1*(1./3.)+(-r4*2./3.));}else{float B=pow(abs(x7)+sqrt(td),1./3.);if(x7<.0) B=-B;p3=B!=.0?B+w7/B:.0;}}p3=abs(p3);e t0011=J1+pd(-p3,-p3,p3,p3);e ud=(B.xyxy*t0011+2.*J.xyxy)*t0011+z2.xyxy;X A2=Xa(A0,D0,L0,T0);c Fg=t0011.x<1e-3?A2[0]:ud.xy;c Gg=t0011.z>1.-1e-3?A2[1]:ud.zw;return acos(X8(Fg,Gg));}f float c9(float l,float b){l=b<.0?-l:l;b=abs(b);return l>.0?(l<b?l/b:1.):.0;}float Hg(c A0,c D0,c L0,c T0,l1(float) Za){c vd=T0-A0;float wd=length(T0-A0);if(wd==.0){Za=.5;return.0;}c S1=p7(-vd.y,vd.x)/wd;float xd=dot(S1,L0-A0);float Y4=dot(S1,D0-A0);float Z4=Y4-xd;
#if 0
float l=3.*Z4;float yd=Z4+Y4;float W1=Y4;float K2=sqrt(max(Z4*Z4+xd*Y4,.0));if(yd<.0) K2=-K2;K2+=yd;c y7=p7(c9(K2,l),c9(W1,K2));c w6=3.*(y7*(y7*(y7*Z4-(Y4+Z4))+Y4));w6=abs(w6);Za=w6.x>w6.y?y7.x:y7.y;return max(w6.x,w6.y);
#else
float zd=3.*Z4;float J=-Y4-Z4;float z2=Y4;float t=.5;for(int N0=0;N0<3;++N0){float Ad=zd*t;t=c9(Ad*t-z2,2.*(Ad+J));}Za=t;return abs(t*(t*(t*zd+3.*J)+3.*z2));
#endif
}
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive