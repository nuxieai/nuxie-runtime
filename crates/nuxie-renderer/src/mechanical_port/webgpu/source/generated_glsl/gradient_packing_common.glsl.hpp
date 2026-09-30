#pragma once

#include "gradient_packing_common.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char gradient_packing_common[] = R"===(#ifdef BB
f e Y9(c l0,Y j9,c m2,float Lh,c ke,float y){e y2;y2.w=y;c le=K0(j9,l0)+m2;float Mh=ke.x;if(Mh>0.9){y2.z=2.0;}else{y2.z=ke.y;}if(Lh==float(uc)){y2.x=le.x;y2.y=0.0;}else{y2.z=-y2.z;y2.xy=le;}return y2;}
#endif
#ifdef EB
f c Cc(e y2){float t=y2.z>0.0?y2.x:length(y2.xy);t=clamp(t,0.0,1.0);float me=abs(y2.z);float x=me>1.0?(1.0-1.0/Ka)*t+(0.5/Ka):(1.0/Ka)*t+me;float Nh=y2.w;return c(x,Nh);}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive