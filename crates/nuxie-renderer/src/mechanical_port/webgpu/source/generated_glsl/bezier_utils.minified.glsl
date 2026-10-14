#ifndef kd
#define kd f
#endif
#ifndef m7
#define m7 c
#endif
e float V8(c m,c b){float vg=dot(m,b);float ld=dot(m,m)*dot(b,b);return(ld==.0)?1.:clamp(vg*inversesqrt(ld),-1.,1.);}e void wg(c z0,c D0,c K0,c T0,k1(c) A,k1(c) J,k1(c) y2){y2=D0-z0;c n7=K0-D0;c W8=T0-z0;J=n7-y2;A=-3.*n7+W8;}e X Ua(c z0,c D0,c K0,c T0){X t;t[0]=(any(notEqual(z0,D0))?D0:any(notEqual(D0,K0))?K0:T0)-z0;t[1]=T0-(any(notEqual(T0,K0))?K0:any(notEqual(K0,D0))?D0:z0);return t;}e float xg(c z0,c D0,c K0,c T0,float E1,float yg){c A,J,y2;wg(z0,D0,K0,T0,A,J,y2);c o7=3.*(((A*E1)+2.*J)*E1+y2);float md=length(o7);if(md==.0){return.0;}o7*=1./md;float X8=2.*dot(A,o7);float p7=3.*(X8*E1+4.*dot(J,o7))*E1+6.*dot(y2,o7);float Va=min(E1,1.-E1);float zg=(X8*Va*Va+p7)*Va;float nd=min(yg,zg*.9999);float p3;if(X8==.0){p3=nd/p7;}else{float R=1./X8;float b=p7*R,R1=-nd*R;float q7=(-1./3.)*b,r7=.5*R1;float od=r7*r7-q7*q7*q7;if(od<.0){float Y8=sqrt(q7);float f1=acos(r7/(Y8*Y8*Y8));p3=-2.*Y8*cos(f1*(1./3.)+(-p4*2./3.));}else{float A=pow(abs(r7)+sqrt(od),1./3.);if(r7<.0) A=-A;p3=A!=.0?A+q7/A:.0;}}p3=abs(p3);f t0011=E1+kd(-p3,-p3,p3,p3);f pd=(A.xyxy*t0011+2.*J.xyxy)*t0011+y2.xyxy;X z2=Ua(z0,D0,K0,T0);c Ag=t0011.x<1e-3?z2[0]:pd.xy;c Bg=t0011.z>1.-1e-3?z2[1]:pd.zw;return acos(V8(Ag,Bg));}e float Z8(float m,float b){m=b<.0?-m:m;b=abs(b);return m>.0?(m<b?m/b:1.):.0;}float Cg(c z0,c D0,c K0,c T0,k1(float) Wa){c qd=T0-z0;float rd=length(T0-z0);if(rd==.0){Wa=.5;return.0;}c N1=m7(-qd.y,qd.x)/rd;float sd=dot(N1,K0-z0);float W4=dot(N1,D0-z0);float X4=W4-sd;
#if 0
float m=3.*X4;float td=X4+W4;float R1=W4;float J2=sqrt(max(X4*X4+sd*W4,.0));if(td<.0) J2=-J2;J2+=td;c v7=m7(Z8(J2,m),Z8(R1,J2));c q6=3.*(v7*(v7*(v7*X4-(W4+X4))+W4));q6=abs(q6);Wa=q6.x>q6.y?v7.x:v7.y;return max(q6.x,q6.y);
#else
float ud=3.*X4;float J=-W4-X4;float y2=W4;float t=.5;for(int M0=0;M0<3;++M0){float vd=ud*t;t=Z8(vd*t-y2,2.*(vd+J));}Wa=t;return abs(t*(t*(t*ud+3.*J)+3.*y2));
#endif
}