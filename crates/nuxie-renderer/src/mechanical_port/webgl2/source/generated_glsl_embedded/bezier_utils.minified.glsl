#ifndef Tb
#define Tb g
#endif
#ifndef L6
#define L6 d
#endif
e float S9(d o,d b){float We=dot(o,b);float Ub=dot(o,o)*dot(b,b);return(Ub==.0)?1.:clamp(We*inversesqrt(Ub),-1.,1.);}e void Xe(d y0,d z0,d F0,d K0,Z0(d)C,Z0(d)H,Z0(d)h2){h2=z0-y0;d M6=F0-z0;d g8=K0-y0;H=M6-h2;C=-3.*M6+g8;}e f0 T9(d y0,d z0,d F0,d K0){f0 t;t[0]=(any(notEqual(y0,z0))?z0:any(notEqual(z0,F0))?F0:K0)-y0;t[1]=K0-(any(notEqual(K0,F0))?F0:any(notEqual(F0,z0))?z0:y0);return t;}e float Ye(d y0,d z0,d F0,d K0,float r1,float Ze){d C,H,h2;Xe(y0,z0,F0,K0,C,H,h2);d N6=3.*(((C*r1)+2.*H)*r1+h2);float Vb=length(N6);if(Vb==.0){return.0;}N6*=1./Vb;float h8=2.*dot(C,N6);float O6=3.*(h8*r1+4.*dot(H,N6))*r1+6.*dot(h2,N6);float U9=min(r1,1.-r1);float af=(h8*U9*U9+O6)*U9;float Wb=min(Ze,af*.9999);float X2;if(h8==.0){X2=Wb/O6;}else{float K=1./h8;float b=O6*K,G1=-Wb*K;float P6=(-1./3.)*b,Q6=.5*G1;float Xb=Q6*Q6-P6*P6*P6;if(Xb<.0){float i8=sqrt(P6);float e1=acos(Q6/(i8*i8*i8));X2=-2.*i8*cos(e1*(1./3.)+(-D3*2./3.));}else{float C=pow(abs(Q6)+sqrt(Xb),1./3.);if(Q6<.0)C=-C;X2=C!=.0?C+P6/C:.0;}}X2=abs(X2);g t0011=r1+Tb(-X2,-X2,X2,X2);g Yb=(C.xyxy*t0011+2.*H.xyxy)*t0011+h2.xyxy;f0 H2=T9(y0,z0,F0,K0);d bf=t0011.x<1e-3?H2[0]:Yb.xy;d cf=t0011.z>1.-1e-3?H2[1]:Yb.zw;return acos(S9(bf,cf));}e float j8(float o,float b){o=b<.0?-o:o;b=abs(b);return o>.0?(o<b?o/b:1.):.0;}float df(d y0,d z0,d F0,d K0,Z0(float)V9){d Zb=K0-y0;float ac=length(K0-y0);if(ac==.0){V9=.5;return.0;}d Y2=L6(-Zb.y,Zb.x)/ac;float bc=dot(Y2,F0-y0);float z4=dot(Y2,z0-y0);float A4=z4-bc;
#if 0
float o=3.*A4;float cc=A4+z4;float G1=z4;float q2=sqrt(max(A4*A4+bc*z4,.0));if(cc<.0)q2=-q2;q2+=cc;d R6=L6(j8(q2,o),j8(G1,q2));d W5=3.*(R6*(R6*(R6*A4-(z4+A4))+z4));W5=abs(W5);V9=W5.x>W5.y?R6.x:R6.y;return max(W5.x,W5.y);
#else
float dc=3.*A4;float H=-z4-A4;float h2=z4;float t=.5;for(int E0=0;E0<3;++E0){float ec=dc*t;t=j8(ec*t-h2,2.*(ec+H));}V9=t;return abs(t*(t*(t*dc+3.*H)+3.*h2));
#endif
}
