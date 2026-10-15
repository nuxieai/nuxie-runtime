#ifndef ld
#define ld f
#endif
#ifndef o7
#define o7 c
#endif
e float X8(c l,c b){float xg=dot(l,b);float md=dot(l,l)*dot(b,b);return(md==.0)?1.:clamp(xg*inversesqrt(md),-1.,1.);}e void yg(c A0,c D0,c K0,c T0,l1(c) A,l1(c) J,l1(c) z2){z2=D0-A0;c p7=K0-D0;c Y8=T0-A0;J=p7-z2;A=-3.*p7+Y8;}e X Xa(c A0,c D0,c K0,c T0){X t;t[0]=(any(notEqual(A0,D0))?D0:any(notEqual(D0,K0))?K0:T0)-A0;t[1]=T0-(any(notEqual(T0,K0))?K0:any(notEqual(K0,D0))?D0:A0);return t;}e float zg(c A0,c D0,c K0,c T0,float H1,float Ag){c A,J,z2;yg(A0,D0,K0,T0,A,J,z2);c q7=3.*(((A*H1)+2.*J)*H1+z2);float nd=length(q7);if(nd==.0){return.0;}q7*=1./nd;float Z8=2.*dot(A,q7);float r7=3.*(Z8*H1+4.*dot(J,q7))*H1+6.*dot(z2,q7);float Ya=min(H1,1.-H1);float Bg=(Z8*Ya*Ya+r7)*Ya;float od=min(Ag,Bg*.9999);float p3;if(Z8==.0){p3=od/r7;}else{float R=1./Z8;float b=r7*R,T1=-od*R;float v7=(-1./3.)*b,w7=.5*T1;float pd=w7*w7-v7*v7*v7;if(pd<.0){float a9=sqrt(v7);float g1=acos(w7/(a9*a9*a9));p3=-2.*a9*cos(g1*(1./3.)+(-p4*2./3.));}else{float A=pow(abs(w7)+sqrt(pd),1./3.);if(w7<.0) A=-A;p3=A!=.0?A+v7/A:.0;}}p3=abs(p3);f t0011=H1+ld(-p3,-p3,p3,p3);f qd=(A.xyxy*t0011+2.*J.xyxy)*t0011+z2.xyxy;X A2=Xa(A0,D0,K0,T0);c Cg=t0011.x<1e-3?A2[0]:qd.xy;c Dg=t0011.z>1.-1e-3?A2[1]:qd.zw;return acos(X8(Cg,Dg));}e float c9(float l,float b){l=b<.0?-l:l;b=abs(b);return l>.0?(l<b?l/b:1.):.0;}float Eg(c A0,c D0,c K0,c T0,l1(float) Za){c rd=T0-A0;float sd=length(T0-A0);if(sd==.0){Za=.5;return.0;}c P1=o7(-rd.y,rd.x)/sd;float td=dot(P1,K0-A0);float W4=dot(P1,D0-A0);float X4=W4-td;
#if 0
float l=3.*X4;float ud=X4+W4;float T1=W4;float K2=sqrt(max(X4*X4+td*W4,.0));if(ud<.0) K2=-K2;K2+=ud;c x7=o7(c9(K2,l),c9(T1,K2));c q6=3.*(x7*(x7*(x7*X4-(W4+X4))+W4));q6=abs(q6);Za=q6.x>q6.y?x7.x:x7.y;return max(q6.x,q6.y);
#else
float vd=3.*X4;float J=-W4-X4;float z2=W4;float t=.5;for(int M0=0;M0<3;++M0){float wd=vd*t;t=c9(wd*t-z2,2.*(wd+J));}Za=t;return abs(t*(t*(t*vd+3.*J)+3.*z2));
#endif
}
