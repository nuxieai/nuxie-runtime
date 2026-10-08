#ifndef id
#define id e
#endif
#ifndef i7
#define i7 c
#endif
f float P8(c k,c b){float ug=dot(k,b);float jd=dot(k,k)*dot(b,b);return(jd==.0)?1.:clamp(ug*inversesqrt(jd),-1.,1.);}f void vg(c A0,c D0,c K0,c U0,c1(c) A,c1(c) J,c1(c) y2){y2=D0-A0;c j7=K0-D0;c Q8=U0-A0;J=j7-y2;A=-3.*j7+Q8;}f W Pa(c A0,c D0,c K0,c U0){W t;t[0]=(any(notEqual(A0,D0))?D0:any(notEqual(D0,K0))?K0:U0)-A0;t[1]=U0-(any(notEqual(U0,K0))?K0:any(notEqual(K0,D0))?D0:A0);return t;}f float wg(c A0,c D0,c K0,c U0,float F1,float xg){c A,J,y2;vg(A0,D0,K0,U0,A,J,y2);c k7=3.*(((A*F1)+2.*J)*F1+y2);float kd=length(k7);if(kd==.0){return.0;}k7*=1./kd;float R8=2.*dot(A,k7);float l7=3.*(R8*F1+4.*dot(J,k7))*F1+6.*dot(y2,k7);float Qa=min(F1,1.-F1);float yg=(R8*Qa*Qa+l7)*Qa;float ld=min(xg,yg*.9999);float o3;if(R8==.0){o3=ld/l7;}else{float R=1./R8;float b=l7*R,S1=-ld*R;float m7=(-1./3.)*b,n7=.5*S1;float md=n7*n7-m7*m7*m7;if(md<.0){float S8=sqrt(m7);float h1=acos(n7/(S8*S8*S8));o3=-2.*S8*cos(h1*(1./3.)+(-n4*2./3.));}else{float A=pow(abs(n7)+sqrt(md),1./3.);if(n7<.0) A=-A;o3=A!=.0?A+m7/A:.0;}}o3=abs(o3);e t0011=F1+id(-o3,-o3,o3,o3);e nd=(A.xyxy*t0011+2.*J.xyxy)*t0011+y2.xyxy;W z2=Pa(A0,D0,K0,U0);c zg=t0011.x<1e-3?z2[0]:nd.xy;c Ag=t0011.z>1.-1e-3?z2[1]:nd.zw;return acos(P8(zg,Ag));}f float T8(float k,float b){k=b<.0?-k:k;b=abs(b);return k>.0?(k<b?k/b:1.):.0;}float Bg(c A0,c D0,c K0,c U0,c1(float) Ra){c od=U0-A0;float pd=length(U0-A0);if(pd==.0){Ra=.5;return.0;}c P1=i7(-od.y,od.x)/pd;float qd=dot(P1,K0-A0);float U4=dot(P1,D0-A0);float V4=U4-qd;
#if 0
float k=3.*V4;float rd=V4+U4;float S1=U4;float J2=sqrt(max(V4*V4+qd*U4,.0));if(rd<.0) J2=-J2;J2+=rd;c o7=i7(T8(J2,k),T8(S1,J2));c n6=3.*(o7*(o7*(o7*V4-(U4+V4))+U4));n6=abs(n6);Ra=n6.x>n6.y?o7.x:o7.y;return max(n6.x,n6.y);
#else
float sd=3.*V4;float J=-U4-V4;float y2=U4;float t=.5;for(int N0=0;N0<3;++N0){float td=sd*t;t=T8(td*t-y2,2.*(td+J));}Ra=t;return abs(t*(t*(t*sd+3.*J)+3.*y2));
#endif
}