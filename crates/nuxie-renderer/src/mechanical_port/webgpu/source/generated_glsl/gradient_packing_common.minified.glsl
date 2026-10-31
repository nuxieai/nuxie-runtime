#define oc 8.0
#ifdef VERTEX
f e Ga(c l0,X V9,c A2,c n8,uint n5,float r4,float n){e U1;U1.xy=D0(V9,l0)+A2;bool q4=(n8.x<0.0);n8.x=max(n8.x,0.0);U1.z=n8.y+(float(n5)/oc)+(n8.x*(f5/oc));U1.z=(q4)?-U1.z:U1.z;U1.w=(n*-0.5-0.25)-round(r4*255.0);return U1;}
#endif
#ifdef FRAGMENT
f c hf(float n5,c o8,c c3,bool q4){
#ifdef ENABLE_ANGULAR_GRADIENT
if(ENABLE_ANGULAR_GRADIENT&&n5==float(Fh)){float t=ib(o8)*(1.0/A7);return c(t,c3.y);}else
#endif
{float t=(n5==float(Eh))?o8.x:length(o8.xy);t=clamp(t,0.0,1.0);float Ti=q4?(1.0-f5):f5;float x=t*Ti+c3.x;return c(x,c3.y);}}f c Cg(float n5,c o8,c jf,bool q4,float pc,float qc){const float rc=0.5*f5;c c3=c(jf.x*f5+rc,jf.y*pc+qc);return hf(n5,o8,c3,q4);}f c Xa(e U1,float pc,float qc){const float rc=0.5*f5;bool q4=U1.z<0.0;U1.z=abs(U1.z);c c3;c3.y=floor(U1.z);c3.x=(U1.z-c3.y)*oc+rc;c3.y=c3.y*pc+qc;float n5=floor(c3.x);return hf(n5,U1.xy,c3,q4);}f d oi(e U1){return fract(U1.w)*-2.0+1.5;}f d ve(e U1){const float Ui=-1.0/255.0;return ceil(U1.w)*Ui;}
#endif
