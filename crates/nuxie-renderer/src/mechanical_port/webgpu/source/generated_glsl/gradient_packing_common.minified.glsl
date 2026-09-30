#ifdef VERTEX
e f Pb(c v0,e0 T8,c I2,float rh,c Wd,float y){f p2;p2.w=y;c Xd=P0(T8,v0)+I2;float sh=Wd.x;if(sh>0.9){p2.z=2.0;}else{p2.z=Wd.y;}if(rh==float(Yb)){p2.x=Xd.x;p2.y=0.0;}else{p2.z=-p2.z;p2.xy=Xd;}return p2;}
#endif
#ifdef FRAGMENT
e c gc(f p2){float t=p2.z>0.0?p2.x:length(p2.xy);t=clamp(t,0.0,1.0);float Yd=abs(p2.z);float x=Yd>1.0?(1.0-1.0/qa)*t+(0.5/qa):(1.0/qa)*t+Yd;float th=p2.w;return c(x,th);}
#endif
