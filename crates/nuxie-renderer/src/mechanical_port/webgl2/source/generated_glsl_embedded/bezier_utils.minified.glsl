#ifndef dc
#define dc f
#endif
#ifndef N6
#define N6 c
#endif
e float V9(c m,c b){float sf=dot(m,b);float ec=dot(m,m)*dot(b,b);return(ec==.0)?1.:clamp(sf*inversesqrt(ec),-1.,1.);}e void tf(c v0,c A0,c E0,c L0,Z0(c)C,Z0(c)H,Z0(c)k2){k2=A0-v0;c O6=E0-A0;c i8=L0-v0;H=O6-k2;C=-3.*O6+i8;}e d0 W9(c v0,c A0,c E0,c L0){d0 t;t[0]=(any(notEqual(v0,A0))?A0:any(notEqual(A0,E0))?E0:L0)-v0;t[1]=L0-(any(notEqual(L0,E0))?E0:any(notEqual(E0,A0))?A0:v0);return t;}e float uf(c v0,c A0,c E0,c L0,float q1,float vf){c C,H,k2;tf(v0,A0,E0,L0,C,H,k2);c P6=3.*(((C*q1)+2.*H)*q1+k2);float fc=length(P6);if(fc==.0){return.0;}P6*=1./fc;float j8=2.*dot(C,P6);float Q6=3.*(j8*q1+4.*dot(H,P6))*q1+6.*dot(k2,P6);float X9=min(q1,1.-q1);float wf=(j8*X9*X9+Q6)*X9;float gc=min(vf,wf*.9999);float Z2;if(j8==.0){Z2=gc/Q6;}else{float P=1./j8;float b=Q6*P,G1=-gc*P;float R6=(-1./3.)*b,S6=.5*G1;float hc=S6*S6-R6*R6*R6;if(hc<.0){float k8=sqrt(R6);float e1=acos(S6/(k8*k8*k8));Z2=-2.*k8*cos(e1*(1./3.)+(-D3*2./3.));}else{float C=pow(abs(S6)+sqrt(hc),1./3.);if(S6<.0)C=-C;Z2=C!=.0?C+R6/C:.0;}}Z2=abs(Z2);f t0011=q1+dc(-Z2,-Z2,Z2,Z2);f ic=(C.xyxy*t0011+2.*H.xyxy)*t0011+k2.xyxy;d0 J2=W9(v0,A0,E0,L0);c xf=t0011.x<1e-3?J2[0]:ic.xy;c yf=t0011.z>1.-1e-3?J2[1]:ic.zw;return acos(V9(xf,yf));}e float l8(float m,float b){m=b<.0?-m:m;b=abs(b);return m>.0?(m<b?m/b:1.):.0;}float zf(c v0,c A0,c E0,c L0,Z0(float)Y9){c jc=L0-v0;float kc=length(L0-v0);if(kc==.0){Y9=.5;return.0;}c a3=N6(-jc.y,jc.x)/kc;float lc=dot(a3,E0-v0);float z4=dot(a3,A0-v0);float A4=z4-lc;
#if 0
float m=3.*A4;float mc=A4+z4;float G1=z4;float w2=sqrt(max(A4*A4+lc*z4,.0));if(mc<.0)w2=-w2;w2+=mc;c T6=N6(l8(w2,m),l8(G1,w2));c Y5=3.*(T6*(T6*(T6*A4-(z4+A4))+z4));Y5=abs(Y5);Y9=Y5.x>Y5.y?T6.x:T6.y;return max(Y5.x,Y5.y);
#else
float nc=3.*A4;float H=-z4-A4;float k2=z4;float t=.5;for(int G0=0;G0<3;++G0){float oc=nc*t;t=l8(oc*t-k2,2.*(oc+H));}Y9=t;return abs(t*(t*(t*nc+3.*H)+3.*k2));
#endif
}
