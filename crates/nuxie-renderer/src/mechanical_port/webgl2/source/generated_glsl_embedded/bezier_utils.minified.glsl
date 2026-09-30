#ifndef kc
#define kc f
#endif
#ifndef Q6
#define Q6 c
#endif
e float Y9(c l,c b){float Af=dot(l,b);float lc=dot(l,l)*dot(b,b);return(lc==.0)?1.:clamp(Af*inversesqrt(lc),-1.,1.);}e void Bf(c x0,c B0,c F0,c M0,c1(c)C,c1(c)H,c1(c)l2){l2=B0-x0;c R6=F0-B0;c k8=M0-x0;H=R6-l2;C=-3.*R6+k8;}e e0 Z9(c x0,c B0,c F0,c M0){e0 t;t[0]=(any(notEqual(x0,B0))?B0:any(notEqual(B0,F0))?F0:M0)-x0;t[1]=M0-(any(notEqual(M0,F0))?F0:any(notEqual(F0,B0))?B0:x0);return t;}e float Cf(c x0,c B0,c F0,c M0,float w1,float Df){c C,H,l2;Bf(x0,B0,F0,M0,C,H,l2);c S6=3.*(((C*w1)+2.*H)*w1+l2);float mc=length(S6);if(mc==.0){return.0;}S6*=1./mc;float l8=2.*dot(C,S6);float T6=3.*(l8*w1+4.*dot(H,S6))*w1+6.*dot(l2,S6);float aa=min(w1,1.-w1);float Ef=(l8*aa*aa+T6)*aa;float nc=min(Df,Ef*.9999);float c3;if(l8==.0){c3=nc/T6;}else{float M=1./l8;float b=T6*M,J1=-nc*M;float U6=(-1./3.)*b,V6=.5*J1;float oc=V6*V6-U6*U6*U6;if(oc<.0){float m8=sqrt(U6);float f1=acos(V6/(m8*m8*m8));c3=-2.*m8*cos(f1*(1./3.)+(-H3*2./3.));}else{float C=pow(abs(V6)+sqrt(oc),1./3.);if(V6<.0)C=-C;c3=C!=.0?C+U6/C:.0;}}c3=abs(c3);f t0011=w1+kc(-c3,-c3,c3,c3);f pc=(C.xyxy*t0011+2.*H.xyxy)*t0011+l2.xyxy;e0 L2=Z9(x0,B0,F0,M0);c Ff=t0011.x<1e-3?L2[0]:pc.xy;c Gf=t0011.z>1.-1e-3?L2[1]:pc.zw;return acos(Y9(Ff,Gf));}e float n8(float l,float b){l=b<.0?-l:l;b=abs(b);return l>.0?(l<b?l/b:1.):.0;}float Hf(c x0,c B0,c F0,c M0,c1(float)ba){c qc=M0-x0;float rc=length(M0-x0);if(rc==.0){ba=.5;return.0;}c d3=Q6(-qc.y,qc.x)/rc;float sc=dot(d3,F0-x0);float D4=dot(d3,B0-x0);float E4=D4-sc;
#if 0
float l=3.*E4;float tc=E4+D4;float J1=D4;float x2=sqrt(max(E4*E4+sc*D4,.0));if(tc<.0)x2=-x2;x2+=tc;c W6=Q6(n8(x2,l),n8(J1,x2));c c6=3.*(W6*(W6*(W6*E4-(D4+E4))+D4));c6=abs(c6);ba=c6.x>c6.y?W6.x:W6.y;return max(c6.x,c6.y);
#else
float uc=3.*E4;float H=-D4-E4;float l2=D4;float t=.5;for(int H0=0;H0<3;++H0){float vc=uc*t;t=n8(vc*t-l2,2.*(vc+H));}ba=t;return abs(t*(t*(t*uc+3.*H)+3.*l2));
#endif
}
