#pragma once

#include "draw_raster_order_path.frag.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_raster_order_path_frag[] = R"===(#ifdef GB
J1 x0(S2,j0);j1(T2,h0);x0(h6,l4);j1(L6,I7);K1 M1(JB){r(f1,g);
#ifdef KB
r(A2,R);
#endif
#ifdef EB
r(i1,c);
#else
r(O,z2);
#endif
r(B0,c);
#ifdef J
r(V1,E);
#endif
#ifdef BB
r(M0,g);
#endif
#ifdef AB
r(f2,c);
#endif
#if!defined(EB)
x2;
#endif
E S4=unpackHalf2x16(Y0(I7));c j9=S4.y;c p0=j9==B0?S4.x:G0(.0);
#ifdef EB
p0+=i1;e2(I7);
#else
p0=Xh(p0,O d1);c1(I7,packHalf2x16(B2(p0,B0)));
#endif
c n;
#ifdef EE
if(EE){n=ea(p0,G0(.0),G0(1.));}else
#endif
{n=abs(p0);
#ifdef XC
if(XC&&B0<.0){n=1.-G0(abs(fract(n*.5)*2.+-1.));}
#endif
n=min(n,G0(1.));}
#ifdef J
if(J&&V1.x<.0){c k1=-V1.x;
#ifdef ZC
if(ZC){c J5=V1.y;if(J5!=.0){E N0=unpackHalf2x16(Y0(h0));c G6=N0.y;c p4;if(G6!=k1){p4=G6==J5?N0.x:.0;
#ifndef EB
y0(l4,C0(p4,.0,.0,.0));
#endif
}else{p4=I0(l4).x;
#ifndef EB
w2(l4);
#endif
}n=min(n,p4);}}
#endif
c1(h0,packHalf2x16(B2(n,k1)));w2(j0);}else
#endif
{
#ifdef J
if(J){c k1=V1.x;if(k1!=.0){E N0=unpackHalf2x16(Y0(h0));c G6=N0.y;n=(G6==k1)?min(N0.x,n):G0(.0);}}
#endif
#ifdef BB
if(BB){c Z4=h3(d5(M0));n=clamp(Z4,G0(.0),n);}
#endif
i j=N7(f1,
#ifdef KB
A2,
#endif
n U2);i L1;if(j9!=B0){L1=I0(j0);
#ifndef EB
y0(l4,L1);
#endif
}else{L1=I0(l4);
#ifndef EB
w2(l4);
#endif
}
#ifdef AB
if(AB){if(f2!=a6(R5)){j.xyz=V4(j.xyz,L1,c6(f2));}j.xyz*=j.w;}
#endif
#ifdef CC
if(CC){j=m3(j);}
#endif
c v2=j.w;j+=L1*(1.-v2);j.xyz=F2(j.xyz,v2,a0.xy,m.B3,m.C3);y0(j0,j);e2(h0);}
#if!defined(EB)
y2;
#endif
Z1;}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive