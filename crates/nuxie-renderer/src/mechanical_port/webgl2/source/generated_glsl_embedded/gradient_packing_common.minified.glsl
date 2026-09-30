#ifdef CB
e f Ob(c v0,Y T8,c J2,float qh,c Vd,float y){f q2;q2.w=y;c Wd=N0(T8,v0)+J2;float rh=Vd.x;if(rh>0.9){q2.z=2.0;}else{q2.z=Vd.y;}if(qh==float(Xb)){q2.x=Wd.x;q2.y=0.0;}else{q2.z=-q2.z;q2.xy=Wd;}return q2;}
#endif
#ifdef EB
e c fc(f q2){float t=q2.z>0.0?q2.x:length(q2.xy);t=clamp(t,0.0,1.0);float Xd=abs(q2.z);float x=Xd>1.0?(1.0-1.0/pa)*t+(0.5/pa):(1.0/pa)*t+Xd;float sh=q2.w;return c(x,sh);}
#endif
