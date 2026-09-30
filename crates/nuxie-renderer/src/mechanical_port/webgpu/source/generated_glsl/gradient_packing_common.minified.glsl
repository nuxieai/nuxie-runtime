#ifdef VERTEX
e f Qb(c r0,d0 U8,c H2,float mh,c Wd,float y){f o2;o2.w=y;c Xd=N0(U8,r0)+H2;float nh=Wd.x;if(nh>0.9){o2.z=2.0;}else{o2.z=Wd.y;}if(mh==float(Zb)){o2.x=Xd.x;o2.y=0.0;}else{o2.z=-o2.z;o2.xy=Xd;}return o2;}
#endif
#ifdef FRAGMENT
e c ec(f o2){float t=o2.z>0.0?o2.x:length(o2.xy);t=clamp(t,0.0,1.0);float Yd=abs(o2.z);float x=Yd>1.0?(1.0-1.0/sa)*t+(0.5/sa):(1.0/sa)*t+Yd;float oh=o2.w;return c(x,oh);}
#endif
