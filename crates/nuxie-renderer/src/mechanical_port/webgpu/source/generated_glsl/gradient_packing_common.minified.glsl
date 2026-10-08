#ifdef VERTEX
f e Tc(c l0,W y6,c L1,float n9,c H7,float y){e L0;L0.w=y;c Te=y0(y6,l0)+L1;float Ki=H7.x;if(Ki>0.9){L0.z=2.0;}else{L0.z=H7.y;}if(n9==float(Ga)){L0.x=Te.x;L0.y=0.0;}else{L0.z=-L0.z;L0.xy=Te;}return L0;}
#endif
#ifdef FRAGMENT
f c hd(e L0){float t=L0.z>0.0?L0.x:length(L0.xy);t=clamp(t,0.0,1.0);float Ue=abs(L0.z);float x=Ue>1.0?(1.0-1.0/i9)*t+(0.5/i9):(1.0/i9)*t+Ue;float wb=L0.w;return c(x,wb);}
#endif
