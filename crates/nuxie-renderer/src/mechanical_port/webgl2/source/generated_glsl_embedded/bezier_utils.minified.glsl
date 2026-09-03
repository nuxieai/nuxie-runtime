#ifndef Ub
#define Ub g
#endif
#ifndef M6
#define M6 d
#endif
e float S9(d o,d b){float Xe=dot(o,b);float Vb=dot(o,o)*dot(b,b);return(Vb==.0)?1.:clamp(Xe*inversesqrt(Vb),-1.,1.);}e void Ye(d r0,d z0,d D0,d K0,Z0(d)C,Z0(d)H,Z0(d)i2){i2=z0-r0;d N6=D0-z0;d h8=K0-r0;H=N6-i2;C=-3.*N6+h8;}e f0 T9(d r0,d z0,d D0,d K0){f0 t;t[0]=(any(notEqual(r0,z0))?z0:any(notEqual(z0,D0))?D0:K0)-r0;t[1]=K0-(any(notEqual(K0,D0))?D0:any(notEqual(D0,z0))?z0:r0);return t;}e float Ze(d r0,d z0,d D0,d K0,float r1,float af){d C,H,i2;Ye(r0,z0,D0,K0,C,H,i2);d O6=3.*(((C*r1)+2.*H)*r1+i2);float Wb=length(O6);if(Wb==.0){return.0;}O6*=1./Wb;float i8=2.*dot(C,O6);float P6=3.*(i8*r1+4.*dot(H,O6))*r1+6.*dot(i2,O6);float U9=min(r1,1.-r1);float bf=(i8*U9*U9+P6)*U9;float Xb=min(af,bf*.9999);float X2;if(i8==.0){X2=Xb/P6;}else{float K=1./i8;float b=P6*K,G1=-Xb*K;float Q6=(-1./3.)*b,R6=.5*G1;float Yb=R6*R6-Q6*Q6*Q6;if(Yb<.0){float j8=sqrt(Q6);float e1=acos(R6/(j8*j8*j8));X2=-2.*j8*cos(e1*(1./3.)+(-D3*2./3.));}else{float C=pow(abs(R6)+sqrt(Yb),1./3.);if(R6<.0)C=-C;X2=C!=.0?C+Q6/C:.0;}}X2=abs(X2);g t0011=r1+Ub(-X2,-X2,X2,X2);g Zb=(C.xyxy*t0011+2.*H.xyxy)*t0011+i2.xyxy;f0 H2=T9(r0,z0,D0,K0);d cf=t0011.x<1e-3?H2[0]:Zb.xy;d df=t0011.z>1.-1e-3?H2[1]:Zb.zw;return acos(S9(cf,df));}e float k8(float o,float b){o=b<.0?-o:o;b=abs(b);return o>.0?(o<b?o/b:1.):.0;}float ef(d r0,d z0,d D0,d K0,Z0(float)V9){d ac=K0-r0;float bc=length(K0-r0);if(bc==.0){V9=.5;return.0;}d Y2=M6(-ac.y,ac.x)/bc;float cc=dot(Y2,D0-r0);float z4=dot(Y2,z0-r0);float A4=z4-cc;
#if 0
float o=3.*A4;float dc=A4+z4;float G1=z4;float r2=sqrt(max(A4*A4+cc*z4,.0));if(dc<.0)r2=-r2;r2+=dc;d S6=M6(k8(r2,o),k8(G1,r2));d X5=3.*(S6*(S6*(S6*A4-(z4+A4))+z4));X5=abs(X5);V9=X5.x>X5.y?S6.x:S6.y;return max(X5.x,X5.y);
#else
float ec=3.*A4;float H=-z4-A4;float i2=z4;float t=.5;for(int F0=0;F0<3;++F0){float fc=ec*t;t=k8(fc*t-i2,2.*(fc+H));}V9=t;return abs(t*(t*(t*ec+3.*H)+3.*i2));
#endif
}
