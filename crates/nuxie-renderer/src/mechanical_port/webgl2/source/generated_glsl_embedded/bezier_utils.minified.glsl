#ifndef Wb
#define Wb g
#endif
#ifndef N6
#define N6 d
#endif
e float Q9(d o,d b){float bf=dot(o,b);float Xb=dot(o,o)*dot(b,b);return(Xb==.0)?1.:clamp(bf*inversesqrt(Xb),-1.,1.);}e void cf(d r0,d z0,d D0,d K0,Z0(d)C,Z0(d)H,Z0(d)i2){i2=z0-r0;d O6=D0-z0;d h8=K0-r0;H=O6-i2;C=-3.*O6+h8;}e f0 R9(d r0,d z0,d D0,d K0){f0 t;t[0]=(any(notEqual(r0,z0))?z0:any(notEqual(z0,D0))?D0:K0)-r0;t[1]=K0-(any(notEqual(K0,D0))?D0:any(notEqual(D0,z0))?z0:r0);return t;}e float df(d r0,d z0,d D0,d K0,float r1,float ef){d C,H,i2;cf(r0,z0,D0,K0,C,H,i2);d P6=3.*(((C*r1)+2.*H)*r1+i2);float Yb=length(P6);if(Yb==.0){return.0;}P6*=1./Yb;float i8=2.*dot(C,P6);float Q6=3.*(i8*r1+4.*dot(H,P6))*r1+6.*dot(i2,P6);float S9=min(r1,1.-r1);float ff=(i8*S9*S9+Q6)*S9;float Zb=min(ef,ff*.9999);float X2;if(i8==.0){X2=Zb/Q6;}else{float J=1./i8;float b=Q6*J,H1=-Zb*J;float R6=(-1./3.)*b,S6=.5*H1;float ac=S6*S6-R6*R6*R6;if(ac<.0){float j8=sqrt(R6);float e1=acos(S6/(j8*j8*j8));X2=-2.*j8*cos(e1*(1./3.)+(-C3*2./3.));}else{float C=pow(abs(S6)+sqrt(ac),1./3.);if(S6<.0)C=-C;X2=C!=.0?C+R6/C:.0;}}X2=abs(X2);g t0011=r1+Wb(-X2,-X2,X2,X2);g bc=(C.xyxy*t0011+2.*H.xyxy)*t0011+i2.xyxy;f0 H2=R9(r0,z0,D0,K0);d gf=t0011.x<1e-3?H2[0]:bc.xy;d hf=t0011.z>1.-1e-3?H2[1]:bc.zw;return acos(Q9(gf,hf));}e float k8(float o,float b){o=b<.0?-o:o;b=abs(b);return o>.0?(o<b?o/b:1.):.0;}float jf(d r0,d z0,d D0,d K0,Z0(float)T9){d cc=K0-r0;float dc=length(K0-r0);if(dc==.0){T9=.5;return.0;}d Y2=N6(-cc.y,cc.x)/dc;float ec=dot(Y2,D0-r0);float z4=dot(Y2,z0-r0);float A4=z4-ec;
#if 0
float o=3.*A4;float fc=A4+z4;float H1=z4;float r2=sqrt(max(A4*A4+ec*z4,.0));if(fc<.0)r2=-r2;r2+=fc;d T6=N6(k8(r2,o),k8(H1,r2));d X5=3.*(T6*(T6*(T6*A4-(z4+A4))+z4));X5=abs(X5);T9=X5.x>X5.y?T6.x:T6.y;return max(X5.x,X5.y);
#else
float gc=3.*A4;float H=-z4-A4;float i2=z4;float t=.5;for(int F0=0;F0<3;++F0){float hc=gc*t;t=k8(hc*t-i2,2.*(hc+H));}T9=t;return abs(t*(t*(t*gc+3.*H)+3.*i2));
#endif
}
