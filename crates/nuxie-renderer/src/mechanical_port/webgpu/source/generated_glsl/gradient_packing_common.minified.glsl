#ifdef VERTEX
e f Tb(c v0,e0 W8,c I2,float th,c ce,float y){f p2;p2.w=y;c de=P0(W8,v0)+I2;float uh=ce.x;if(uh>0.9){p2.z=2.0;}else{p2.z=ce.y;}if(th==float(cc)){p2.x=de.x;p2.y=0.0;}else{p2.z=-p2.z;p2.xy=de;}return p2;}
#endif
#ifdef FRAGMENT
e c kc(f p2){float t=p2.z>0.0?p2.x:length(p2.xy);t=clamp(t,0.0,1.0);float ee=abs(p2.z);float x=ee>1.0?(1.0-1.0/ua)*t+(0.5/ua):(1.0/ua)*t+ee;float vh=p2.w;return c(x,vh);}
#endif
