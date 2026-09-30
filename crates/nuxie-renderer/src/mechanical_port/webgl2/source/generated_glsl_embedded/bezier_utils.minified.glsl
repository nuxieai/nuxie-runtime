#ifndef Ec
#define Ec f
#endif
#ifndef Z6
#define Z6 c
#endif
e float y8(c k,c b){float Qf=dot(k,b);float Fc=dot(k,k)*dot(b,b);return(Fc==.0)?1.:clamp(Qf*inversesqrt(Fc),-1.,1.);}e void Rf(c z0,c C0,c J0,c P0,i1(c) B,i1(c) J,i1(c) q2){q2=C0-z0;c a7=J0-C0;c z8=P0-z0;J=a7-q2;B=-3.*a7+z8;}e Y ra(c z0,c C0,c J0,c P0){Y t;t[0]=(any(notEqual(z0,C0))?C0:any(notEqual(C0,J0))?J0:P0)-z0;t[1]=P0-(any(notEqual(P0,J0))?J0:any(notEqual(J0,C0))?C0:z0);return t;}e float Sf(c z0,c C0,c J0,c P0,float C1,float Tf){c B,J,q2;Rf(z0,C0,J0,P0,B,J,q2);c c7=3.*(((B*C1)+2.*J)*C1+q2);float Gc=length(c7);if(Gc==.0){return.0;}c7*=1./Gc;float A8=2.*dot(B,c7);float d7=3.*(A8*C1+4.*dot(J,c7))*C1+6.*dot(q2,c7);float sa=min(C1,1.-C1);float Uf=(A8*sa*sa+d7)*sa;float Hc=min(Tf,Uf*.9999);float h3;if(A8==.0){h3=Hc/d7;}else{float Q=1./A8;float b=d7*Q,O1=-Hc*Q;float e7=(-1./3.)*b,f7=.5*O1;float Ic=f7*f7-e7*e7*e7;if(Ic<.0){float B8=sqrt(e7);float x1=acos(f7/(B8*B8*B8));h3=-2.*B8*cos(x1*(1./3.)+(-j4*2./3.));}else{float B=pow(abs(f7)+sqrt(Ic),1./3.);if(f7<.0) B=-B;h3=B!=.0?B+e7/B:.0;}}h3=abs(h3);f t0011=C1+Ec(-h3,-h3,h3,h3);f Jc=(B.xyxy*t0011+2.*J.xyxy)*t0011+q2.xyxy;Y r2=ra(z0,C0,J0,P0);c Vf=t0011.x<1e-3?r2[0]:Jc.xy;c Wf=t0011.z>1.-1e-3?r2[1]:Jc.zw;return acos(y8(Vf,Wf));}e float C8(float k,float b){k=b<.0?-k:k;b=abs(b);return k>.0?(k<b?k/b:1.):.0;}float Xf(c z0,c C0,c J0,c P0,i1(float) ta){c Kc=P0-z0;float Lc=length(P0-z0);if(Lc==.0){ta=.5;return.0;}c P2=Z6(-Kc.y,Kc.x)/Lc;float Mc=dot(P2,J0-z0);float O4=dot(P2,C0-z0);float P4=O4-Mc;
#if 0
float k=3.*P4;float Nc=P4+O4;float O1=O4;float C2=sqrt(max(P4*P4+Mc*O4,.0));if(Nc<.0) C2=-C2;C2+=Nc;c g7=Z6(C8(C2,k),C8(O1,C2));c i6=3.*(g7*(g7*(g7*P4-(O4+P4))+O4));i6=abs(i6);ta=i6.x>i6.y?g7.x:g7.y;return max(i6.x,i6.y);
#else
float Oc=3.*P4;float J=-O4-P4;float q2=O4;float t=.5;for(int L0=0;L0<3;++L0){float Pc=Oc*t;t=C8(Pc*t-q2,2.*(Pc+J));}ta=t;return abs(t*(t*(t*Oc+3.*J)+3.*q2));
#endif
}
