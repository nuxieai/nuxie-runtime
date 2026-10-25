#define mc 8.0
#ifdef VERTEX
f e Ga(c l0,X V9,c y2,c p8,uint o5,float r4,float n){e U1;U1.xy=D0(V9,l0)+y2;bool q4=(p8.x<0.0);p8.x=max(p8.x,0.0);U1.z=p8.y+(float(o5)/mc)+(p8.x*(f5/mc));U1.z=(q4)?-U1.z:U1.z;U1.w=(n*-0.5-0.25)-round(r4*255.0);return U1;}
#endif
#ifdef FRAGMENT
f c gf(float o5,c a7,c a3,bool q4){
#ifdef ENABLE_ANGULAR_GRADIENT
if(ENABLE_ANGULAR_GRADIENT&&o5==float(Fh)){float t=hb(c(a7.x,-a7.y))*(1.0/B7);return c(t,a3.y);}else
#endif
{float t=(o5==float(Eh))?a7.x:length(a7.xy);t=clamp(t,0.0,1.0);float Ti=q4?(1.0-f5):f5;float x=t*Ti+a3.x;return c(x,a3.y);}}f c Cg(float o5,c a7,c hf,bool q4,float nc,float oc){const float pc=0.5*f5;c a3=c(hf.x*f5+pc,hf.y*nc+oc);return gf(o5,a7,a3,q4);}f c Wa(e U1,float nc,float oc){const float pc=0.5*f5;bool q4=U1.z<0.0;U1.z=abs(U1.z);c a3;a3.y=floor(U1.z);a3.x=(U1.z-a3.y)*mc+pc;a3.y=a3.y*nc+oc;float o5=floor(a3.x);return gf(o5,U1.xy,a3,q4);}f d ni(e U1){return fract(U1.w)*-2.0+1.5;}f d ve(e U1){const float Ui=-1.0/255.0;return ceil(U1.w)*Ui;}
#endif
