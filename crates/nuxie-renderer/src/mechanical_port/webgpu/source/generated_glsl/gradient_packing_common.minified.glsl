#define ic 8.0
#ifdef VERTEX
f e Fa(c l0,X U9,c y2,c n8,uint m5,float q4,float n){e U1;U1.xy=C0(U9,l0)+y2;bool p4=(n8.x<0.0);n8.x=max(n8.x,0.0);U1.z=n8.y+(float(m5)/ic)+(n8.x*(d5/ic));U1.z=(p4)?-U1.z:U1.z;U1.w=(n*-0.5-0.25)-round(q4*255.0);return U1;}
#endif
#ifdef FRAGMENT
f c Ye(float m5,c a7,c a3,bool p4){if(m5==float(zh)){float t=gb(c(a7.x,-a7.y))/(3.14159*2.0);return c(t,a3.y);}else{float t=(m5==float(yh))?a7.x:length(a7.xy);t=clamp(t,0.0,1.0);float Oi=p4?(1.0-d5):d5;float x=t*Oi+a3.x;return c(x,a3.y);}}f c wg(float m5,c a7,c Ze,bool p4,float jc,float kc){const float lc=0.5*d5;c a3=c(Ze.x*d5+lc,Ze.y*jc+kc);return Ye(m5,a7,a3,p4);}f c Va(e U1,float jc,float kc){const float lc=0.5*d5;bool p4=U1.z<0.0;U1.z=abs(U1.z);c a3;a3.y=floor(U1.z);a3.x=(U1.z-a3.y)*ic+lc;a3.y=a3.y*jc+kc;float m5=floor(a3.x);return Ye(m5,U1.xy,a3,p4);}f d ji(e U1){return fract(U1.w)*-2.0+1.5;}f d pe(e U1){const float Pi=-1.0/255.0;return ceil(U1.w)*Pi;}
#endif
