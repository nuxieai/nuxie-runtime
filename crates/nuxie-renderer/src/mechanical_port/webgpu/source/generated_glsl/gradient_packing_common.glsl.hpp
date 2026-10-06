#pragma once

#include "gradient_packing_common.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char gradient_packing_common[] = R"===(#ifdef BB
f e Z9(c l0,Y k9,c m2,float Wh,c le,float y){e x2;x2.w=y;c me=M0(k9,l0)+m2;float Xh=le.x;if(Xh>0.9){x2.z=2.0;}else{x2.z=le.y;}if(Wh==float(vc)){x2.x=me.x;x2.y=0.0;}else{x2.z=-x2.z;x2.xy=me;}return x2;}
#endif
#ifdef FB
f c Dc(e x2){float t=x2.z>0.0?x2.x:length(x2.xy);t=clamp(t,0.0,1.0);float ne=abs(x2.z);float x=ne>1.0?(1.0-1.0/La)*t+(0.5/La):(1.0/La)*t+ne;float Yh=x2.w;return c(x,Yh);}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive