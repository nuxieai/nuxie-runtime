#ifndef Dc
#define Dc e
#endif
#ifndef X6
#define X6 c
#endif
f float w8(c k,c b){float Mf=dot(k,b);float Ec=dot(k,k)*dot(b,b);return(Ec==.0)?1.:clamp(Mf*inversesqrt(Ec),-1.,1.);}f void Nf(c y0,c C0,c H0,c P0,i1(c) B,i1(c) J,i1(c) q2){q2=C0-y0;c Y6=H0-C0;c x8=P0-y0;J=Y6-q2;B=-3.*Y6+x8;}f Y qa(c y0,c C0,c H0,c P0){Y t;t[0]=(any(notEqual(y0,C0))?C0:any(notEqual(C0,H0))?H0:P0)-y0;t[1]=P0-(any(notEqual(P0,H0))?H0:any(notEqual(H0,C0))?C0:y0);return t;}f float Of(c y0,c C0,c H0,c P0,float B1,float Pf){c B,J,q2;Nf(y0,C0,H0,P0,B,J,q2);c Z6=3.*(((B*B1)+2.*J)*B1+q2);float Fc=length(Z6);if(Fc==.0){return.0;}Z6*=1./Fc;float y8=2.*dot(B,Z6);float a7=3.*(y8*B1+4.*dot(J,Z6))*B1+6.*dot(q2,Z6);float ra=min(B1,1.-B1);float Qf=(y8*ra*ra+a7)*ra;float Gc=min(Pf,Qf*.9999);float h3;if(y8==.0){h3=Gc/a7;}else{float Q=1./y8;float b=a7*Q,N1=-Gc*Q;float c7=(-1./3.)*b,d7=.5*N1;float Hc=d7*d7-c7*c7*c7;if(Hc<.0){float z8=sqrt(c7);float w1=acos(d7/(z8*z8*z8));h3=-2.*z8*cos(w1*(1./3.)+(-i4*2./3.));}else{float B=pow(abs(d7)+sqrt(Hc),1./3.);if(d7<.0) B=-B;h3=B!=.0?B+c7/B:.0;}}h3=abs(h3);e t0011=B1+Dc(-h3,-h3,h3,h3);e Ic=(B.xyxy*t0011+2.*J.xyxy)*t0011+q2.xyxy;Y r2=qa(y0,C0,H0,P0);c Rf=t0011.x<1e-3?r2[0]:Ic.xy;c Sf=t0011.z>1.-1e-3?r2[1]:Ic.zw;return acos(w8(Rf,Sf));}f float A8(float k,float b){k=b<.0?-k:k;b=abs(b);return k>.0?(k<b?k/b:1.):.0;}float Tf(c y0,c C0,c H0,c P0,i1(float) sa){c Jc=P0-y0;float Kc=length(P0-y0);if(Kc==.0){sa=.5;return.0;}c P2=X6(-Jc.y,Jc.x)/Kc;float Lc=dot(P2,H0-y0);float N4=dot(P2,C0-y0);float O4=N4-Lc;
#if 0
float k=3.*O4;float Mc=O4+N4;float N1=N4;float C2=sqrt(max(O4*O4+Lc*N4,.0));if(Mc<.0) C2=-C2;C2+=Mc;c e7=X6(A8(C2,k),A8(N1,C2));c h6=3.*(e7*(e7*(e7*O4-(N4+O4))+N4));h6=abs(h6);sa=h6.x>h6.y?e7.x:e7.y;return max(h6.x,h6.y);
#else
float Nc=3.*O4;float J=-N4-O4;float q2=N4;float t=.5;for(int J0=0;J0<3;++J0){float Oc=Nc*t;t=A8(Oc*t-q2,2.*(Oc+J));}sa=t;return abs(t*(t*(t*Nc+3.*J)+3.*q2));
#endif
}