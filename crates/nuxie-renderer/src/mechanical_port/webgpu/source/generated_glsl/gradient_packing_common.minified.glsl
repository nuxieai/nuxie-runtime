#define hc 8.0
#ifdef VERTEX
e f Fa(c l0,X U9,c y2,c n8,uint l5,float o4,float n){f R1;R1.xy=C0(U9,l0)+y2;bool n4=(n8.x<0.0);n8.x=max(n8.x,0.0);R1.z=n8.y+(float(l5)/hc)+(n8.x*(a5/hc));R1.z=(n4)?-R1.z:R1.z;R1.w=(n*-0.5-0.25)-round(o4*255.0);return R1;}
#endif
#ifdef FRAGMENT
e c Ve(float l5,c Z6,c a3,bool n4){if(l5==float(wh)){float t=gb(c(Z6.x,-Z6.y))/(3.14159*2.0);return c(t,a3.y);}else{float t=(l5==float(vh))?Z6.x:length(Z6.xy);t=clamp(t,0.0,1.0);float Li=n4?(1.0-a5):a5;float x=t*Li+a3.x;return c(x,a3.y);}}e c tg(float l5,c Z6,c We,bool n4,float ic,float jc){const float kc=0.5*a5;c a3=c(We.x*a5+kc,We.y*ic+jc);return Ve(l5,Z6,a3,n4);}e c Va(f R1,float ic,float jc){const float kc=0.5*a5;bool n4=R1.z<0.0;R1.z=abs(R1.z);c a3;a3.y=floor(R1.z);a3.x=(R1.z-a3.y)*hc+kc;a3.y=a3.y*ic+jc;float l5=floor(a3.x);return Ve(l5,R1.xy,a3,n4);}e d gi(f R1){return fract(R1.w)*-2.0+1.5;}e d me(f R1){const float Mi=-1.0/255.0;return ceil(R1.w)*Mi;}
#endif
