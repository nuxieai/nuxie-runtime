#ifdef CB
e f Sb(c v0,e0 W8,c I2,float sh,c be,float y){f p2;p2.w=y;c ce=P0(W8,v0)+I2;float th=be.x;if(th>0.9){p2.z=2.0;}else{p2.z=be.y;}if(sh==float(bc)){p2.x=ce.x;p2.y=0.0;}else{p2.z=-p2.z;p2.xy=ce;}return p2;}
#endif
#ifdef EB
e c jc(f p2){float t=p2.z>0.0?p2.x:length(p2.xy);t=clamp(t,0.0,1.0);float de=abs(p2.z);float x=de>1.0?(1.0-1.0/ua)*t+(0.5/ua):(1.0/ua)*t+de;float uh=p2.w;return c(x,uh);}
#endif
