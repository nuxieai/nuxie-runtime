#ifdef DB
e f Ob(c q0,d0 U8,c I2,float kh,c Ud,float y){f o2;o2.w=y;c Vd=N0(U8,q0)+I2;float lh=Ud.x;if(lh>0.9){o2.z=2.0;}else{o2.z=Ud.y;}if(kh==float(Xb)){o2.x=Vd.x;o2.y=0.0;}else{o2.z=-o2.z;o2.xy=Vd;}return o2;}
#endif
#ifdef FB
e c cc(f o2){float t=o2.z>0.0?o2.x:length(o2.xy);t=clamp(t,0.0,1.0);float Wd=abs(o2.z);float x=Wd>1.0?(1.0-1.0/ra)*t+(0.5/ra):(1.0/ra)*t+Wd;float mh=o2.w;return c(x,mh);}
#endif
