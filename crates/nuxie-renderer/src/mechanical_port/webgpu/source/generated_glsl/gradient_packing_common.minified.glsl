#define hc 8.0
#ifdef VERTEX
e f Da(c l0,X S9,c x2,c l8,uint l5,float o4,float n){f P1;P1.xy=B0(S9,l0)+x2;bool n4=(l8.x<0.0);l8.x=max(l8.x,0.0);P1.z=l8.y+(float(l5)/hc)+(l8.x*(a5/hc));P1.z=(n4)?-P1.z:P1.z;P1.w=(n*-0.5-0.25)-round(o4*255.0);return P1;}
#endif
#ifdef FRAGMENT
e c Te(float l5,c W6,c a3,bool n4){if(l5==float(th)){float t=db(c(W6.x,-W6.y))/(3.14159*2.0);return c(t,a3.y);}else{float t=(l5==float(sh))?W6.x:length(W6.xy);t=clamp(t,0.0,1.0);float Ii=n4?(1.0-a5):a5;float x=t*Ii+a3.x;return c(x,a3.y);}}e c rg(float l5,c W6,c Ue,bool n4,float ic,float jc){const float kc=0.5*a5;c a3=c(Ue.x*a5+kc,Ue.y*ic+jc);return Te(l5,W6,a3,n4);}e c Sa(f P1,float ic,float jc){const float kc=0.5*a5;bool n4=P1.z<0.0;P1.z=abs(P1.z);c a3;a3.y=floor(P1.z);a3.x=(P1.z-a3.y)*hc+kc;a3.y=a3.y*ic+jc;float l5=floor(a3.x);return Te(l5,P1.xy,a3,n4);}e d di(f P1){return fract(P1.w)*-2.0+1.5;}e d ke(f P1){const float Ji=-1.0/255.0;return ceil(P1.w)*Ji;}
#endif
