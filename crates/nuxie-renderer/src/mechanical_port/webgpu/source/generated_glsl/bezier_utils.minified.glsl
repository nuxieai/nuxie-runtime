#ifndef Vb
#define Vb g
#endif
#ifndef N6
#define N6 d
#endif
e float S9(d o,d b){float bf=dot(o,b);float Wb=dot(o,o)*dot(b,b);return(Wb==.0)?1.:clamp(bf*inversesqrt(Wb),-1.,1.);}e void cf(d r0,d z0,d D0,d K0,Z0(d)C,Z0(d)H,Z0(d)i2){i2=z0-r0;d O6=D0-z0;d h8=K0-r0;H=O6-i2;C=-3.*O6+h8;}e f0 T9(d r0,d z0,d D0,d K0){f0 t;t[0]=(any(notEqual(r0,z0))?z0:any(notEqual(z0,D0))?D0:K0)-r0;t[1]=K0-(any(notEqual(K0,D0))?D0:any(notEqual(D0,z0))?z0:r0);return t;}e float df(d r0,d z0,d D0,d K0,float r1,float ef){d C,H,i2;cf(r0,z0,D0,K0,C,H,i2);d P6=3.*(((C*r1)+2.*H)*r1+i2);float Xb=length(P6);if(Xb==.0){return.0;}P6*=1./Xb;float i8=2.*dot(C,P6);float Q6=3.*(i8*r1+4.*dot(H,P6))*r1+6.*dot(i2,P6);float U9=min(r1,1.-r1);float ff=(i8*U9*U9+Q6)*U9;float Yb=min(ef,ff*.9999);float X2;if(i8==.0){X2=Yb/Q6;}else{float K=1./i8;float b=Q6*K,G1=-Yb*K;float R6=(-1./3.)*b,S6=.5*G1;float Zb=S6*S6-R6*R6*R6;if(Zb<.0){float j8=sqrt(R6);float e1=acos(S6/(j8*j8*j8));X2=-2.*j8*cos(e1*(1./3.)+(-D3*2./3.));}else{float C=pow(abs(S6)+sqrt(Zb),1./3.);if(S6<.0)C=-C;X2=C!=.0?C+R6/C:.0;}}X2=abs(X2);g t0011=r1+Vb(-X2,-X2,X2,X2);g ac=(C.xyxy*t0011+2.*H.xyxy)*t0011+i2.xyxy;f0 H2=T9(r0,z0,D0,K0);d gf=t0011.x<1e-3?H2[0]:ac.xy;d hf=t0011.z>1.-1e-3?H2[1]:ac.zw;return acos(S9(gf,hf));}e float k8(float o,float b){o=b<.0?-o:o;b=abs(b);return o>.0?(o<b?o/b:1.):.0;}float jf(d r0,d z0,d D0,d K0,Z0(float)V9){d bc=K0-r0;float cc=length(K0-r0);if(cc==.0){V9=.5;return.0;}d Y2=N6(-bc.y,bc.x)/cc;float dc=dot(Y2,D0-r0);float A4=dot(Y2,z0-r0);float B4=A4-dc;
#if 0
float o=3.*B4;float ec=B4+A4;float G1=A4;float r2=sqrt(max(B4*B4+dc*A4,.0));if(ec<.0)r2=-r2;r2+=ec;d T6=N6(k8(r2,o),k8(G1,r2));d Y5=3.*(T6*(T6*(T6*B4-(A4+B4))+A4));Y5=abs(Y5);V9=Y5.x>Y5.y?T6.x:T6.y;return max(Y5.x,Y5.y);
#else
float fc=3.*B4;float H=-A4-B4;float i2=A4;float t=.5;for(int F0=0;F0<3;++F0){float gc=fc*t;t=k8(gc*t-i2,2.*(gc+H));}V9=t;return abs(t*(t*(t*fc+3.*H)+3.*i2));
#endif
}