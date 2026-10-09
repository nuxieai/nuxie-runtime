#ifndef jd
#define jd f
#endif
#ifndef l7
#define l7 c
#endif
e float U8(c m,c b){float vg=dot(m,b);float kd=dot(m,m)*dot(b,b);return(kd==.0)?1.:clamp(vg*inversesqrt(kd),-1.,1.);}e void wg(c z0,c D0,c K0,c T0,k1(c) A,k1(c) J,k1(c) y2){y2=D0-z0;c m7=K0-D0;c V8=T0-z0;J=m7-y2;A=-3.*m7+V8;}e X Ua(c z0,c D0,c K0,c T0){X t;t[0]=(any(notEqual(z0,D0))?D0:any(notEqual(D0,K0))?K0:T0)-z0;t[1]=T0-(any(notEqual(T0,K0))?K0:any(notEqual(K0,D0))?D0:z0);return t;}e float xg(c z0,c D0,c K0,c T0,float E1,float yg){c A,J,y2;wg(z0,D0,K0,T0,A,J,y2);c n7=3.*(((A*E1)+2.*J)*E1+y2);float ld=length(n7);if(ld==.0){return.0;}n7*=1./ld;float W8=2.*dot(A,n7);float o7=3.*(W8*E1+4.*dot(J,n7))*E1+6.*dot(y2,n7);float Va=min(E1,1.-E1);float zg=(W8*Va*Va+o7)*Va;float md=min(yg,zg*.9999);float o3;if(W8==.0){o3=md/o7;}else{float R=1./W8;float b=o7*R,R1=-md*R;float p7=(-1./3.)*b,q7=.5*R1;float nd=q7*q7-p7*p7*p7;if(nd<.0){float X8=sqrt(p7);float f1=acos(q7/(X8*X8*X8));o3=-2.*X8*cos(f1*(1./3.)+(-p4*2./3.));}else{float A=pow(abs(q7)+sqrt(nd),1./3.);if(q7<.0) A=-A;o3=A!=.0?A+p7/A:.0;}}o3=abs(o3);f t0011=E1+jd(-o3,-o3,o3,o3);f od=(A.xyxy*t0011+2.*J.xyxy)*t0011+y2.xyxy;X z2=Ua(z0,D0,K0,T0);c Ag=t0011.x<1e-3?z2[0]:od.xy;c Bg=t0011.z>1.-1e-3?z2[1]:od.zw;return acos(U8(Ag,Bg));}e float Y8(float m,float b){m=b<.0?-m:m;b=abs(b);return m>.0?(m<b?m/b:1.):.0;}float Cg(c z0,c D0,c K0,c T0,k1(float) Wa){c pd=T0-z0;float qd=length(T0-z0);if(qd==.0){Wa=.5;return.0;}c N1=l7(-pd.y,pd.x)/qd;float rd=dot(N1,K0-z0);float W4=dot(N1,D0-z0);float X4=W4-rd;
#if 0
float m=3.*X4;float sd=X4+W4;float R1=W4;float J2=sqrt(max(X4*X4+rd*W4,.0));if(sd<.0) J2=-J2;J2+=sd;c r7=l7(Y8(J2,m),Y8(R1,J2));c q6=3.*(r7*(r7*(r7*X4-(W4+X4))+W4));q6=abs(q6);Wa=q6.x>q6.y?r7.x:r7.y;return max(q6.x,q6.y);
#else
float td=3.*X4;float J=-W4-X4;float y2=W4;float t=.5;for(int M0=0;M0<3;++M0){float ud=td*t;t=Y8(ud*t-y2,2.*(ud+J));}Wa=t;return abs(t*(t*(t*td+3.*J)+3.*y2));
#endif
}