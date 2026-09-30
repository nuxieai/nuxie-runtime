#ifdef BB
e f Z9(c l0,Y l9,c m2,float Vh,c le,float y){f y2;y2.w=y;c me=M0(l9,l0)+m2;float Wh=le.x;if(Wh>0.9){y2.z=2.0;}else{y2.z=le.y;}if(Vh==float(vc)){y2.x=me.x;y2.y=0.0;}else{y2.z=-y2.z;y2.xy=me;}return y2;}
#endif
#ifdef EB
e c Dc(f y2){float t=y2.z>0.0?y2.x:length(y2.xy);t=clamp(t,0.0,1.0);float ne=abs(y2.z);float x=ne>1.0?(1.0-1.0/La)*t+(0.5/La):(1.0/La)*t+ne;float Xh=y2.w;return c(x,Xh);}
#endif
