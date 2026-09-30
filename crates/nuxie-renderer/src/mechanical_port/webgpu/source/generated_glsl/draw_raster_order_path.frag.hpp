#pragma once

#include "draw_raster_order_path.frag.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_raster_order_path_frag[] = R"===(#ifdef EB
M1 z0(G2,m0);k1(X2,i0);z0(j6,p4);k1(M6,J7);N1 P1(HB){r(X1,f);
#ifdef IB
r(C2,R);
#endif
#ifdef DB
r(j1,d);
#else
r(O,B2);
#endif
r(D0,d);
#ifdef I
r(Y1,E);
#endif
#ifdef AB
r(O0,f);
#endif
#ifdef S
r(g1,d);
#endif
#if!defined(DB)
z2;
#endif
E V4=unpackHalf2x16(a1(J7));d m9=V4.y;d r0=m9==D0?V4.x:J0(.0);
#ifdef DB
r0+=j1;h2(J7);
#else
r0=wi(r0,O e1);d1(J7,packHalf2x16(D2(r0,D0)));
#endif
d o;
#ifdef GE
if(GE){o=ka(r0,J0(.0),J0(1.));}else
#endif
{o=abs(r0);
#ifdef WC
if(WC&&D0<.0){o=1.-J0(abs(fract(o*.5)*2.+-1.));}
#endif
o=min(o,J0(1.));}
#ifdef I
if(I&&Y1.x<.0){d m1=-Y1.x;
#ifdef YC
if(YC){d M5=Y1.y;if(M5!=.0){E Q0=unpackHalf2x16(a1(i0));d H6=Q0.y;d w4;if(H6!=m1){w4=H6==M5?Q0.x:.0;
#ifndef DB
A0(p4,E0(w4,.0,.0,.0));
#endif
}else{w4=K0(p4).x;
#ifndef DB
y2(p4);
#endif
}o=min(o,w4);}}
#endif
d1(i0,packHalf2x16(D2(o,m1)));y2(m0);}else
#endif
{
#ifdef I
if(I){d m1=Y1.x;if(m1!=.0){E Q0=unpackHalf2x16(a1(i0));d H6=Q0.y;o=(H6==m1)?min(Q0.x,o):J0(.0);}}
#endif
#ifdef AB
if(AB){d c5=m3(g5(O0));o=clamp(c5,J0(.0),o);}
#endif
i k=O7(
#ifdef IB
C2,
#endif
#ifdef S
g3(g1),
#endif
X1 Y2);i O1;if(m9!=D0){O1=K0(m0);
#ifndef DB
A0(p4,O1);
#endif
}else{O1=K0(p4);
#ifndef DB
y2(p4);
#endif
}
#ifdef S
if(S&&g1!=e6(B4)){k.xyz=Z4(k.xyz,O1,g3(g1))*k.w;}
#endif
k*=o;
#ifdef AC
if(AC){k=q3(k);}
#endif
d j3=k.w;k+=O1*(1.-j3);k.xyz=K2(k.xyz,j3,d0.xy,j.F3,j.G3);A0(m0,k);h2(i0);}
#if!defined(DB)
A2;
#endif
d2;}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive