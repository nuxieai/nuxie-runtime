#ifndef fc
#define fc f
#endif
#ifndef O6
#define O6 c
#endif
e float W9(c m,c b){float uf=dot(m,b);float gc=dot(m,m)*dot(b,b);return(gc==.0)?1.:clamp(uf*inversesqrt(gc),-1.,1.);}e void vf(c w0,c A0,c E0,c L0,Z0(c)C,Z0(c)H,Z0(c)k2){k2=A0-w0;c P6=E0-A0;c i8=L0-w0;H=P6-k2;C=-3.*P6+i8;}e d0 X9(c w0,c A0,c E0,c L0){d0 t;t[0]=(any(notEqual(w0,A0))?A0:any(notEqual(A0,E0))?E0:L0)-w0;t[1]=L0-(any(notEqual(L0,E0))?E0:any(notEqual(E0,A0))?A0:w0);return t;}e float wf(c w0,c A0,c E0,c L0,float q1,float xf){c C,H,k2;vf(w0,A0,E0,L0,C,H,k2);c Q6=3.*(((C*q1)+2.*H)*q1+k2);float hc=length(Q6);if(hc==.0){return.0;}Q6*=1./hc;float j8=2.*dot(C,Q6);float R6=3.*(j8*q1+4.*dot(H,Q6))*q1+6.*dot(k2,Q6);float Y9=min(q1,1.-q1);float yf=(j8*Y9*Y9+R6)*Y9;float ic=min(xf,yf*.9999);float Y2;if(j8==.0){Y2=ic/R6;}else{float P=1./j8;float b=R6*P,G1=-ic*P;float S6=(-1./3.)*b,T6=.5*G1;float jc=T6*T6-S6*S6*S6;if(jc<.0){float k8=sqrt(S6);float e1=acos(T6/(k8*k8*k8));Y2=-2.*k8*cos(e1*(1./3.)+(-E3*2./3.));}else{float C=pow(abs(T6)+sqrt(jc),1./3.);if(T6<.0)C=-C;Y2=C!=.0?C+S6/C:.0;}}Y2=abs(Y2);f t0011=q1+fc(-Y2,-Y2,Y2,Y2);f kc=(C.xyxy*t0011+2.*H.xyxy)*t0011+k2.xyxy;d0 J2=X9(w0,A0,E0,L0);c zf=t0011.x<1e-3?J2[0]:kc.xy;c Af=t0011.z>1.-1e-3?J2[1]:kc.zw;return acos(W9(zf,Af));}e float l8(float m,float b){m=b<.0?-m:m;b=abs(b);return m>.0?(m<b?m/b:1.):.0;}float Bf(c w0,c A0,c E0,c L0,Z0(float)Z9){c lc=L0-w0;float mc=length(L0-w0);if(mc==.0){Z9=.5;return.0;}c Z2=O6(-lc.y,lc.x)/mc;float nc=dot(Z2,E0-w0);float z4=dot(Z2,A0-w0);float A4=z4-nc;
#if 0
float m=3.*A4;float oc=A4+z4;float G1=z4;float w2=sqrt(max(A4*A4+nc*z4,.0));if(oc<.0)w2=-w2;w2+=oc;c U6=O6(l8(w2,m),l8(G1,w2));c X5=3.*(U6*(U6*(U6*A4-(z4+A4))+z4));X5=abs(X5);Z9=X5.x>X5.y?U6.x:U6.y;return max(X5.x,X5.y);
#else
float pc=3.*A4;float H=-z4-A4;float k2=z4;float t=.5;for(int G0=0;G0<3;++G0){float qc=pc*t;t=l8(qc*t-k2,2.*(qc+H));}Z9=t;return abs(t*(t*(t*pc+3.*H)+3.*k2));
#endif
}