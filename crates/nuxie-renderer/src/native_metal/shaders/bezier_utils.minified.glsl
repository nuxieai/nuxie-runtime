#ifndef gc
#define gc f
#endif
#ifndef P6
#define P6 c
#endif
e float X9(c n,c b){float vf=dot(n,b);float hc=dot(n,n)*dot(b,b);return(hc==.0)?1.:clamp(vf*inversesqrt(hc),-1.,1.);}e void wf(c x0,c B0,c F0,c M0,a1(c)C,a1(c)H,a1(c)l2){l2=B0-x0;c Q6=F0-B0;c k8=M0-x0;H=Q6-l2;C=-3.*Q6+k8;}e e0 Y9(c x0,c B0,c F0,c M0){e0 t;t[0]=(any(notEqual(x0,B0))?B0:any(notEqual(B0,F0))?F0:M0)-x0;t[1]=M0-(any(notEqual(M0,F0))?F0:any(notEqual(F0,B0))?B0:x0);return t;}e float xf(c x0,c B0,c F0,c M0,float v1,float yf){c C,H,l2;wf(x0,B0,F0,M0,C,H,l2);c R6=3.*(((C*v1)+2.*H)*v1+l2);float ic=length(R6);if(ic==.0){return.0;}R6*=1./ic;float l8=2.*dot(C,R6);float S6=3.*(l8*v1+4.*dot(H,R6))*v1+6.*dot(l2,R6);float Z9=min(v1,1.-v1);float zf=(l8*Z9*Z9+S6)*Z9;float jc=min(yf,zf*.9999);float Z2;if(l8==.0){Z2=jc/S6;}else{float P=1./l8;float b=S6*P,I1=-jc*P;float T6=(-1./3.)*b,U6=.5*I1;float kc=U6*U6-T6*T6*T6;if(kc<.0){float m8=sqrt(T6);float f1=acos(U6/(m8*m8*m8));Z2=-2.*m8*cos(f1*(1./3.)+(-H3*2./3.));}else{float C=pow(abs(U6)+sqrt(kc),1./3.);if(U6<.0)C=-C;Z2=C!=.0?C+T6/C:.0;}}Z2=abs(Z2);f t0011=v1+gc(-Z2,-Z2,Z2,Z2);f lc=(C.xyxy*t0011+2.*H.xyxy)*t0011+l2.xyxy;e0 K2=Y9(x0,B0,F0,M0);c Af=t0011.x<1e-3?K2[0]:lc.xy;c Bf=t0011.z>1.-1e-3?K2[1]:lc.zw;return acos(X9(Af,Bf));}e float n8(float n,float b){n=b<.0?-n:n;b=abs(b);return n>.0?(n<b?n/b:1.):.0;}float Cf(c x0,c B0,c F0,c M0,a1(float)aa){c mc=M0-x0;float nc=length(M0-x0);if(nc==.0){aa=.5;return.0;}c a3=P6(-mc.y,mc.x)/nc;float oc=dot(a3,F0-x0);float C4=dot(a3,B0-x0);float D4=C4-oc;
#if 0
float n=3.*D4;float pc=D4+C4;float I1=C4;float x2=sqrt(max(D4*D4+oc*C4,.0));if(pc<.0)x2=-x2;x2+=pc;c V6=P6(n8(x2,n),n8(I1,x2));c a6=3.*(V6*(V6*(V6*D4-(C4+D4))+C4));a6=abs(a6);aa=a6.x>a6.y?V6.x:V6.y;return max(a6.x,a6.y);
#else
float qc=3.*D4;float H=-C4-D4;float l2=C4;float t=.5;for(int H0=0;H0<3;++H0){float rc=qc*t;t=n8(rc*t-l2,2.*(rc+H));}aa=t;return abs(t*(t*(t*qc+3.*H)+3.*l2));
#endif
}