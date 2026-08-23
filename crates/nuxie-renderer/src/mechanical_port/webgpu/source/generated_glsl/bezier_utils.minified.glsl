#ifndef Rb
#define Rb g
#endif
#ifndef J6
#define J6 d
#endif
e float S9(d m,d b){float Te=dot(m,b);float Sb=dot(m,m)*dot(b,b);return(Sb==.0)?1.:clamp(Te*inversesqrt(Sb),-1.,1.);}e void Ue(d y0,d z0,d F0,d I0,Z0(d)C,Z0(d)H,Z0(d)g2){g2=z0-y0;d K6=F0-z0;d e8=I0-y0;H=K6-g2;C=-3.*K6+e8;}e g0 T9(d y0,d z0,d F0,d I0){g0 t;t[0]=(any(notEqual(y0,z0))?z0:any(notEqual(z0,F0))?F0:I0)-y0;t[1]=I0-(any(notEqual(I0,F0))?F0:any(notEqual(F0,z0))?z0:y0);return t;}e float Ve(d y0,d z0,d F0,d I0,float r1,float We){d C,H,g2;Ue(y0,z0,F0,I0,C,H,g2);d L6=3.*(((C*r1)+2.*H)*r1+g2);float Tb=length(L6);if(Tb==.0){return.0;}L6*=1./Tb;float f8=2.*dot(C,L6);float M6=3.*(f8*r1+4.*dot(H,L6))*r1+6.*dot(g2,L6);float U9=min(r1,1.-r1);float Xe=(f8*U9*U9+M6)*U9;float Ub=min(We,Xe*.9999);float V2;if(f8==.0){V2=Ub/M6;}else{float K=1./f8;float b=M6*K,G1=-Ub*K;float N6=(-1./3.)*b,O6=.5*G1;float Vb=O6*O6-N6*N6*N6;if(Vb<.0){float g8=sqrt(N6);float e1=acos(O6/(g8*g8*g8));V2=-2.*g8*cos(e1*(1./3.)+(-B3*2./3.));}else{float C=pow(abs(O6)+sqrt(Vb),1./3.);if(O6<.0)C=-C;V2=C!=.0?C+N6/C:.0;}}V2=abs(V2);g t0011=r1+Rb(-V2,-V2,V2,V2);g Wb=(C.xyxy*t0011+2.*H.xyxy)*t0011+g2.xyxy;g0 F2=T9(y0,z0,F0,I0);d Ye=t0011.x<1e-3?F2[0]:Wb.xy;d Ze=t0011.z>1.-1e-3?F2[1]:Wb.zw;return acos(S9(Ye,Ze));}e float h8(float m,float b){m=b<.0?-m:m;b=abs(b);return m>.0?(m<b?m/b:1.):.0;}float af(d y0,d z0,d F0,d I0,Z0(float)V9){d Xb=I0-y0;float Yb=length(I0-y0);if(Yb==.0){V9=.5;return.0;}d W2=J6(-Xb.y,Xb.x)/Yb;float Zb=dot(W2,F0-y0);float x4=dot(W2,z0-y0);float y4=x4-Zb;
#if 0
float m=3.*y4;float ac=y4+x4;float G1=x4;float q2=sqrt(max(y4*y4+Zb*x4,.0));if(ac<.0)q2=-q2;q2+=ac;d P6=J6(h8(q2,m),h8(G1,q2));d U5=3.*(P6*(P6*(P6*y4-(x4+y4))+x4));U5=abs(U5);V9=U5.x>U5.y?P6.x:P6.y;return max(U5.x,U5.y);
#else
float bc=3.*y4;float H=-x4-y4;float g2=x4;float t=.5;for(int E0=0;E0<3;++E0){float cc=bc*t;t=h8(cc*t-g2,2.*(cc+H));}V9=t;return abs(t*(t*(t*bc+3.*H)+3.*g2));
#endif
}