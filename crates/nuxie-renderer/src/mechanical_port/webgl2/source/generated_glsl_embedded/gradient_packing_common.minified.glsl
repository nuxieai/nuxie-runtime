#ifdef BB
f e Rc(c l0,W y6,c L1,float l9,c H7,float y){e K0;K0.w=y;c Qe=y0(y6,l0)+L1;float Gi=H7.x;if(Gi>0.9){K0.z=2.0;}else{K0.z=H7.y;}if(l9==float(Ea)){K0.x=Qe.x;K0.y=0.0;}else{K0.z=-K0.z;K0.xy=Qe;}return K0;}
#endif
#ifdef EB
f c fd(e K0){float t=K0.z>0.0?K0.x:length(K0.xy);t=clamp(t,0.0,1.0);float Re=abs(K0.z);float x=Re>1.0?(1.0-1.0/h9)*t+(0.5/h9):(1.0/h9)*t+Re;float ub=K0.w;return c(x,ub);}
#endif
