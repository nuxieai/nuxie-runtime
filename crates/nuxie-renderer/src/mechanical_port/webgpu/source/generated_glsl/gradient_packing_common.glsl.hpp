#pragma once

#include "gradient_packing_common.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char gradient_packing_common[] = R"===(#ifdef DB
e f Pb(c q0,d0 U8,c I2,float lh,c Vd,float y){f o2;o2.w=y;c Wd=N0(U8,q0)+I2;float mh=Vd.x;if(mh>0.9){o2.z=2.0;}else{o2.z=Vd.y;}if(lh==float(Yb)){o2.x=Wd.x;o2.y=0.0;}else{o2.z=-o2.z;o2.xy=Wd;}return o2;}
#endif
#ifdef FB
e c dc(f o2){float t=o2.z>0.0?o2.x:length(o2.xy);t=clamp(t,0.0,1.0);float Xd=abs(o2.z);float x=Xd>1.0?(1.0-1.0/ra)*t+(0.5/ra):(1.0/ra)*t+Xd;float nh=o2.w;return c(x,nh);}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive