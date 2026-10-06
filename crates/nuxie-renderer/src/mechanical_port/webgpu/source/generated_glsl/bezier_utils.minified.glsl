#ifndef Ec
#define Ec e
#endif
#ifndef Y6
#define Y6 c
#endif
f float w8(c k,c b){float Rf=dot(k,b);float Fc=dot(k,k)*dot(b,b);return(Fc==.0)?1.:clamp(Rf*inversesqrt(Fc),-1.,1.);}f void Sf(c z0,c C0,c J0,c P0,i1(c) B,i1(c) J,i1(c) p2){p2=C0-z0;c Z6=J0-C0;c x8=P0-z0;J=Z6-p2;B=-3.*Z6+x8;}f Y ra(c z0,c C0,c J0,c P0){Y t;t[0]=(any(notEqual(z0,C0))?C0:any(notEqual(C0,J0))?J0:P0)-z0;t[1]=P0-(any(notEqual(P0,J0))?J0:any(notEqual(J0,C0))?C0:z0);return t;}f float Tf(c z0,c C0,c J0,c P0,float D1,float Uf){c B,J,p2;Sf(z0,C0,J0,P0,B,J,p2);c a7=3.*(((B*D1)+2.*J)*D1+p2);float Gc=length(a7);if(Gc==.0){return.0;}a7*=1./Gc;float y8=2.*dot(B,a7);float c7=3.*(y8*D1+4.*dot(J,a7))*D1+6.*dot(p2,a7);float sa=min(D1,1.-D1);float Vf=(y8*sa*sa+c7)*sa;float Hc=min(Uf,Vf*.9999);float h3;if(y8==.0){h3=Hc/c7;}else{float R=1./y8;float b=c7*R,P1=-Hc*R;float d7=(-1./3.)*b,e7=.5*P1;float Ic=e7*e7-d7*d7*d7;if(Ic<.0){float z8=sqrt(d7);float y1=acos(e7/(z8*z8*z8));h3=-2.*z8*cos(y1*(1./3.)+(-j4*2./3.));}else{float B=pow(abs(e7)+sqrt(Ic),1./3.);if(e7<.0) B=-B;h3=B!=.0?B+d7/B:.0;}}h3=abs(h3);e t0011=D1+Ec(-h3,-h3,h3,h3);e Jc=(B.xyxy*t0011+2.*J.xyxy)*t0011+p2.xyxy;Y q2=ra(z0,C0,J0,P0);c Wf=t0011.x<1e-3?q2[0]:Jc.xy;c Xf=t0011.z>1.-1e-3?q2[1]:Jc.zw;return acos(w8(Wf,Xf));}f float A8(float k,float b){k=b<.0?-k:k;b=abs(b);return k>.0?(k<b?k/b:1.):.0;}float Yf(c z0,c C0,c J0,c P0,i1(float) ta){c Kc=P0-z0;float Lc=length(P0-z0);if(Lc==.0){ta=.5;return.0;}c O2=Y6(-Kc.y,Kc.x)/Lc;float Mc=dot(O2,J0-z0);float O4=dot(O2,C0-z0);float P4=O4-Mc;
#if 0
float k=3.*P4;float Nc=P4+O4;float P1=O4;float B2=sqrt(max(P4*P4+Mc*O4,.0));if(Nc<.0) B2=-B2;B2+=Nc;c f7=Y6(A8(B2,k),A8(P1,B2));c h6=3.*(f7*(f7*(f7*P4-(O4+P4))+O4));h6=abs(h6);ta=h6.x>h6.y?f7.x:f7.y;return max(h6.x,h6.y);
#else
float Oc=3.*P4;float J=-O4-P4;float p2=O4;float t=.5;for(int L0=0;L0<3;++L0){float Pc=Oc*t;t=A8(Pc*t-p2,2.*(Pc+J));}ta=t;return abs(t*(t*(t*Oc+3.*J)+3.*p2));
#endif
}